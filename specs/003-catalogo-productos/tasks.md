# Tasks: Catálogo de productos y categorías (003)

**Spec**: [spec.md](./spec.md) | **Plan**: [plan.md](./plan.md) | **Data model**: [data-model.md](./data-model.md) | **Creado**: 2026-10-26

**Input**: Design documents from `/specs/003-catalogo-productos/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/api.md, contracts/validation.md, quickstart.md

**Tests**: **Sí, obligatorios.** Principio IV de la constitución (TDD RED→GREEN, sin waiver desde 003) y
Quality Gates de `AGENTS.md` (`cargo test` en verde, tests de `#lib/api/*.ts`, `cargo sqlx prepare
--check` sin diferencias). No es una opción de este proyecto.

**Ruta de implementación**: inline. La unidad de revisión es el commit de cada tarea o grupo lógico,
con tests y documentación en el mismo commit (`work-unit-commits`).

**Estado**: plan v2 implementado, probado por el usuario y commiteado el 2026-10-26 en la rama
`feat/003-catalogo-productos`; **T001–T037 marcadas**, con los gates y las verificaciones del
quickstart ejecutados (evidencia en [progress.md](./progress.md)). La revisión nativa de RDD revisó
el commit de la migración (aprobado) y los slices posteriores quedaron dentro del presupuesto. El WIP
anterior vivía en la rama `wip/003-catalogo` (`941b0cf`) y **no se reutilizó**: describía `DELETE`
físico, `activo` dentro del `PUT` y la migración `0004`.

**Historias**: esta feature tiene **una sola** historia (US3, P2). Las fases 3, 4 y 5 son incrementos
de esa misma historia, ordenados por dependencia técnica y cada uno verificable por separado.

## Formato: `[ID] [P?] [Story] Descripción`

- **[P]**: puede ejecutarse en paralelo (archivo distinto, sin dependencia de una tarea incompleta).
- **[Story]**: `[US3]` en las fases de historia; las fases de Setup, Foundational y Polish no llevan.
- Cada descripción nombra el archivo exacto a crear o modificar.

## Path Conventions

Aplicación web (plan.md): `backend/src/`, `backend/tests/`, `frontend/src/`, `migrations/` en la raíz
del repositorio. Alias de imports del frontend: **`#lib`** (no `$lib`).

---

## Phase 1: Setup (Infraestructura compartida)

**Purpose**: dependencias, contrato de errores y verificación de esquema previa a cualquier escritura.

- [X] T001 Agregar `rust_decimal = { version = "1", features = ["serde-float"] }` a `[dependencies]` y la feature `rust_decimal` a `sqlx` en `backend/Cargo.toml`. **Aceptación**: `cargo build` compila; `Decimal` serializa `10.1` como `10.1` (número JSON, no string).
- [X] T002 Verificar el esquema real **antes** de migrar con el MCP de PostgreSQL o `sqlx-cli`: `\d categorias`, `\d productos` y `SELECT conname FROM pg_constraint WHERE conrelid IN ('categorias'::regclass,'productos'::regclass)`. Registrar en `specs/003-catalogo-productos/progress.md` los nombres reales de `productos_codigo_barras_key`, `categorias_nombre_key` y si ya existe `idx_productos_categoria`. **Aceptación**: nombres confirmados; la migración de T005 usa exactamente esos nombres.
- [X] T003 [P] Ajustar `backend/tests/unit/test_apperror.rs`: reemplazar la aserción de `AppError::ProductoConReferencias` por `AppError::CategoriaDuplicada` → `409` y `categoria_duplicada`. **Aceptación**: RED (la variante nueva todavía no existe).
- [X] T004 Modificar `backend/src/error.rs`: agregar la variante `CategoriaDuplicada` con mensaje `"Ya existe una categoría con ese nombre."` en `status()` (409), `codigo()` (`categoria_duplicada`) y `mensaje_publico()`; eliminar `ProductoConReferencias` y su brazo. **Aceptación**: T003 en verde; `cargo build` sin warnings de variante muerta.

**Checkpoint**: dependencias y contrato de errores listos.

---

## Phase 2: Foundational (Prerrequisitos bloqueantes)

**Purpose**: esquema, modelos y utilidades compartidas que TODAS las rutas del catálogo necesitan.

**⚠️ CRITICAL**: ninguna tarea de las fases 3–5 puede empezar antes de completar esta fase.

