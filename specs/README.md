# Specs — índice por caso de uso

**Regla de estructura**: una spec por **caso de uso** (user story), no una spec por módulo.
Cada spec tiene su propio alcance, requisitos, contratos, plan y tareas; la base técnica
compartida vive en `001-autenticacion-sesion`.

**Origen**: hasta el 2026-10-25 existía una sola spec (`specs/001-user-auth-catalog/`) que
agrupaba los 6 casos de uso del Módulo 1. Se dividió a pedido del usuario; el contenido
original queda en el historial de git. Los textos de requisitos, escenarios y supuestos se
conservan **verbatim**; solo se distribuyeron por caso de uso.

## Mapa de specs

| Spec | Caso de uso | Requisitos | Criterios | Estado | Depende de |
|---|---|---|---|---|---|
| [001-autenticacion-sesion](./001-autenticacion-sesion/spec.md) | Login, sesión única y cambio obligatorio de contraseña | FR-001, FR-002, FR-003, FR-008a, FR-070 ... FR-073 | SC-003, SC-004, SC-026 ... SC-028 | Implementado; ajustes 2026-11 (registro, SSO, credenciales) en revisión | — |
| [002-gestion-usuarios](./002-gestion-usuarios/spec.md) | Alta, edición, baja lógica y reactivación de usuarios | FR-004, FR-005, FR-006, FR-007, FR-008, FR-008b | SC-001 | Implementado (retro-documentado); ajustes 2026-11 en Phase 2 | 001 |
| [003-catalogo-productos](./003-catalogo-productos/spec.md) | CRUD de productos y categorías, baja lógica | FR-009, FR-010, FR-012, FR-014, FR-020 … FR-023, FR-074 | SC-006, SC-009 … SC-012, SC-029 | Implementado y verificado; ampliación de ubicaciones implementada y verificada (2026-11-06) | 001 |
| [004-busqueda-caja](./004-busqueda-caja/spec.md) | Búsqueda rápida por SKU, código, nombre y categoría | FR-011, FR-013 | SC-002 | Implementado (2026-10-26); ampliación 2026-11-07 (aviso de truncados, debounce testeado, SC-002 a 10.000 productos, W-006) | 001, 003 |
| [005-depositos-ubicaciones](./005-depositos-ubicaciones/spec.md) | Depósitos, ubicaciones jerárquicas y asignación a producto | FR-015, FR-016, FR-017, FR-018, FR-075 | SC-005 | Implementado y verificado (2026-10-26); lectura masiva ampliada y verificada 2026-11-06 | 001 (003 para asignación) |
| [006-rbac-auditoria](./006-rbac-auditoria/spec.md) | Verificación transversal de RBAC y auditoría básica | FR-003 (verificación), FR-019 (derivado) | SC-007, SC-008, SC-030 | Implementado y verificado (2026-11-07): auditoría de todas las escrituras de gestión, matriz completa, lecturas, degradación y SC-007 | 001–005 |
| [007-movimientos-stock](./007-movimientos-stock/spec.md) | Entradas, salidas, mermas y ajustes con stock por depósito | FR-024 … FR-030 | SC-013 … SC-015 | Implementado y verificado (2026-11-07): matriz tipo ↔ motivo, mensajes con siguiente paso, cantidad entera, W-005 | 001, 003, 005, 006 |
| [008-transferencias-depositos](./008-transferencias-depositos/spec.md) | Transferencias atómicas entre depósitos | FR-031 … FR-033 | SC-016 | Implementado y verificado (2026-11-07): remediación T005–T036 (corregido 500 en alta/listado), W-004 | 007 |
| [009-inventario-fisico](./009-inventario-fisico/spec.md) | Conteo físico y ajuste automático de diferencias | FR-034 … FR-037 | SC-017 | Implementado y verificado por tests (2026-11-07); quickstart manual pendiente (T031) | 007, 005 |
| [010-consulta-stock](./010-consulta-stock/spec.md) | Consulta, desglose y tablero de stock | FR-038 … FR-040 | SC-018 | Implementado y verificado (2026-11-07): faltante, valorizado por perfil, `producto_ids`, `/depositos/activos`; quickstart manual pendiente (T021) | 007, 005 |
| [011-alertas-stock-minimo](./011-alertas-stock-minimo/spec.md) | Alertas in-app y digest por correo bajo mínimo | FR-041 … FR-043 | SC-019 | Implementado y verificado (2026-11-07): `notificaciones_stock` (0015), ciclo con scheduler y digest; quickstart manual pendiente (T026) | 007, 010, 002 |
| [012-proveedores-compras](./012-proveedores-compras/spec.md) | Proveedores, compras y recepción con entrada de stock | FR-044 … FR-048 | SC-020, SC-021 | Implementado y verificado (2026-11-07): fase 6 (anulación concurrente, `ResultadoCompra`, códigos nuevos), W-003 | 007, 003, 005 |
| [013-ventas-caja](./013-ventas-caja/spec.md) | Ventas de caja, clientes y descuento de stock | FR-049 … FR-055 | SC-021 | Implementado y verificado (2026-11-07): POS en `/caja`, `/admin/ventas`, `/admin/clientes` | 003, 004, 005, 007 |
| [014-reportes-kardex](./014-reportes-kardex/spec.md) | Kardex, valorización, rotación, ventas y CSV | FR-056 … FR-059 | SC-022, SC-031 | Implementado y verificado (2026-11-07): cuatro reportes GESTION, días locales, CSV RFC 4180 y `/admin/reportes` | 007, 008, 012, 013 |
| [015-auditoria-consultable](./015-auditoria-consultable/spec.md) | Consulta de auditoría y cobertura ampliada | FR-060 … FR-062 | SC-023 | Especificado | 006, 007, 012, 013 |
| [016-importacion-masiva](./016-importacion-masiva/spec.md) | Importación CSV de productos y stock inicial | FR-063 … FR-065 | SC-024 | Implementado y verificado (2026-11-07): importación atómica, `importacion_invalida` con detalles, SC-024 en 10 s; quickstart manual pendiente (T018) | 003, 007 |
| [017-asistente-inventario](./017-asistente-inventario/spec.md) | Asistente con tools de lectura y escrituras vía API | FR-066 … FR-069, FR-076 … FR-078 | SC-025 | Implementado y verificado (2026-11-07) | 007, 010, 013, 014 |
| [018-rediseno-visual-desktop](./018-rediseno-visual-desktop/spec.md) | Rediseño visual del escritorio (17 capturas) y tema claro/oscuro | FR-079 … FR-099 | SC-032 … SC-038 | Implementado en frontend (2026-11-07) con tipografía Outfit autoalojada; test/check/build en verde y evidencia 1440×900 claro/oscuro + 320 px; pendiente la captura del cambio de contraseña y la revisión manual de escenarios | 001 |

