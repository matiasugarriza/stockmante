# Validación — spec 018

**Fecha**: 2026-11-07 | **Rama**: `018-rediseno-visual-desktop`

## Gates automatizados

| Gate | Comando | Resultado |
|---|---|---|
| Tests unitarios | `npm run test` (frontend) | 31 archivos, 277 tests en verde (incluye 11 de navegación, 11 de formato, 9 de gráficos y 2 del cliente de reportes) |
| Tipos y plantillas | `npm run check` (frontend) | 0 errores, 0 warnings |
| Svelte con config mínima | `svelte-check --config` (sin Vite/Tailwind) | 0 errores, 0 warnings sobre todas las pantallas re-pieladas |
| Build SPA | `npm run build` (frontend) | correcto (adaptador static, `build/` generado) — previo al re-piel de acceso |

## Marco y tema (US18.1)

Verificado con Playwright sobre el build estático y una sesión simulada por interceptión de
`/api/auth/sesion` (sin backend):

- **1440×900, tema claro**: barra lateral 248 px, topbar 64 px, sin desborde
  (`scrollWidth = 1440`, `innerWidth = 1440`).
- **1440×900, tema oscuro**: misma estructura, paleta azul noche, sin desborde.
- **390×844**: la barra lateral se oculta; al abrir el menú se muestra como panel superpuesto con
  todas las etiquetas y sin desborde (`scrollWidth = 390`, `innerWidth = 390`).
- **OPERACION**: la barra solo muestra Inicio, Caja, Movimientos, Transferencias y Asistente; no
  aparecen Conteos ni los ítems de Administración.
- **Tema**: `light`/`dark` se aplican antes del primer pintado; el selector refleja la preferencia.
- **Preferencia `system`**: con el SO en claro resuelve `light`; con el SO en oscuro resuelve `dark`.
- **Sin sesión**: una ruta privada redirige a `/login`.
- Evidencia preliminar en `evidencia/smoke/`.

## Tablero de Inicio (US18.2)

- Implementado con endpoints existentes: stock bajo mínimo, movimientos del día, productos activos y
  valor del stock (GESTION), movimientos por tipo, estado del stock, actividad reciente y casos
  urgentes.
- Cada widget resuelve su propia carga, vacío y error; los indicadores de gestión se ocultan a
  `OPERACION`.
- `frontend/src/lib/api/reportes.ts` + schema cubiertos con 2 tests.
- **Nota de proceso**: el test de `reportes` se escribió junto con el cliente y quedó verde en su
  primera corrida; no se reconstruyó un RED artificial.

## Acceso (US18.3)

- `/login` y el modo de cambio obligatorio usan `LienzoAcceso`, `Tarjeta`, `Campo`, `Boton` y
  `Aviso`, conservando validaciones, SSO, mensajes y redirecciones de 001.
- `/registro` y `/verificar-correo` comparten el mismo lienzo y patrón.
- Se retiraron los estilos `.login-*` de `routes/layout.css`; la identidad vive en
  `lib/diseno/acceso.css`.

## Operación (US18.4)

- Caja, Movimientos, Transferencias, Conteos y Asistente usan `EncabezadoPagina`, `Tarjeta`,
  `Campo`, `Boton`, `Aviso`, `TablaDatos` y `Paginacion`.
- Se conservaron la búsqueda diferida de Caja, las líneas y validaciones de Movimientos, la
  atomicidad de Transferencias, las operaciones de Conteos y la confirmación del Asistente.
- `TarjetaPropuesta` se re-pieló con tokens.
- `svelte-check` con config mínima: **0 errores, 0 warnings**.

## Administración (US18.5, parcial)

- Re-pieladas **las 8 páginas**: Usuarios, Categorías, Productos, Stock, Proveedores, Compras,
  Auditoría y Depósitos/Ubicaciones, con sus tablas, formularios, modales y badges.
- Depósitos conserva el árbol de ubicaciones, la baja lógica, el borrado condicionado y la
  asignación producto–ubicación; se re-pieló con tarjetas, campos y acciones.