- [X] T005 Crear `migrations/0006_catalogo_descripcion_codigo_barras.sql` con exactamente cuatro sentencias: (1) `ALTER TABLE categorias ADD COLUMN descripcion VARCHAR(255) NULL;`; (2) `CREATE UNIQUE INDEX uq_categorias_nombre_normalizado ON categorias (lower(btrim(nombre)));`; (3) `ALTER TABLE productos DROP CONSTRAINT productos_codigo_barras_key; ALTER TABLE productos ADD CONSTRAINT productos_codigo_barras_key UNIQUE (codigo_barras);` (verificar el nombre real en T002 y confirmar que el resultado es `NULLS DISTINCT`); (4) `CREATE INDEX idx_productos_categoria ON productos (categoria_id);`. **No** se elimina `categorias_nombre_key` y **no** se reutiliza el número `0004`. **Aceptación**: `sqlx migrate run` aplica en `stockmante` y en una base vacía; `\d productos` muestra `UNIQUE (codigo_barras)` sin `NULLS NOT DISTINCT`.
- [X] T006 Crear `backend/src/models/producto.rs` y registrar `pub mod producto;` en `backend/src/models/mod.rs`. Debe contener: structs `Categoria { id: i32, nombre: String, descripcion: Option<String>, activo: bool }`, `CategoriaBreve { id: i32, nombre: String }`, `Producto { id, nombre, sku, codigo_barras, categoria: CategoriaBreve, unidad_medida, costo, precio_venta, stock_minimo, activo }` y `ListaProductos { items, total, pagina, por_pagina }`, más `ProductoFila` (fila plana con `categoria_id` + `categoria_nombre`, `sqlx::FromRow`) y su conversión a `Producto` — SQLx no mapea structs anidados. Funciones puras: `normalizar_sku(&str) -> String` (trim + mayúsculas), `validar_sku(&str) -> Result<String, AppError>` (3–30 caracteres, `^[A-Z0-9][A-Z0-9-]{2,29}$`), `normalizar_codigo_barras(Option<&str>) -> Option<String>` (trim; vacío/blanco → `None`), `validar_codigo_barras` (8–64 alfanuméricos), `normalizar_descripcion`, `normalizar_nombre_categoria` (trim), `validar_nombre_producto` (1–160 tras trim), `validar_unidad` (lista cerrada `UNIDAD`, `KG`, `METRO`, `CAJA`), `validar_monto(Decimal)` (≥ 0, `scale() <= 2`, `< 10_000_000_000`) y `validar_stock_minimo(i64)` (≥ 0). Constraints del modelo: `nombre VARCHAR(160)` 1–160; `sku VARCHAR(30)`; `codigo_barras VARCHAR(64)` NULL único `NULLS DISTINCT`; `unidad_medida VARCHAR(20)`; `costo`/`precio_venta NUMERIC(12,2)`; `stock_minimo INT DEFAULT 0`; `activo BOOLEAN NOT NULL DEFAULT TRUE`. **Aceptación**: `cargo build` compila; sin `unwrap`/`expect`/`panic!`.
- [X] T007 Mover `parsear_activo` desde `backend/src/handlers/usuarios.rs` a `backend/src/handlers/mod.rs` como `pub(crate) fn parsear_activo(valor: Option<&str>, por_defecto: Option<bool>) -> Result<Option<bool>, AppError>`; actualizar la llamada de `handlers/usuarios.rs` con `None` y conservar el mensaje 422 actual. **Aceptación**: usuarios pasa `None` (ausente = todas), catálogo pasará `Some(true)` (ausente = solo activos); `cargo build` compila.
- [X] T008 Agregar en `backend/tests/integration/test_usuarios_http.rs` el caso de regresión `GET /api/usuarios` **sin** `activo` que devuelve activos e inactivos. **Aceptación**: verde con el `parsear_activo` centralizado (protege el default de 002).
- [X] T009 Extender `backend/tests/unit/common/mod.rs` con helpers de catálogo: `sku_unico(etiqueta: &str) -> String` (deriva de `Uuid::new_v4()`), `codigo_barras_unico() -> String`, `nombre_categoria_unico(etiqueta: &str) -> String`, y helpers de sesión `sesion_gestion(&PgPool) -> String` / `sesion_operacion(&PgPool) -> String` (cookie) reutilizando `crear_usuario` y el login de `test_usuarios_http.rs`. **Aceptación**: `cargo test --test test_usuarios_http` sigue en verde; los helpers no truncan tablas.