## Partición de requisitos funcionales

Cada FR tiene **un solo dueño**. Los demás specs lo referencian, no lo repiten.

| FR | Dueño | Tema |
|---|---|---|
| FR-001 | 001 | Login y sesión única |
| FR-002 | 001 | Logout |
| FR-003 | 001 | Modelo RBAC de dos familias (002–006 aportan sus filas de permisos) |
| FR-004 | 002 | Unicidad de correo |
| FR-005 | 002 | Baja lógica de usuarios |
| FR-006 | 002 | Guarda del último gestor GESTION activo |
| FR-007 | 002 | CRUD de usuarios solo gestión |
| FR-008 | 002 | Validación de formato de correo |
| FR-008a | 001 | Contraseña temporal: generación, entrega, cambio obligatorio y reseteo |
| FR-008b | 002 | Elección manual/automática de la contraseña en el alta y reseteo, y visualización en pantalla de las credenciales |
| FR-009 | 003 | CRUD de productos y formato de SKU/código |
| FR-010 | 003 | Categorías planas |
| FR-011 | 004 | Búsqueda y filtrado de productos |
| FR-012 | 003 | Borrado físico bloqueado / baja lógica de productos |
| FR-013 | 004 | Inactivos fuera de búsquedas de venta |
| FR-014 | 003 | Precio y baja de productos solo gestión |
| FR-015 | 005 | CRUD de depósitos |
| FR-016 | 005 | Ubicaciones jerárquicas sin huérfanas |
| FR-017 | 005 | Asignación de ubicaciones permitidas por producto |
| FR-018 | 005 | Borrado bloqueado de depósitos con dependencias |
| FR-019 | 006 | Auditoría de todas las escrituras de gestión de 001–005 (usuarios, productos, categorías, depósitos, ubicaciones y asignaciones producto–ubicación) |
| FR-020 | 003 | Listado de catálogo paginado y filtrable |
| FR-021 | 003 | Reactivación de productos y categorías |
| FR-022 | 003 | Productos inactivos sin movimientos nuevos |
| FR-023 | 003 | Categoría activa al crear o recategorizar |
| FR-024 | 007 | Registro de movimientos (tipos, líneas y motivos) |
| FR-025 | 007 | Atomicidad y no negatividad del stock (tabla puente + cache) |
| FR-026 | 007 | Autoría y auditoría de movimientos |
| FR-027 | 007 | Permisos por tipo de movimiento |
| FR-028 | 007 | Validaciones de contexto de un movimiento |
| FR-029 | 007 | Consulta de stock y de movimientos |
| FR-030 | 007 | Resultado concreto en la respuesta de un movimiento |
| FR-031 | 008 | Transferencia atómica entre depósitos |
| FR-032 | 008 | Consulta de transferencias |
| FR-033 | 008 | Permisos y resultado concreto de transferencias |
| FR-034 | 009 | Conteo físico y líneas |
| FR-035 | 009 | Cierre de conteo con ajuste automático |
| FR-036 | 009 | Cancelación y consulta de conteos |
| FR-037 | 009 | Permisos de inventario físico |
| FR-038 | 010 | Listado de stock |
| FR-039 | 010 | Detalle de stock por producto |
| FR-040 | 010 | Vistas de stock y valorizado por rol |
| FR-041 | 011 | Detección de productos bajo mínimo |
| FR-042 | 011 | Digest por correo con deduplicación |
| FR-043 | 011 | Ejecución periódica, on-demand y errores de alertas |
| FR-044 | 012 | CRUD de proveedores |
| FR-045 | 012 | Compras en borrador |
| FR-046 | 012 | Confirmación de compra con entrada de stock y costo |
| FR-047 | 012 | Anulación de compra con reversa |
| FR-048 | 012 | Consultas y permisos de compras |
| FR-049 | 013 | CRUD de clientes |
| FR-050 | 013 | Venta y numeración |
| FR-051 | 013 | Descuento de stock y validaciones de venta |
| FR-052 | 013 | Anulación de venta con reversa |
| FR-053 | 013 | Medios de pago |
| FR-054 | 013 | Consultas de ventas y punto de venta |
| FR-055 | 013 | Permisos de ventas y clientes |
| FR-056 | 014 | Kardex por producto |
| FR-057 | 014 | Stock valorizado |
| FR-058 | 014 | Rotación y ventas por período |
| FR-059 | 014 | Exportación CSV y permisos de reportes |
| FR-060 | 015 | Consulta de auditoría |
| FR-061 | 015 | Cobertura ampliada de auditoría |
| FR-062 | 015 | Retención y permisos de auditoría |
| FR-063 | 016 | Importación CSV en modo validar/aplicar |
| FR-064 | 016 | Stock inicial como movimiento de importación |
| FR-065 | 016 | Límites, errores y permisos de importación |
| FR-066 | 017 | Tools de lectura del asistente |
| FR-067 | 017 | Escrituras del asistente solo por API |
| FR-068 | 017 | Seguridad del token y límites del asistente |
| FR-069 | 017 | Permisos y trazabilidad del asistente |
| FR-070 | 001 | Formulario de registro público con verificación por correo y anti-enumeración |
| FR-071 | 001 | SSO Google/Microsoft con `GET /auth/sso/config` y ocultado si faltan credenciales o falla la salud |
| FR-072 | 001 | Vinculación/desvinculación de identidades externas |
| FR-073 | 001 | Provisión de familia `OPERACION` por defecto en cuentas SSO |
| FR-074 | 003 | Ubicaciones permitidas visibles en la vista de productos (solo lectura) |
| FR-075 | 005 | Lectura masiva de ubicaciones asignadas por producto |
| FR-076 | 017 | Resultado visible tras confirmar una propuesta del asistente |
| FR-077 | 017 | Contexto de conversación: últimos 10 mensajes, solo en memoria |
| FR-078 | 017 | Logs del asistente solo con metadatos, sin contenido |
| FR-079 | 018 | Capa única de tokens visuales para claro y oscuro |
| FR-080 | 018 | Tema Light/Dark/System con persistencia y sin destello inicial |
| FR-081 | 018 | Login rediseñado conservando campos y comportamiento de 001 |
| FR-082 | 018 | Estados visibles y contraste WCAG AA en ambos temas |
| FR-083 | 018 | Cambio obligatorio de contraseña con la misma estética |
| FR-084 | 018 | Adaptabilidad de 320 px a 1440 px sin desborde |
| FR-085 | 018 | Accesibilidad: etiquetas, foco, teclado y movimiento reducido |
| FR-086 | 018 | Identidad visual de referencia (panel de administración, azul/teal/ámbar, tarjetas, claro y oscuro) |
| FR-087 | 018 | Incorporación incremental del rediseño página por página |
| FR-088 | 018 | Marco de escritorio con barra lateral y barra superior |
| FR-089 | 018 | Navegación agrupada con visibilidad por rol y contador de stock |
| FR-090 | 018 | Tablero de Inicio con indicadores, gráficos, actividad y accesos rápidos |
| FR-091 | 018 | Patrón de página operativa con tarjetas de formulario y resultado |
| FR-092 | 018 | Patrón de página de administración con filtros, tabla, badges y paginación |
| FR-093 | 018 | Catálogo de componentes visuales reutilizables |
| FR-094 | 018 | Registro y verificación con el mismo lienzo de acceso |
| FR-095 | 018 | Selector de tema y menú de usuario en el marco |
| FR-096 | 018 | Evidencia visual por página en ambos temas y 320 px |
| FR-097 | 018 | Cero cambios de comportamiento, API, esquema o permisos |

