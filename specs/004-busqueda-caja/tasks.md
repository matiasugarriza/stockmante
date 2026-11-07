# Tasks — Búsqueda y caja (004)

**Spec**: [spec.md](./spec.md) | **Plan**: [plan.md](./plan.md) | **Contratos**:
[contracts/api.md](./contracts/api.md) | **Creado**: 2026-10-26

**Tests**: sí. TDD en validación del servicio; integración HTTP para RBAC y orden; tests de
schema/cliente en el frontend.

## Fase 1 — Setup y datos

- [X] T000 (registro retroactivo 2026-11-07) Verificar el esquema real de `productos` y
  `categorias` contra `stockmante` y `stockmante_test`.
  **Evidencia**: 17 columnas revisadas; `productos.codigo_barras` y `categorias.descripcion`
  nullable, resto NOT NULL, coincide con `data-model.md`; `idx_productos_nombre_trgm` es
  `GIN (nombre gin_trgm_ops)`; `pg_trgm` presente. `stockmante_test` validado por T006.
  **Aceptación**: evidencia registrada en `progress.md`.
- [X] T001 Crear `migrations/0007_busqueda_trgm.sql` (`pg_trgm` + GIN sobre `productos.nombre`)
  y aplicarla en `stockmante` y `stockmante_test`. **Aceptación**: `\d productos` muestra el
  índice; `SELECT similarity('yerba','yerva')` funciona.
- [X] T002 Agregar `ListaBusqueda { items: Vec<Producto> }` en `models/producto.rs` con
  `ToSchema`. **Aceptación**: `cargo check` compila.
- [X] T003 RED: crear `backend/tests/unit/test_busqueda.rs` con parsing de `campo` (5 valores +
  inválido), longitudes mínimas, `limite` y `categoria_id` inválidos. **Aceptación**: falla antes
  de implementar.

## Fase 2 — Servicio

- [X] T004 Implementar `CampoBusqueda::parsear` y `BusquedaProductos` en
  `services/productos.rs`. **Aceptación**: T003 en verde.
- [X] T005 Implementar `buscar()` con una rama SQL estática por campo, escapado de `LIKE`,
  filtro de activos/categoría y límite. **Aceptación**: `cargo check`.
- [X] T006 RED/GREEN: `backend/tests/integration/test_busqueda_http.rs` — SKU exacto, código
  exacto, nombre con orden, categoría, `q` vacío + categoría, inactivos excluidos,
  `incluir_inactivos` GESTION/OPERACION, 401 y 422. **Aceptación**: verde.

## Fase 3 — Handler y contrato

- [X] T007 Registrar `GET /api/productos/buscar` en `handlers/productos.rs` (antes de `/{id}`),
  con `AuthUsuario`, validación de rol para inactivos y `#[utoipa::path]`.
- [X] T008 Agregar path y schema a `docs.rs`; extender `test_openapi` para la ruta nueva.
  **Aceptación**: `/api-docs/openapi.json` la lista y el test pasa.

## Fase 4 — Frontend

- [X] T009 RED: tests de `ListaBusquedaSchema` y de `buscarProductos` (query params y parseo).
- [X] T010 Implementar schema `ListaBusquedaSchema` y `buscarProductos()` en
  `#lib/api/productos.ts`. **Aceptación**: T009 verde.
- [X] T011 Crear `/caja/+page.svelte`: buscador con debounce 250 ms, selector de campo, filtro
  por categoría, checkbox de inactivos solo GESTION, tabla con precio/stock mínimo/estado y
  mensaje de sin resultados.
- [X] T012 Agregar el enlace `Caja` en `+layout.svelte` para ambas familias; los enlaces de
  administración siguen solo para GESTION.
- [X] T013 Ajustar `+page.svelte` para orientar a Caja/Administración según el rol.

## Fase 5 — Cierre

- [X] T014 Ejecutar `cargo test`, `cargo sqlx prepare --check` (OK, sin diferencias;
  2026-11-07), `npm test`, `npm run check` y `npm run build`; registrar resultados.
- [X] T015 Correr los pasos de `quickstart.md` contra el backend real y registrar evidencia
  (incluido el tiempo de SC-002 para SKU/código exacto y nombre/categoría como peor caso).
- [X] T016 Actualizar `specs/README.md` (estado de 004) y `docs/api.md` si el contrato final
  difiere; escribir `progress.md`.

## Trazabilidad

| Requisito | Tareas |
|---|---|
| Constitución V (verificación de esquema) | T000 |
| FR-011 | T001–T012 |
| FR-013 | T005, T006, T011 |
| SC-002 | T001, T005, T015 |
