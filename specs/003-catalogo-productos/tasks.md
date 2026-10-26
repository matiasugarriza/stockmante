# Tasks: Catálogo de productos y categorías (003)

**Spec**: [spec.md](./spec.md) | **Plan**: [plan.md](./plan.md) | **Creado**: 2026-10-25

**Input**: Design documents from `/specs/003-catalogo-productos/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/api.md, contracts/validation.md

**Tests**: Sí — Principio IV (TDD RED→GREEN) y quality gates.

**Ruta**: implementación **inline** (sin escritores delegados); la unidad de revisión es el
commit de cada tarea.

**Estado**: WIP de backend deshecho el 2026-10-25; replanteo pendiente. El código, los tests y la
migración viven solo en la rama `wip/003-catalogo` (`941b0cf`).

## Trazabilidad con la spec original

| Original | Actual |
|---|---|
| T032 | T001 |
| T033 | T002 |
| T034 | T003 |
| T035 | T004 |
| T036 | T005 |

---

## Phase 1: Catálogo de productos y categorías (US3, P2)

**Goal**: CRUD productos con SKU normalizado, categorías, baja lógica si hay referencias,
precios solo gestión.

**Independent Test**: Crear producto TEST-001 OK; SKU repetido → 409; SKU mal formado → 422;
DELETE con asignación → 409; desactivar sale de caja (búsqueda en 004).

### Tests FIRST (RED)

- [ ] T001 [P] [US3] Unit test: `normalizar_sku` + validación regex (`sku-001`→`SKU-001`; `bad!` → `Validacion`) en `backend/tests/unit/test_sku.rs`. **Aceptación**: RED. *(WIP deshecho el 2026-10-25: el test existe solo en `wip/003-catalogo` (`941b0cf`); a replantear.)*
- [ ] T002 [P] [US3] Unit test: SKU duplicado y código de barras duplicado → 409 en `backend/tests/unit/test_producto_unicidad.rs`. **Aceptación**: RED. *(WIP deshecho el 2026-10-25: el test existe solo en `wip/003-catalogo` (`941b0cf`); a replantear.)*

### Implementation

- [ ] T003 [US3] Implementar servicio `backend/src/services/productos.rs` + `backend/src/services/categorias.rs`: CRUD, normalización SKU, mapeo unique violation → `SkuDuplicado`/`CodigoBarrasDuplicado`/`CategoriaDuplicada`, DELETE solo sin referencias else `ProductoConReferencias`/`CategoriaConProductos`, desactivar; migración `0004_productos_codigo_barras_null_distinct.sql`. **Aceptación**: T001–T002 verdes. *(WIP deshecho el 2026-10-25: servicios, modelo, variantes de `AppError` y migración están solo en `wip/003-catalogo` (`941b0cf`). La constraint `UNIQUE (codigo_barras)` (`NULLS DISTINCT`) quedó aplicada a mano en `stockmante` y `stockmante_test`, sin registro en `_sqlx_migrations`; el replanteo decide cómo formalizarla.)*
- [ ] T004 [US3] Implementar handlers `backend/src/handlers/productos.rs` y `categorias.rs`: escritura y `GET /productos` con `RequiereGestion`; `GET /productos/{id}` y `GET /categorias` con `AuthUsuario`; mover `parsear_activo` a `handlers/mod.rs`; registrar routers en `app.rs`; agregar `[[test]] test_productos_http` en `Cargo.toml`. **Aceptación**: `backend/tests/integration/test_productos_http.rs` en verde (CRUD, 409 de unicidad y referencias, RBAC, 401, 404, 422). *(Estado: tests RED escritos; el target aún no compila porque faltan los handlers.)*
- [ ] T005 [P] [US3] Frontend: schemas producto/categoría, cliente API, rutas `admin/productos` y `admin/categorias` (sin tilde), formularios con Zod. **Aceptación**: vitest de schemas y clientes; UI alta/edición/baja con mensajes `ApiError`.

**Checkpoint**: catálogo administrable.

---

## Gates de cierre de la spec

- [ ] `cargo test` (todos los targets, incluido `test_productos_http`).
- [ ] `cargo sqlx prepare --check`.
- [ ] `npm test`, `npm run check`, `npm run build`.
- [ ] Grep sin `unwrap`/`panic!`/`expect` en rutas de producción del backend.
- [ ] Actualizar [progress.md](./progress.md) y el estado en [specs/README.md](../README.md).

## Notes

- `parsear_activo` se comparte con usuarios; no duplicar la semántica de `true|false|all`.
- El borrado físico de producto se bloquea por asignaciones de ubicación; el fallback `23503`
  cubre referencias futuras.
- El código de barras vacío en `PUT` limpia el campo; omitirlo lo conserva.
- Los tests usan SKU/código derivados de UUID para correr en paralelo.