**Checkpoint**: esquema migrado, modelos y helpers listos — los incrementos pueden empezar.

---

## Phase 3: User Story 3 — Categorías (Priority: P2) · Incremento 1

**Goal**: CRUD completo de categorías planas con descripción, estado, unicidad global sin distinguir
mayúsculas y baja lógica; sin borrado físico.

**Independent Test**: crear dos categorías, repetir el nombre con otra capitalización (409), editar
descripción a `""`, desactivar y reactivar (204 idempotente), y confirmar que `DELETE` responde 405
sin alterar la fila. Cubre CU-CAT-01…06 y la mitad de CU-RET-01.

### Tests (RED primero) ⚠️

- [X] T010 [P] [US3] Crear `backend/tests/unit/test_categoria_normalizacion.rs` y su `[[test]]` en `backend/Cargo.toml`: `normalizar_nombre_categoria("  Almacén  ") == "Almacén"`; nombre vacío o solo espacios → `Validacion`; 81 caracteres → `Validacion`; `descripcion = ""` o `"   "` → `None`; 256 caracteres → `Validacion`. **Aceptación**: RED.
- [X] T011 [P] [US3] Crear `backend/tests/integration/test_categorias_http.rs` y su `[[test]]` en `backend/Cargo.toml`: CU-CAT-01 (201/409 `categoria_duplicada` comparando `"  almacén  "`/422/403), CU-CAT-02 (`?activo` default solo activas, `all`, inválido → 422, orden por nombre), CU-CAT-03 (detalle 200 incluso inactiva, 404, 401), CU-CAT-04 (edición parcial, `descripcion: ""` → `null`, `{}` → 422), CU-CAT-05 y CU-CAT-06 (204 idempotente, sin cascada a productos, 404, 403). Usar `cookie` de sesión y `tower::ServiceExt::oneshot` como `test_usuarios_http.rs`. **Aceptación**: RED (rutas inexistentes → 404/405).

### Implementation

- [X] T012 [US3] Crear `backend/src/services/categorias.rs` y registrar `pub mod categorias;` en `backend/src/services/mod.rs`: `listar(pool, activo: Option<bool>)` (`ORDER BY nombre ASC`), `obtener`, `crear`, `actualizar`, `desactivar`, `reactivar`. Reglas: `activo` ausente = solo activas (vía `parsear_activo(..., Some(true))` en el handler); `PUT` con cuerpo vacío → `Validacion("Indicá al menos un campo para actualizar.")`; `UPDATE … RETURNING` con 0 filas → `RecursoNoEncontrado`; `desactivar`/`reactivar` idempotentes (`UPDATE … SET activo = $2 WHERE id = $1`); mapeo `23505` de `uq_categorias_nombre_normalizado` **y** `categorias_nombre_key` → `CategoriaDuplicada`; sin pre-chequeo de duplicados. **Aceptación**: T010 y T011 verdes para categorías.
- [X] T013 [US3] Crear `backend/src/handlers/categorias.rs` con `router()`: `GET /` (`AuthUsuario`), `POST /` (`RequiereGestion`), `GET /{id}` (`AuthUsuario`), `PUT /{id}` (`RequiereGestion`), `POST /{id}/desactivar` y `POST /{id}/reactivar` (`RequiereGestion`). DTOs: `CrearCategoria { nombre, descripcion: Option<String> }` y `ActualizarCategoria { nombre: Option<String>, descripcion: Option<Option<String>> }` (doble opción: omitido conserva, `""` limpia). Rechazar `activo` presente en el `PUT` con 422 (mensaje: "El estado se cambia con /desactivar o /reactivar"). Usar `JsonValido`, `PathValido`, `QueryValida` de `backend/src/extractores.rs`. **No** registrar `DELETE`. **Aceptación**: compila y responde los códigos del contrato.
- [X] T014 [US3] Nestear `handlers::categorias::router()` bajo `/categorias` dentro del grupo `autenticado` de `backend/src/app.rs` (después del gate de contraseña temporal). **Aceptación**: T011 en verde; `DELETE /api/categorias/{id}` → 405 con `Allow: GET, HEAD, PUT` y sin cuerpo.

**Checkpoint**: categorías completamente administrables por API — valor entregable sin productos.

---

## Phase 4: User Story 3 — Productos (Priority: P2) · Incremento 2

