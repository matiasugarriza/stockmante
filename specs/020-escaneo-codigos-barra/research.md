# Research: Escaneo y generación de códigos de barras/SKU

**Feature**: [spec.md](./spec.md) · **Plan**: [plan.md](./plan.md) · **Fecha**: 2026-11-07

Cada decisión sigue el formato Decisión / Justificación / Alternativas. Las versiones de paquetes
se fijan de forma exacta en `frontend/package-lock.json` al implementar; las citadas acá son las
vigentes en npm el 2026-11-07.

## Verificación de esquema (P-V)

**Estado**: contrastado contra `migrations/`, **no** contra la base en ejecución (en esta sesión no
estaba disponible el MCP de PostgreSQL ni `psql`). La tarea T000 lo repite contra la BD real antes
de tocar el backend.

| Tabla | Columna | Tipo / restricción | Fuente |
|---|---|---|---|
| `productos` | `codigo_barras` | `VARCHAR(64) NULL`, `UNIQUE` (`productos_codigo_barras_key`, NULLS DISTINCT) | 0002, 0006 |
| `productos` | `sku` | `VARCHAR(30) NOT NULL UNIQUE`, `CHECK (sku ~ '^[A-Z0-9][A-Z0-9-]{2,29}$')` | 0002 |
| `productos` | `activo` | `BOOLEAN NOT NULL DEFAULT TRUE` | 0002 |
| `auditoria` | `tabla_ref` / `accion` | `VARCHAR(32)` / `VARCHAR(24)`, sin `CHECK` de valores | 0008 |

Conclusión: **no hay migración nueva**. Las acciones nuevas de auditoría caben en `VARCHAR(24)`
(`ASIGNAR_CODIGO_BARRAS` = 21, `REEMPLAZAR_CODIGO_BARRAS` = 24 caracteres).

**Hallazgo**: `7791234567890`, usado como ejemplo en tests de 003/004/010, **no** es un EAN-13
válido (su dígito verificador es 8). Con la nueva validación de FR-143 esos tests deben pasar a
`7791234567898`. T000 además cuenta en la BD real los códigos de 13 dígitos con dígito
verificador inválido: siguen siendo legibles y buscables, pero no podrán volver a guardarse sin
corregirse.

## R1 — Lectura por cámara en el navegador

- **Decisión**: `barcode-detector` (3.x, ponyfill de la API estándar `BarcodeDetector` sobre
  `zxing-wasm`), usado **siempre** en modo ponyfill y cargado con `import()` diferido solo al abrir
  la cámara. El `.wasm` se sirve desde la propia app (`zxing-wasm/reader/zxing_reader.wasm?url` +
  `setZXingModuleOverrides({ locateFile })`), nunca desde un CDN. Formatos pedidos al detector:
  `ean_13`, `ean_8`, `upc_a`, `code_128`, `code_39`, `qr_code` (FR-157). Video con
  `getUserMedia({ video: { facingMode: { ideal: 'environment' } } })` para usar la cámara trasera.
- **Justificación**: el `BarcodeDetector` nativo existe en Chrome Android pero no en Safari iOS ni
  en Firefox; como no sabemos qué celular se usa, el ponyfill da el mismo comportamiento en todos y
  un solo camino de código para testear (P-IV). La carga diferida evita penalizar el resto de la
  SPA. Servir el wasm local mantiene la app funcionando en la red local sin internet.
- **Alternativas**: `BarcodeDetector` nativo + fallback manual (deja afuera a iPhone);
  `html5-qrcode` (trae su propia UI y está poco mantenido); `@zxing/browser` (en modo
  mantenimiento); `quagga2` (solo lineales, sin QR).

## R2 — Lector físico tipo pistola

- **Decisión**: sin código específico. El lector emula teclado y escribe en el buscador con el
  foco; cada pantalla ya busca con Enter o con debounce. Se agrega `normalizarLectura()` (trim,
  quita espacios internos y caracteres de control como el separador GS `\x1d` que algunos lectores
  envían en Code 128) y se aplica tanto a la lectura por cámara como al texto del buscador cuando
  el usuario pulsa Enter.
- **Justificación**: FR-135 pide «sin configuración especial»; detectar ráfagas de teclas sería
  sobreingeniería (P-I).
- **Alternativas**: WebHID/WebSerial para lectores en modo serie (innecesario: el usuario no tiene
  lector y los del mercado emulan teclado).

## R3 — Resolución de una lectura a un producto