## Partición de criterios de éxito

| SC | Dueño | Verificación |
|---|---|---|
| SC-001 | 002 | Alta de usuario + primer login < 2 min |
| SC-002 | 004 | 95% de búsquedas por SKU/código ≤ 1 s |
| SC-003 | 001 | Login genérico sin enumeración |
| SC-004 | 001 | Cero acciones con sesión de usuario desactivado |
| SC-005 | 005 | Cero ubicaciones sin depósito |
| SC-006 | 003 | Cero SKU y códigos de barras duplicados |
| SC-007 | 006 | Borrados con dependencias bloqueados (productos y depósitos) |
| SC-008 | 006 | Operario no completa operaciones de gestión |
| SC-009 | 003 | Categorías duplicadas rechazadas sin cambios parciales |
| SC-010 | 003 | Ninguna operación reduce filas de productos o categorías |
| SC-011 | 003 | Productos dados de baja siguen accesibles por id y `activo=all` |
| SC-012 | 003 | Listado filtrado ≤ 1 s con 5.000 productos |
| SC-013 | 007 | Cero stock negativo ante salidas o mermas sin stock |
| SC-014 | 007 | Cache global igual a la suma por depósito |
| SC-015 | 007 | Cero movimientos parciales con una línea inválida |
| SC-016 | 008 | Suma global de stock invariante en toda transferencia |
| SC-017 | 009 | Stock igual a lo contado al cerrar cada conteo |
| SC-018 | 010 | Página de stock < 1 s con 10.000 productos |
| SC-019 | 011 | Cero correos duplicados por producto y día |
| SC-020 | 012 | Toda compra confirmada tiene su entrada de stock |
| SC-021 | 013 | Cero ventas con stock negativo y descuento exacto |
| SC-022 | 014 | El kardex reconstruye el stock actual |
| SC-023 | 015 | Auditoría consultable < 2 s con 1.000.000 de filas |
| SC-024 | 016 | 5.000 filas importadas < 60 s o ninguna |
| SC-025 | 017 | Cero escrituras del asistente fuera de la API |
| SC-026 | 001 | Registro con verificación por correo y sin enumeración |
| SC-027 | 001 | Botones SSO ocultos sin credenciales; flujo completo con ellas |
| SC-028 | 001 | Credenciales (correo + contraseña) visibles en pantalla al crear usuario (criterio compartido con SC-001/002; evidencia del módulo en 002 Phase 2, ver dueños §FR-008a/FR-008b) |
| SC-029 | 003 | Una llamada masiva por página y «Sin ubicación» explícito en la vista de productos |
| SC-030 | 006 | Una fila de auditoría por escritura de gestión completada; ninguna por rechazo |
| SC-031 | 014 | Los cuatro reportes (JSON y CSV) solo para `GESTION` |
| SC-032 | 018 | Tema persistido y aplicado antes del primer pintado, sin destellos |
| SC-033 | 018 | Cero regresiones funcionales en los cuatro recorridos de acceso |
| SC-034 | 018 | Sin desborde de 320 a 1440 px y contraste AA en ambos temas |
| SC-035 | 018 | Comparación visual contra la referencia en 1440×900 claro y oscuro |
| SC-036 | 018 | Cada página posterior llega con capturas en ambos temas y cero regresiones |
| SC-037 | 018 | Las 17 capturas rediseñadas y cubiertas por navegación (14 privadas + 3 públicas) |