- Se reutilizaron `EncabezadoPagina`, `Tarjeta`, `Campo`, `Boton`, `TablaDatos`, `Paginacion`,
  `Modal`, `Aviso`, `EtiquetaEstado` y `EstadoVacio`.
- `svelte-check` con config mínima: **0 errores, 0 warnings**.
- **Pendiente**: la verificación funcional de estados de las ocho páginas (T079), su evidencia
  (T080) y la revisión final (T081).

## Nota de alcance

El árbol de trabajo contiene cambios de backend en `categorias`, `depositos` y `auditoria` que no
fueron realizados por esta feature (corresponden a otro frente de trabajo). La spec 018 no modificó
`backend/`, `migrations/` ni contratos HTTP; el control de T088 debe revisar solo los archivos de
este frente.

## Tipografía Outfit y segunda pasada de fidelidad (Session 2026-11-07 (2), T090–T099)

- **FR-098**: `@fontsource-variable/outfit@5.3.0` instalado con versión exacta e importado en
  `routes/layout.css`; `--fuente-sans` = `'Outfit Variable'` + pila del sistema. Verificado en el
  navegador: `getComputedStyle(body).fontFamily` empieza por `"Outfit Variable"` y
  `document.fonts.check('16px "Outfit Variable"') === true`. El build empaqueta
  `outfit-latin-wght-normal.*.woff2` y `outfit-latin-ext-wght-normal.*.woff2`; `build/` no contiene
  referencias a `fonts.googleapis.com` ni `fonts.gstatic.com` (**SC-038**).
- La monoespaciada decorativa (kicker y pie del acceso, versión de la barra lateral) pasó a Outfit;
  queda solo para SKU/IDs.
- **FR-099**: acento `#1F3BB3` (claro, 8.99:1 con blanco) y `#3F5EFB` con texto blanco (oscuro,
  4.99:1); barra superior azul marino `#1F2A44` en claro mediante `.zona-oscura` (texto 12.95:1,
  suave 7.88:1); saludo "Buenos días/tardes/noches, <nombre>" en Inicio; ítem activo en píldora;
  marca bicolor; pie de página del marco; KPIs de Inicio como franja de métricas (`TarjetaKpi plano`).
- **Corrección encontrada en la pasada**: `ui/Campo.svelte` no tenía bloque `<style>` (T014 se
  había marcado hecha sin estilos): todos los campos —incluido el login— se veían sin borde ni
  fondo. Se agregaron estilos de campo, select, textarea, foco, error y deshabilitado.
- **Corrección a 320 px**: la barra superior desbordaba (`scrollWidth` 325–370 px) en Inicio,
  Movimientos, Transferencias, Categorías y Proveedores. El título ahora se trunca con elipsis y en
  ≤640 px el selector de tema queda solo en el panel lateral. Las 14 rutas privadas quedan en
  `scrollWidth = 320`; las tablas anchas se desplazan dentro de su tarjeta.
- Con la barra colapsada el selector de tema del pie se oculta (no entraba en 76 px); sigue en la
  barra superior.
- Gates: `npm run test` 40 archivos / 327 tests en verde; `npm run check` 0 errores, 0 warnings;
  `npm run build` correcto.
- Evidencia regenerada con `scripts/capturar-rediseno.mjs` contra la app local: 17 claras, 17
  oscuras y 12 angostas.
- **Observación de entorno**: el widget "Estado del stock" de Inicio muestra "La respuesta del
  servidor no tiene el formato esperado" porque el backend en ejecución es un binario anterior a
  010 (`/api/stock` no devuelve `faltante`, `costo` ni `valorizado`); el código de `backend/src` sí
  los incluye. Se resuelve recompilando/reiniciando el backend, no es un defecto del frontend.

## Pendiente

- Captura del modo de cambio obligatorio de contraseña (requiere `CAPTURE_EMAIL_TEMP`/
  `CAPTURE_PASSWORD_TEMP`).
- Re-capturar Inicio con el backend recompilado (widget "Estado del stock").
- Rebuild posterior al re-piel de acceso y recorrido visual de las cuatro pantallas de acceso.
- Páginas de operación (US18.4) y administración (US18.5).
- Auditoría de contraste AA y de teclado en las 17 rutas.