- **Decisión**: función de frontend `resolverLectura(valor)` en `$lib/api/productos.ts` que usa el
  endpoint existente `GET /api/productos/buscar` con coincidencia **exacta**: primero
  `campo=codigo`, y si no hay resultado `campo=sku`, ambos con `incluir_inactivos=true` y
  `limite=2` (`incluir_inactivos=true` solo para gestión: el backend de 004 responde 403 a
  operación). Devuelve `{ estado: 'encontrado' | 'inactivo' | 'no_encontrado', producto? }`. Un
  valor de más de 160 caracteres (límite `Q_MAX`) se resuelve como `no_encontrado` sin pedir nada.
  Los errores de red se propagan sin perder el valor leído (FR-155).
- **Justificación**: el backend ya resuelve exacto por código y por SKU normalizado (004); no hace
  falta un endpoint nuevo. **Consecuencia (implementación 2026-11-07)**: operación no ve productos
  inactivos, así que un inactivo escaneado por operación se informa como «No se encontró un
  producto activo con el código …» y nunca se agrega (FR-141 se cumple en lo que importa: no se
  vende); gestión ve el aviso explícito de «dado de baja». Se evita `campo=todos` porque también busca por similitud de nombre y
  eso «aproximaría» un producto (FR-140). `incluir_inactivos=true` permite informar «dado de baja»
  en vez de «no encontrado» (FR-141).
- **Alternativas**: endpoint nuevo `GET /api/productos/por-codigo/{valor}` (duplica la búsqueda
  existente); `campo=todos` (viola FR-140).

## R4 — EAN-13: validación y generación

- **Decisión**: funciones puras en `backend/src/models/codigo_barras.rs`:
  `digito_verificador_ean13(&str) -> Result<char, AppError>` (12 dígitos), `es_ean13_valido(&str)`
  y `ean13_automatico(producto_id: i32) -> String` = `"20"` + id con ceros a 10 dígitos + dígito
  verificador. `validar_codigo_barras` agrega una regla: **si el código tiene exactamente 13
  dígitos numéricos, su dígito verificador DEBE ser correcto** (422 con el dígito esperado en el
  mensaje). Se replica la misma lógica en `frontend/src/lib/codigos/ean13.ts` solo para la vista
  previa y para decidir si se ofrece el formato EAN-13 en la etiqueta; el backend es la autoridad.
- **Justificación**: el prefijo `20`–`29` está reservado por GS1 para circulación interna, así que
  no choca con códigos reales de proveedores. Derivarlo del `id` lo hace determinista (testeable
  sin reloj ni aleatoriedad, P-IV) y único por construcción; solo colisiona si alguien cargó a mano
  ese mismo valor en otro producto, caso que cubre la `UNIQUE` (409, se sugiere ingresar 12
  dígitos). Validar el dígito en `validar_codigo_barras` cubre ficha, endpoint nuevo e importación
  CSV (016) con una sola regla.
- **Alternativas**: secuencia propia en BD (requiere migración); número aleatorio (no determinista);
  validar EAN-8/UPC-A también (no lo pide la spec; esos códigos se leen pero no se generan).

## R5 — Asignar o reemplazar el código (API)

- **Decisión**: endpoint nuevo `POST /api/productos/{id}/codigo-barras` (solo gestión) con
  `{ modo: 'automatico' | 'manual', valor?: string, reemplazar: boolean }`. En `manual`, `valor`
  admite 12 dígitos (se calcula el verificador) o 13 (se valida). Transacción con
  `SELECT … FOR UPDATE`; si el producto ya tiene código y `reemplazar` no es `true` → 409
  `codigo_barras_existente` sin cambios; duplicado → 409 `codigo_barras_duplicado` sin cambios.
  Audita `ASIGNAR_CODIGO_BARRAS` o `REEMPLAZAR_CODIGO_BARRAS` en la misma transacción (FR-149).
  `PUT /api/productos/{id}` no cambia su contrato (003): la confirmación de reemplazo para la ficha
  la pide la interfaz y la auditoría sigue siendo `ACTUALIZAR`.
- **Justificación**: la confirmación de reemplazo y la acción de auditoría específica necesitan una
  operación con intención explícita; agregarlas al `PUT` rompería a sus clientes (ficha e
  importación). La regla de reemplazo vive en el servidor para que no dependa de la UI (SC-057).
- **Alternativas**: extender el `PUT` con `reemplazar_codigo` (rompe compatibilidad);
  pre-chequeo de duplicado con `SELECT` (TOCTOU; se usa el `23505` como ya hace 003).

## R6 — Etiqueta imprimible

- **Decisión**: generación **en el navegador** como SVG: `jsbarcode` (3.12.x) para EAN-13 y
  Code 128 y `qrcode-generator` (2.0.x) para QR, ambos cargados con `import()` diferido al abrir la
  etiqueta. Etiqueta estándar de **50 × 30 mm** con símbolo, nombre (recortado a 2 líneas), SKU y
  el valor legible. Imprimir con `window.print()` y `@page { size: 50mm 30mm; margin: 0 }` sobre un
  contenedor exclusivo de impresión; descargar como `.svg` (vectorial, no pierde nitidez). La
  vista previa muestra la etiqueta a tamaño real y el aviso «Imprimí al 100 % (tamaño real)».
  Formatos: EAN-13 solo si el código es un EAN-13 válido; Code 128 y QR codifican el código o, si
  no hay, el SKU (FR-158, FR-159). El formato no se persiste.