## Ledger de migraciones

| Versión | Archivo | Propósito | Spec dueña | Creada en |
|---|---|---|---|---|
| 0001 | `0001_usuarios_sesiones.sql` | `usuarios`, `sesiones` | 001 | 001 / T007 |
| 0002 | `0002_categorias_productos.sql` | `categorias`, `productos` | 003 | 001 / T008 (fase foundational) |
| 0003 | `0003_depositos_ubicaciones.sql` | `depositos`, `ubicaciones`, `producto_ubicacion_permitida` | 005 | 001 / T009 (fase foundational) |
| 0004 | — (no existe) | Reservado por el WIP de 003; nunca se publicó y **no se reutiliza** | — | — |
| 0005 | `0005_sesion_unica.sql` | Índice único parcial de sesión vigente | 001 | 001 / T027 (T-AUTH-09) |
| 0006 | `0006_catalogo_descripcion_codigo_barras.sql` | `categorias.descripcion`, unicidad normalizada, `codigo_barras` `NULLS DISTINCT` e índice de `categoria_id` | 003 | 003 / T005 |
| 0007 | `0007_busqueda_trgm.sql` | Extensión `pg_trgm` + índice GIN de `productos.nombre` | 004 | 004 / T001 |
| 0008 | `0008_auditoria.sql` | Tabla de auditoría básica de usuarios y productos | 006 | 006 / T001 |
| 0009 | `0009_stock_movimientos.sql` | `stock_por_deposito`, `movimientos`, `movimiento_lineas`, `productos.stock_actual` | 007 | 007 / T001 |