**Goal**: CRUD de productos con SKU normalizado único global, código de barras opcional único,
montos `NUMERIC(12,2)`, categoría activa al asignar, listado paginado y filtrable por texto, categoría
y estado, y baja lógica con reactivación.

**Independent Test**: crear un producto con `sku: " yerba-1k "` (→ `YERBA-1K`, `codigo_barras: null`),
repetirlo (409), filtrar por `q` y `categoria_id` (AND, orden estable), editar `precio_venta` solo,
desactivar (sale del listado, sigue en el detalle), reactivar y confirmar 405 en `DELETE`. Cubre
CU-PROD-01…06 y el resto de CU-RET-01.

### Tests (RED primero) ⚠️

- [X] T015 [P] [US3] Crear `backend/tests/unit/test_sku.rs` y su `[[test]]`: `sku-001` → `SKU-001`; `bad` → `BAD` (válido); `bad!`, `AB`, `-ABC` y 31 caracteres → `Validacion`; 3 y 30 caracteres válidos. **Aceptación**: RED.
- [X] T016 [P] [US3] Crear `backend/tests/unit/test_producto_validacion.rs` y su `[[test]]`: código de barras `""`/`"   "` → `None`, 7 y 65 caracteres → `Validacion`, `"7791234567890"` → `Some`; montos `-1`, `10.999`, `10_000_000_000` → `Validacion`, `1200.50` → válido; `stock_minimo` `-3` → `Validacion` (el `2.5` se rechaza en el handler al deserializar `i64` → 422); unidad `LITRO` → `Validacion`; nombre vacío y 161 caracteres → `Validacion`. **Aceptación**: RED.
- [X] T017 [P] [US3] Crear `backend/tests/integration/test_productos_http.rs` y su `[[test]]`: CU-PROD-01 (14 criterios: normalización, 409 `sku_duplicado` contra activo **e inactivo**, 409 `codigo_barras_duplicado`, dos `NULL` conviven, 422 de categoría inexistente/inactiva/montos/unidad/nombre, 403), CU-PROD-02 (`total`, `?activo`, `q` sobre nombre y SKU, `categoria_id` inexistente → 200 vacío, paginación y 422 de rangos, 403 OPERACION), CU-PROD-03 (detalle 200 activo e inactivo, 401, 404), CU-PROD-04 (parcial, `codigo_barras: ""` limpia y omitido conserva, 422 con `activo` y con `{}`, atomicidad, 200 sobre inactivo), CU-PROD-05 y CU-PROD-06 (204 idempotente, sin cascada, reactivar con categoría inactiva → 204). **Aceptación**: RED.
- [X] T018 [P] [US3] Crear `backend/tests/integration/test_catalogo_baja_logica.rs` y su `[[test]]` (CU-RET-01): `DELETE /api/productos/{id}` y `DELETE /api/categorias/{id}` → 405 con la fila intacta (`id`, `activo`, columnas); tras 50 bajas lógicas `SELECT count(*)` de `productos` y `categorias` no cambia y solo cambia `activo`; `?activo=all` devuelve los inactivos con sus identificadores originales; sin huecos en la numeración de `id`. **Aceptación**: RED.

### Implementation