- **Justificación**: no hay nada que guardar (la etiqueta se deriva del producto), así que no hace
  falta backend; SVG garantiza barras nítidas a cualquier escala de impresora. Dos librerías chicas
  y estables son más simples y seguras que implementar Code 128 y QR a mano.
- **Alternativas**: `bwip-js` (una sola librería pero mucho más pesada); generar en Rust con un
  crate (agrega endpoint y dependencia al backend sin necesidad); implementar QR propio (complejo y
  propenso a etiquetas ilegibles); PDF (dependencia extra y otra pila de impresión).

## R7 — HTTPS en la red local para el celular

- **Decisión**: modo de Vite `movil`: `npm run dev:movil` = `vite --host --mode movil`. En ese
  modo `vite.config.ts` agrega `@vitejs/plugin-basic-ssl` (2.3.x, peer `vite ^8`, solo
  devDependency) y `server.proxy` de `/api` a `http://localhost:3000`, y define `PUBLIC_API_URL` vacío para que el
  cliente use rutas relativas al mismo origen (no se usa `.env.movil`: el `.gitignore` excluye
  `.env.*`). El celular abre
  `https://<IP-del-PC>:5173` y acepta el certificado autofirmado una vez. El modo normal
  (`npm run dev`) no cambia.
- **Justificación**: `getUserMedia` solo existe en contextos seguros; además, en el celular
  `localhost:3000` apuntaría al propio teléfono, así que la API debe ir por el mismo origen. El
  proxy evita contenido mixto y CORS, y la cookie de sesión (`Secure` solo en producción,
  `SameSite=Lax`) funciona igual por HTTPS. Elegido por el usuario en clarify.
- **Alternativas**: túnel público (expone la app y depende de un tercero; descartado en clarify);
  certificado `mkcert` (requiere instalar una CA en el celular); abrir solo con la webcam del PC.

## R8 — Integración en las 5 pantallas

- **Decisión**: un componente `BotonEscanear.svelte` (botón etiquetado «Escanear con cámara» que
  abre `EscanerCamara.svelte`, un overlay con video, guía de encuadre, «Cancelar» y avisos con
  `aria-live`) que emite `onLectura(valor)`. Cada pantalla decide qué hacer con la lectura usando
  `resolverLectura`:

  | Pantalla | Archivo | Acción ante una lectura encontrada |
  |---|---|---|
  | Caja | `routes/caja/+page.svelte` | muestra ese producto como único resultado, con stock |
  | Movimientos | `routes/movimientos/+page.svelte` | selecciona el producto en la línea activa |
  | Transferencias | `routes/transferencias/+page.svelte` | selecciona el producto a transferir |
  | Conteos | `routes/conteos/+page.svelte` | resalta la línea del producto y enfoca su cantidad contada |
  | Catálogo | `routes/admin/productos/+page.svelte` | abre la ficha del producto |

  No encontrado → aviso y foco en el buscador manual con el valor precargado; en catálogo un
  usuario de gestión ve además «Crear producto con este código». Inactivo → aviso «dado de baja» y
  no se agrega a la operación. Cada activación produce una sola lectura y apaga la cámara (FR-153).
- **Justificación**: un solo componente para cámara/errores/accesibilidad (P-II) y el
  comportamiento específico en cada pantalla, que ya conoce su estado.
- **Alternativas**: escáner global en la cabecera (no sabe qué hacer con la lectura en cada
  contexto); modo continuo para conteos (fuera de alcance por clarify, P-I).

## R9 — Estado del escáner y manejo de errores

- **Decisión**: lógica pura en `frontend/src/lib/codigos/escaner.ts`: `normalizarLectura`,
  `clasificarErrorCamara(error, entorno)` → `'inseguro' | 'sin_soporte' | 'permiso_denegado' |
  'sin_camara' | 'ocupada' | 'desconocido'` con su mensaje (qué pasó, por qué, qué hacer: P-III), y
  `elegirLectura(detecciones)` que toma una sola lectura (la de mayor área si hay varias iguales,
  la primera si difieren). `EscanerCamara.svelte` solo orquesta video, bucle de detección (cada
  ~150 ms con `requestAnimationFrame`) y limpieza de pistas en `onDestroy`.
- **Justificación**: separa la I/O (cámara) de las reglas para testearlas en Vitest `node` sin
  navegador (P-IV).
- **Alternativas**: máquina de estados con librería (innecesario, P-I).