> La numeración es global y **no se reutiliza**. La fase foundational de 001 creó las tablas
> de 003 y 005 para no bloquear las user stories; la propiedad del esquema queda en el spec
> que lo usa.
>
> La `0004` perteneció al WIP de 003 y se retiró con él (nunca se publicó); el replanteo absorbió su
> contenido en la `0006` en lugar de reciclar el número. Las reservas anteriores (`0005` para 004 y
> `0006` para 006) quedaron desactualizadas cuando 001 creó `0005_sesion_unica.sql`; 004 tomó la
> `0007` y 006 la `0008`.

### Reservas para las specs planificadas (no creadas todavía)

| Versión | Archivo previsto | Propósito | Spec dueña |
|---|---|---|---|
| 0015 | `0015_alertas_stock.sql` | `notificaciones_stock` | 011 |
| 0016 | `0016_auditoria_consultas.sql` | Índices de consulta de auditoría | 015 |

> 0010–0014 ya están creadas: `0010_transferencias` (008), `0011_inventario_fisico` (009),
> `0012_registro_sso` (001), `0013_proveedores_compras` (012) y `0014_ventas` (013).
> 011 tomó la `0015` al generar su `plan.md`, así que 015 se corre a la `0016`.

> Las reservas son orientativas hasta que cada spec genere su `plan.md` y verifique el esquema real.
> Un número reservado que no se use **no se reutiliza**: queda como hueco, igual que la `0004`.
> Las specs 010, 014, 016 y 017 no prevén migración propia.