- [X] T019 [US3] Crear `backend/src/services/productos.rs` y registrar `pub mod productos;` en `backend/src/services/mod.rs`: `listar(pool, q, categoria_id, activo, pagina, por_pagina)` con `ILIKE` sobre `nombre` y `sku` (escapando `%`/`_` del término), `categoria_id` como filtro (inexistente → lista vacía), `activo` ausente → `true`, `ORDER BY nombre ASC, id ASC`, `total` antes de paginar y `offset = (pagina - 1) * por_pagina` con aritmética verificada; `crear`, `obtener`, `actualizar`, `desactivar`, `reactivar`. Reglas: `crear`/`actualizar` corren en `pool.begin()` y validan la categoría **dentro de la transacción** (`SELECT activo FROM categorias WHERE id = $1`; inexistente o inactiva → 422 `Validacion` salvo conservar la categoría actual); `actualizar` usa `COALESCE` por campo y `CASE WHEN $n THEN $m ELSE codigo_barras END` para distinguir “limpiar” de “conservar”; `activo` en el `PUT` → 422; 0 filas → `RecursoNoEncontrado`; mapeo `23505` de `productos_sku_key` → `SkuDuplicado` y `productos_codigo_barras_key` → `CodigoBarrasDuplicado`; `23503` como red de seguridad; `actualizado_en = now()` en cada escritura. **Aceptación**: T015–T018 verdes.
- [X] T020 [US3] Crear `backend/src/handlers/productos.rs` con `router()`: `GET /` (`RequiereGestion`), `POST /` (`RequiereGestion`), `GET /{id}` (`AuthUsuario`), `PUT /{id}` (`RequiereGestion`), `POST /{id}/desactivar` y `POST /{id}/reactivar` (`RequiereGestion`). `ListarQuery { q: Option<String>, categoria_id: Option<String>, activo: Option<String>, pagina: Option<i64>, por_pagina: Option<i64> }` parseado con `QueryValida`; `parsear_activo(activo.as_deref(), Some(true))`. DTOs `CrearProducto` y `ActualizarProducto` con doble opción en `codigo_barras` y rechazo de `activo`. **No** registrar `DELETE` ni `/buscar` (004). **Aceptación**: compila y responde los códigos del contrato.
- [X] T021 [US3] Nestear `handlers::productos::router()` bajo `/productos` en el grupo `autenticado` de `backend/src/app.rs`. **Aceptación**: T017 y T018 en verde; `DELETE /api/productos/{id}` → 405 con `Allow: GET, HEAD, PUT`.

**Checkpoint**: API de catálogo completa (12 rutas útiles + los dos `DELETE` que responden 405), verificable con `quickstart.md` V6a/V6b.

---

## Phase 5: User Story 3 — Administración en el frontend (Priority: P2) · Incremento 3

**Goal**: pantallas de administración de categorías y productos que consumen la API tipada, con
validación Zod espejo y baja/reactivación (sin `DELETE`).

**Independent Test**: desde `/admin/categorias` y `/admin/productos`, crear, editar, filtrar,
desactivar y reactivar; los errores 409/422 se muestran por campo y ninguna pantalla ofrece borrado.

- [X] T022 [P] [US3] Crear `frontend/src/lib/schemas/categorias.ts` y `frontend/src/lib/schemas/categorias.test.ts` (Zod 4): `nombre` `trim().min(1).max(80)`; `descripcion` `trim().max(255).nullish()`; `CategoriaUpdateSchema` parcial con `refine` de al menos un campo y **sin** `activo`. **Aceptación**: vitest verde con casos válidos e inválidos.
- [X] T023 [P] [US3] Crear `frontend/src/lib/schemas/productos.ts` y `frontend/src/lib/schemas/productos.test.ts`: `sku` `trim().toUpperCase()` + regex `/^[A-Z0-9][A-Z0-9-]{2,29}$/`; `codigo_barras` `'' | /^[0-9A-Za-z]{8,64}$/` nullish; `unidad_medida` enum `UNIDAD|KG|METRO|CAJA`; `costo`/`precio_venta` `nonnegative().multipleOf(0.01)` y `< 10_000_000_000`; `stock_minimo` entero ≥ 0; `nombre` 1–160; update parcial sin `activo`. **Aceptación**: vitest verde.
- [X] T024 [P] [US3] Crear `frontend/src/lib/api/categorias.ts` y `categorias.test.ts` usando el helper `api` de `frontend/src/lib/api/client.ts`: `listar(filtroActivo)`, `obtener`, `crear`, `actualizar`, `desactivar`, `reactivar`. **Aceptación**: vitest cubre 200/204 y que un 409 propague `ApiError` con `codigo`.
- [X] T025 [P] [US3] Crear `frontend/src/lib/api/productos.ts` y `productos.test.ts`: `listar(filtros)` con `q`, `categoria_id`, `activo`, `pagina`, `por_pagina`; `obtener`, `crear`, `actualizar`, `desactivar`, `reactivar`. **Aceptación**: vitest cubre el query string (incluido `activo` ausente = default del backend) y `ApiError`.
- [X] T026 [US3] Crear `frontend/src/lib/components/categorias/CategoriaForm.svelte` y `CategoriaTabla.svelte` (Svelte 5 runes): formulario alta/edición con errores por campo, tabla con estado y acciones “Editar”, “Desactivar”, “Reactivar” (sin “Eliminar”). **Aceptación**: `npm run check` sin errores de tipos.
- [X] T027 [US3] Crear `frontend/src/lib/components/productos/ProductoForm.svelte`, `ProductoTabla.svelte` y `FiltrosCatalogo.svelte`: selector de categorías activas (mostrando la actual aunque esté inactiva al editar), campos de unidad y montos, tabla con paginación y filtros `q` + categoría + estado. **Aceptación**: `npm run check` en verde; sin `fetch` directo en componentes.
- [X] T028 [US3] Crear `frontend/src/routes/admin/categorias/+page.svelte`: carga el listado con `FiltroActivo` (default activas), alta/edición en un panel y confirmación de baja lógica que llama a `/desactivar`. **Aceptación**: la pantalla no invoca `DELETE` en ninguna ruta.
- [X] T029 [US3] Crear `frontend/src/routes/admin/productos/+page.svelte`: listado paginado con `FiltrosCatalogo`, alta/edición con `ProductoForm` y acciones de baja/reactivación; mensajes de `ApiError` (409 de SKU/código, 422 de validación) visibles. **Aceptación**: `npm run check` y `npm run build` en verde.
- [X] T030 [US3] Agregar los enlaces “Categorías” y “Productos” en `frontend/src/routes/+layout.svelte`, junto al de “Usuarios” (línea 38), visibles solo para sesión con familia GESTION. **Aceptación**: navegación funcional en la SPA.

