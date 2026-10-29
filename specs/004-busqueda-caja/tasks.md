# Tasks â€” BÃºsqueda y caja (004)

**Spec**: [spec.md](./spec.md) | **Plan**: [plan.md](./plan.md) | **Contratos**:
[contracts/api.md](./contracts/api.md) | **Creado**: 2026-10-26

**Tests**: sÃ­. TDD en validaciÃ³n del servicio; integraciÃ³n HTTP para RBAC y orden; tests de
schema/cliente en el frontend.

## Fase 1 â€” Setup y datos

- [X] T001 Crear `migrations/0007_busqueda_trgm.sql` (`pg_trgm` + GIN sobre `productos.nombre`)
  y aplicarla en `stockmante` y `stockmante_test`. **AceptaciÃ³n**: `\d productos` muestra el
  Ã­ndice; `SELECT similarity('yerba','yerva')` funciona.
- [X] T002 Agregar `ListaBusqueda { items: Vec<Producto> }` en `models/producto.rs` con
  `ToSchema`. **AceptaciÃ³n**: `cargo check` compila.
- [X] T003 RED: crear `backend/tests/unit/test_busqueda.rs` con parsing de `campo` (5 valores +
  invÃ¡lido), longitudes mÃ­nimas, `limite` y `categoria_id` invÃ¡lidos. **AceptaciÃ³n**: falla antes
  de implementar.

## Fase 2 â€” Servicio

- [X] T004 Implementar `CampoBusqueda::parsear` y `BusquedaProductos` en
  `services/productos.rs`. **AceptaciÃ³n**: T003 en verde.
- [X] T005 Implementar `buscar()` con una rama SQL estÃ¡tica por campo, escapado de `LIKE`,
  filtro de activos/categorÃ­a y lÃ­mite. **AceptaciÃ³n**: `cargo check`.
- [X] T006 RED/GREEN: `backend/tests/integration/test_busqueda_http.rs` â€” SKU exacto, cÃ³digo
  exacto, nombre con orden, categorÃ­a, `q` vacÃ­o + categorÃ­a, inactivos excluidos,
  `incluir_inactivos` GESTION/OPERACION, 401 y 422. **AceptaciÃ³n**: verde.

## Fase 3 â€” Handler y contrato

- [X] T007 Registrar `GET /api/productos/buscar` en `handlers/productos.rs` (antes de `/{id}`),
  con `AuthUsuario`, validaciÃ³n de rol para inactivos y `#[utoipa::path]`.
- [X] T008 Agregar path y schema a `docs.rs`; extender `test_openapi` para la ruta nueva.
  **AceptaciÃ³n**: `/api-docs/openapi.json` la lista y el test pasa.

## Fase 4 â€” Frontend

- [X] T009 RED: tests de `ListaBusquedaSchema` y de `buscarProductos` (query params y parseo).
- [X] T010 Implementar schema `ListaBusquedaSchema` y `buscarProductos()` en
  `#lib/api/productos.ts`. **AceptaciÃ³n**: T009 verde.
- [X] T011 Crear `/caja/+page.svelte`: buscador con debounce 250 ms, selector de campo, filtro
  por categorÃ­a, checkbox de inactivos solo GESTION, tabla con precio/stock mÃ­nimo/estado y
  mensaje de sin resultados.
- [X] T012 Agregar el enlace `Caja` en `+layout.svelte` para ambas familias; los enlaces de
  administraciÃ³n siguen solo para GESTION.
- [X] T013 Ajustar `+page.svelte` para orientar a Caja/AdministraciÃ³n segÃºn el rol.

## Fase 5 â€” Cierre

- [X] T014 Ejecutar `cargo test`, `cargo sqlx prepare --check`, `npm test`, `npm run check` y
  `npm run build`; registrar resultados.
- [X] T015 Correr los pasos de `quickstart.md` contra el backend real y registrar evidencia
  (incluido el tiempo de SC-002).
- [X] T016 Actualizar `specs/README.md` (estado de 004) y `docs/api.md` si el contrato final
  difiere; escribir `progress.md`.

## Trazabilidad

| Requisito | Tareas |
|---|---|
| FR-011 | T001â€“T012 |
| FR-013 | T005, T006, T011 |
| SC-002 | T001, T005, T015 |