## Convenciones de tareas

- Las specs creadas después del split reinician en `T001` y llevan una tabla de trazabilidad
  con el ID anterior cuando aplica.
- 001 conserva los IDs originales `T001–T025`; 002 renumera `T026–T031` → `T001–T006` con
  trazabilidad; 003 usa `T001–T037` y la ampliación de ubicaciones continúa en `T038+`.
- Cada spec cierra con sus gates: `cargo test`, `cargo sqlx prepare --check`, `npm test`,
  `npm run check`, `npm run build` cuando corresponda.

## Cierre transversal del Módulo 1 (ex T048–T052)

Estas tareas no pertenecen a un caso de uso; se ejecutan cuando 001–006 estén cerradas.

- [X] **C1** Actualizar `README.md` de la raíz (setup local, env, comandos, mapa de specs).
  Completado con la spec 000 (2026-10-26).
- [X] **C2** Correr los gates completos del monorepo y el grep de `unwrap`/`panic!`/`expect`.
  Ejecutado el 2026-10-26: `cargo test` **115/115**, `cargo sqlx prepare --check` exit 0 (aviso
  preexistente de `.sqlx`), `npm test` **136/136**, `npm run check` **0/0**, `npm run build` OK.
  El grep en `backend/src` solo encuentra las palabras en comentarios; no hay pánicos en
  producción.
- [X] **C3** Ejecutar los escenarios V1–V8 de los quickstarts de 001–005 y registrar resultados.
  Ejecutado en su forma automatizada equivalente contra el router y la base reales:
  V1/V2/V4/V9/V10 → `test_auth_http`, `test_sesion_unica`, `test_cambio_password`;
  V3/V5 → `test_usuarios_http`, `test_ultimo_admin`, `test_usuarios_email`;
  V6a/V6b → `test_categorias_http`, `test_productos_http`, `test_catalogo_baja_logica`;
  búsqueda de caja → `test_busqueda_http`; V7 → `test_depositos_http`; RBAC/auditoría →
  `test_rbac_matriz`. Smoke del servidor real: `/api/salud` 200, `/docs` 200 y 29 rutas en
  `/api-docs/openapi.json`. Los pasos exclusivamente visuales de navegador se cubren con
  `npm run check` y `npm run build`; no se repitió el recorrido manual con capturas.
- [X] **C4** Revisar mensajes de error accionables en español (Principio III). Revisados
  `backend/src/error.rs`, los servicios y los mensajes de UI: no quedan textos en inglés ni
  códigos sin mensaje accionable. Los códigos nuevos de 005 están documentados en
  `docs/api.md`.
