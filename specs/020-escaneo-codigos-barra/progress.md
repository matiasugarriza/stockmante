# Progress: 020-escaneo-codigos-barra

## 2026-11-07 — clarify, plan y tasks

**Clarify** (5 preguntas, ver `spec.md § Clarifications`):

- Cámara del celular como medio principal (no hay lector físico); pistola que emula teclado y
  entrada manual como alternativas.
- HTTPS en la red local con certificado autofirmado y API por el mismo origen (`npm run dev:movil`).
- Formatos al generar/imprimir: EAN-13, Code 128 y QR; la cámara lee además EAN-8, UPC-A y Code 39.
- La etiqueta codifica el código del producto (o el SKU si no tiene, en Code 128/QR); el formato se
  elige al imprimir y no se persiste.
- Escaneo en los 5 buscadores: caja, movimientos, transferencias, conteos y catálogo.
- Se agregaron FR-157 … FR-161 y SC-060, SC-061. QR salió de «fuera de alcance».

**Correcciones**:

- `spec.md` tenía doble codificación UTF-8 (`Ã³`); se reparó y se normalizaron los fines de línea.
- El ejemplo `7791234567890` no es un EAN-13 válido (verificador 8): la spec pasa a
  `7791234567898`; T019 corrige los tests que lo usan.

**Plan**: sin migración; sin endpoint de lectura (se reutiliza la búsqueda exacta de 004); un
endpoint nuevo `POST /api/productos/{id}/codigo-barras`; 4 dependencias de frontend justificadas en
`plan.md § Complexity Tracking`.

**Pendiente**:

- T000: verificación de esquema contra la BD real (en esta sesión no había MCP de PostgreSQL ni
  `psql`; se contrastó con `migrations/`).
- Implementación: 43 tareas en `tasks.md` (T000–T042). Ningún RED observado todavía.

## 2026-11-07 — implementación de US1 (MVP cámara)

**Tareas cerradas**: T003–T014 (T002 parcial: las librerías de etiquetas se instalan con US4).

**RED observado** (P-IV):

- T004/T005: `escaner.test.ts` falló por módulo inexistente y `productos.test.ts` por
  `resolverLectura` sin exportar (8 tests rojos); luego 34/34 en verde.
- `mensajeLectura` (agregado a `escaner.ts`): 3 tests rojos (`is not a function`) → verde.
- `lectura.test.ts` (`leerProducto`): falló por módulo inexistente → 4/4 en verde.

**Archivos nuevos**: `frontend/src/lib/codigos/{escaner,lectura}.ts` (+ tests),
`frontend/src/lib/components/codigos/{EscanerCamara,BotonEscanear,AvisoLectura}.svelte`, icono
`camara` en `iconos.ts`, `.claude/launch.json` (configuraciones `frontend` y `frontend-movil`).

**Integración**: Caja (filtra por el campo que coincidió), Movimientos (botón por línea),
Transferencias, Conteos (resalta la línea y enfoca la cantidad; aviso si el producto no está en el
conteo) y Catálogo (abre la ficha o «Crear producto con el código …» precargado en
`ProductoForm`).

**Decisiones**:

- `incluir_inactivos=true` devuelve 403 a operación (regla de 004): `resolverLectura` solo lo pide
  para gestión. Para operación un inactivo se informa como «No se encontró un producto activo con
  el código …». Registrado en `spec.md` (US1 escenario 3) y `research.md` R3.
- `zxing-wasm` como dependencia directa (misma versión que usa `barcode-detector`) para importar
  el `.wasm` con `?url` sin depender del hoisting de npm. El build lo emite como asset local de
  947 kB (414 kB gzip) en un chunk diferido.
- Sin `.env.movil` (el `.gitignore` excluye `.env.*`): el modo `movil` define `PUBLIC_API_URL`
  vacío en `vite.config.ts`.
- Helper `leerProducto` + componente `AvisoLectura` (no estaban en el plan) para que las 5
  pantallas compartan mensajes y reintento (P-II).

**Gates**: `npm test` 418/418, `npm run check` 0 errores y 0 advertencias, `npm run build` OK.
Backend sin cambios en US1.

**Verificado**: `https://localhost:5173` y `https://192.168.1.48:5173` responden en modo `movil`, y
`/api` llega al backend por el proxy (401 JSON de Rust sin sesión).

**Pendiente**:

- T015: prueba manual con el celular (permiso, lectura < 2 s, no encontrado, inactivo, 5
  pantallas, errores de cámara) y capturas en `evidencia/`. El navegador integrado no acepta el
  certificado autofirmado, así que esta prueba solo puede hacerse en el teléfono.
- Revisión visual con una sesión de gestión: Catálogo y Conteos (ver abajo).
- T000 (esquema contra la BD real) sigue pendiente; US1 no toca el backend.

## 2026-11-07 — revisión visual en el navegador (sesión de operación)

- **Caja, Movimientos y Transferencias**: el botón aparece donde se planificó (Caja en las acciones
  del filtro, Movimientos por línea, Transferencias bajo el buscador), con 44 px de alto. A 375 px no
  hay scroll horizontal.
- **Escáner sin cámara** (el panel no tiene): el overlay muestra «No se pudo usar la cámara · No
  encontramos una cámara en este dispositivo…» con Reintentar y Cancelar. Escape lo cierra y el
  foco vuelve al botón. Sin errores de consola propios.
- **No revisables con operación**: `/admin/productos` redirige a `/caja` (guarda de `admin/`), y
  `/api/conteos` y `/api/depositos` responden 403, porque son de gestión (comportamiento previo, no de
  020). En Conteos el botón solo aparece con un conteo abierto. Se revisaron después con un usuario
  de gestión (abajo).

## 2026-11-07 — revisión con gestión y lectura real simulada

Se reemplazó `getUserMedia` en el panel por un `canvas.captureStream()` que dibuja un EAN-13
generado a mano. Así la decodificación real de zxing-wasm se ejercita de punta a punta, sin cámara
física.

- **Catálogo**: «Escanear con cámara» junto a «Nuevo producto». Al leer `7791234567898` aparecieron
  «No se encontró un producto con el código 7791234567898. Buscalo por nombre o SKU.» y «Crear
  producto con el código 7791234567898». Ese botón abre el alta con el código precargado (no se
  guardó nada).
- **Conteos** (conteo #1 abierto): «Escanear producto» encima de la tabla. La lectura del mismo
  código muestra el aviso y el overlay se cierra solo tras leer.
- **Caso «encontrado» sin probar con decodificación real**: ningún producto de la base tiene un
  EAN-13 válido (ver hallazgo). Lo cubren los tests de `resolverLectura` y `leerProducto`; queda
  para T015 con una etiqueta real.
- **Detalle de texto**: en Conteos el «Buscalo por nombre o SKU» no aplica, porque la pantalla no
  tiene buscador. Se puede ajustar en Polish.

**Hallazgo para US3**: los 6 códigos de 13 dígitos de la base (`7790001112223`, `7790002223334`,
`7790003334445`, `7790004445556`, `7790005556667` y `7791234567890`) tienen el dígito verificador
incorrecto. Provienen de `captures/samples/importacion/productos-validos.csv`. Con la regla de
FR-143 ese CSV dejaría de importarse y esos productos no podrían volver a guardarse sin corregir el
código (research R2 ya lo anticipa). T019 debe corregir también ese CSV, y T000 debe contar estos
casos en la BD real.