**Checkpoint**: catálogo administrable de punta a punta desde la UI.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: gates de cierre, documentación y consistencia entre artefactos.

- [X] T031 Ejecutar `cd backend && cargo test` (todos los targets, incluidos los cuatro nuevos) y corregir cualquier fallo. **Aceptación**: 100% en verde contra `stockmante_test`.
- [X] T032 Ejecutar `cd backend && cargo sqlx prepare --check` y regenerar `.sqlx` si hay diferencias. **Aceptación**: sin diferencias; `.sqlx` versionado en el mismo commit que las queries.
- [X] T033 Ejecutar `cd frontend && npm test && npm run check && npm run build`. **Aceptación**: los tres en verde.
- [X] T034 Verificar que no haya `unwrap()`, `expect()` ni `panic!` en rutas de producción del backend: revisar `backend/src/models/producto.rs`, `backend/src/services/categorias.rs`, `backend/src/services/productos.rs`, `backend/src/handlers/categorias.rs` y `backend/src/handlers/productos.rs`. **Aceptación**: sin coincidencias fuera de `#[cfg(test)]`.
- [X] T035 Ejecutar los escenarios de [quickstart.md](./quickstart.md) V6a y V6b (incluidos los pasos 9–15 de productos, el 405 y los invariantes finales) contra un entorno local. Registrar el resultado en `specs/003-catalogo-productos/progress.md`. **Aceptación**: todos los pasos observados; cualquier desvío documentado con su causa.
- [X] T036 Actualizar `specs/003-catalogo-productos/progress.md` (estado final: plan v2 implementado, qué se deshecho y qué no) y `specs/README.md`: el ledger de migraciones reserva `0005` a 004 y `0006` a 006, pero `0005_sesion_unica.sql` ya existe y esta feature usa `0006`. Corregir ambas reservas y el estado de 003. **Aceptación**: ambos documentos coinciden con `migrations/` y con `data-model.md`.
- [X] T037 Pasada de consistencia final: marcar las casillas de este archivo solo con evidencia observada; comparar `contracts/api.md` y `contracts/validation.md` contra lo implementado y corregir el documento (no el contrato) ante cualquier divergencia. **Aceptación**: cero contradicciones entre `spec.md`, `plan.md`, `data-model.md`, `contracts/` y el código.

---

## Trazabilidad casos de uso → tareas