- [X] **C5** Commit final de cierre del Módulo 1 y actualización de estado de las specs.

## Roadmap posterior al Módulo 1

El Módulo 1 queda cerrado con 001–006. Lo que estaba fuera de su alcance se especificó sin
implementar todavía, ordenado por dependencia real (no por número de spec):

| Módulo | Specs | Foco |
|---|---|---|
| 2 — Operación de stock | 007, 008, 009, 010, 011 | Movimientos, transferencias, consulta, conteo y alertas |
| 3 — Abastecimiento y ventas | 012, 013, 014 | Compras con proveedores, ventas y reportes de salida |
| 4 — Gobierno y asistencia | 015, 016, 017 | Auditoría consultable, importación y asistente |

**Orden de implementación** (por dependencia real, no por número de spec):
007 → 008 → 010 → 013 → 009 → 011 → 012 → 014 → 015 → 016 → 017.

**Prioridades**: P1 = 007, 008, 010 y 013 (sin ellas no hay inventario operable); P2 = 009, 011,
012, 014 y 015 (control y gestión); P3 = 016 y 017 (adopción y conveniencia).

**Reglas de la planificación**:

- Cada spec se implementa con su propio `plan.md` y `tasks.md`; ninguno se escribe por adelantado
  para no congelar decisiones antes de verificar el esquema real.
- 007 es la base: crea `stock_por_deposito`, `movimientos` y `productos.stock_actual` según la
  constitución (Principio V). Ninguna otra spec de stock puede implementarse antes.
- 013 convierte `/caja` en punto de venta; 004 sigue siendo su búsqueda.
- 015 extiende la cobertura de auditoría de 006 a las acciones de 007, 012 y 013.
- 017 es la única con un componente fuera del stack Rust/SvelteKit (servicio LangChain), tal como
  fija la constitución; no habilita escrituras directas a la base.

Quedan **fuera del roadmap** por no aportar valor al alcance actual: integración con balanzas o
impresoras de etiquetas, facturación electrónica, contabilidad, cuentas corrientes y multi-moneda.
Se reevalúan solo si aparece un requisito concreto (Principio I: YAGNI).

## Decisiones de la partición

- **`bad` no es un SKU inválido**: `bad` normaliza a `BAD`, que cumple el regex del contrato.
  El caso inválido de T032 se corrigió a `bad!`; es un error del texto original de la tarea,
  no un cambio de requisito.
- **FR-008a** queda entera en 001 porque describe el ciclo de vida de la contraseña temporal;
  002 la referencia para el alta.
- **FR-008b** nació del ajuste 2026-11 de 002 (elección manual/automática + visualización en
  pantalla): se le asignó subíndice `b` porque `FR-008a` ya estaba registrada a 001 y cada FR
  tiene un solo dueño. 001 puede repetir el texto como contexto de su FR-008a, pero el dueño
  del requisito de operativa de alta es 002.
- **Renumeración 2026-11-01**: los IDs que la actualización 2026-11 de 001 había asignado
  libremente (`FR-009`–`FR-012` y `SC-005`–`SC-007`) chocaban con dueños ya registrados de 003,
  004, 005 y 006. Se movieron a los siguientes libres del registro: `FR-070` (registro),
  `FR-071` (SSO config/ocultado), `FR-072` (vinculación), `FR-073` (rol SSO), `SC-026`
  (registro), `SC-027` (ocultado SSO), `SC-028` (credenciales en pantalla). Las specs dueñas
  originales no se tocaron.
- **Auditoría**: el spec original solo la menciona en US6 y en Supuestos; 006 la formaliza como
  FR-019 (derivado). La ampliación 2026-11-07 extiende su alcance a todas las escrituras de gestión
  de 001–005 (categorías, depósitos, ubicaciones y asignaciones), con SC-030.