| Caso de uso | Tareas |
|---|---|
| CU-CAT-01 Crear categoría | T010–T014 |
| CU-CAT-02 Listar/filtrar categorías | T010–T014 |
| CU-CAT-03 Ver detalle de categoría | T012–T014 |
| CU-CAT-04 Editar categoría | T010, T012–T014 |
| CU-CAT-05 Desactivar categoría | T012–T014 |
| CU-CAT-06 Reactivar categoría | T012–T014 |
| CU-PROD-01 Crear producto | T015–T021 |
| CU-PROD-02 Listar/filtrar productos | T015–T021 |
| CU-PROD-03 Ver detalle de producto | T019–T021 |
| CU-PROD-04 Editar producto | T016, T019–T021 |
| CU-PROD-05 Dar de baja un producto | T017–T021 |
| CU-PROD-06 Reactivar producto | T017–T021 |
| CU-RET-01 Retiro sin borrado físico | T011, T014, T018, T021 |
| FR-009 / FR-010 / FR-012 / FR-014 | T005–T021 |
| FR-020 (filtros del listado) | T017, T019–T021, T027, T029 |
| FR-021 (reactivación) | T012–T014, T019–T021, T026, T029 |
| FR-022 (contrato de inactividad) | **fuera de 003**: lo implementa el módulo de stock; acá solo se documenta |
| FR-023 (categoría activa al asignar) | T016, T019, T020 |
| SC-006 / SC-009 / SC-010 / SC-011 | T015–T018, T031 |
| SC-012 (latencia con 5.000 productos) | T035 (medición manual) |

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Fase 1)**: sin dependencias. T003 antes de T004 (RED → GREEN).
- **Foundational (Fase 2)**: depende de Setup. T005 después de T002 (nombres reales verificados) y de T001 (dependencias). **Bloquea las fases 3–5.**
- **US3 · Categorías (Fase 3)**: depende de Foundational. T010/T011 (RED) antes de T012–T014.
- **US3 · Productos (Fase 4)**: depende de la Fase 3 (los tests crean categorías por API y necesitan `parsear_activo` y el patrón de handler ya resuelto). T015–T018 (RED) antes de T019–T021.
- **US3 · Frontend (Fase 5)**: T022–T025 dependen solo de Foundational (son Zod y clientes tipados); T026–T030 dependen de la API completa (Fase 4).
- **Polish (Fase 6)**: depende de todas las fases anteriores.

### Within Each User Story Increment

1. Tests RED primero; confirmar el fallo antes de implementar.
2. Modelos y funciones puras antes de servicios.
3. Servicios antes de handlers.
4. Handlers antes de registrar el router.
5. Registro del router antes de dar por verde la integración.
6. Commit por tarea o grupo lógico, con tests y documentación incluidos.

### Parallel Opportunities

- T003 en paralelo con T002 (archivos distintos).
- T010 y T011 en paralelo (test unitario y de integración).
- T015, T016, T017 y T018 en paralelo (cuatro archivos de test distintos).
- T022, T023, T024 y T025 en paralelo (schemas y clientes del frontend).
- T026 y T027 pueden ir en paralelo (directorios distintos) si hay dos personas; el resto es secuencial.
- **No** paralelizar T012 y T013, ni T019 y T020: el handler consume el servicio y comparten archivos de registro (`services/mod.rs`, `handlers/mod.rs`).

### Parallel Example: Productos (Fase 4)

```text
# Escribir los cuatro tests a la vez (deben fallar):
Task: "backend/tests/unit/test_sku.rs"
Task: "backend/tests/unit/test_producto_validacion.rs"
Task: "backend/tests/integration/test_productos_http.rs"
Task: "backend/tests/integration/test_catalogo_baja_logica.rs"

# Después, en secuencia:
Task: "backend/src/services/productos.rs"
Task: "backend/src/handlers/productos.rs"
Task: "backend/src/app.rs"
```

---

## Implementation Strategy

### MVP funcional (API de catálogo)

1. Fase 1: Setup.
2. Fase 2: Foundational (bloqueante).
3. Fase 3: Categorías → **parar y validar** con V6a del quickstart.
4. Fase 4: Productos → **parar y validar** con V6b del quickstart.

Con las fases 1–4 el catálogo queda completo y verificable por HTTP; es el mínimo que desbloquea 004
y 005.

### Incremental Delivery

1. Setup + Foundational → esquema y contrato de errores listos.
2. Categorías → valor administrativo independiente (alta, edición, baja lógica).
3. Productos → catálogo maestro completo con filtros y baja lógica.
4. Frontend → incremento demostrable para el usuario final.
5. Polish → gates y documentación.

### Notas de alcance

- **No** se implementa `GET /api/productos/buscar` (004), ni ubicaciones (005), ni el rechazo de
  movimientos sobre productos inactivos (FR-022, módulo de stock futuro).
- **No** se registra ninguna ruta `DELETE`: el 405 lo emite Axum y no lleva cuerpo JSON.
- `activo` nunca viaja por `PUT`: se cambia solo con `/desactivar` y `/reactivar`, idempotentes.
- Los tests derivan SKU, código de barras y nombre de categoría de un UUID para correr en paralelo.
