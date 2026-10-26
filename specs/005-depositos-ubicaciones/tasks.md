# Tasks — Depósitos y ubicaciones (005)

**Spec**: [spec.md](./spec.md) | **Plan**: [plan.md](./plan.md) | **Contratos**:
[contracts/api.md](./contracts/api.md) | **Creado**: 2026-10-26

**Tests**: sí. TDD en validadores puros; integración HTTP para jerarquía, dependencias y RBAC;
schemas/cliente en el frontend.

## Fase 1 — Modelos y validadores

- [X] T001 Crear `models/deposito.rs`: `Deposito`, `Ubicacion`, `ListaUbicaciones`, constantes de
  tipos y validadores con `Result<_, AppError>`. **Aceptación**: `cargo check`.
- [X] T002 RED/`GREEN`: `tests/unit/test_deposito_validacion.rs` — longitudes, `trim`, dirección
  vacía → `None`, enum de tipo y mapeo de errores 409/422. **Aceptación**: 5/5 en verde.

## Fase 2 — Servicio y esquema

- [X] T003 Confirmar que `0003_depositos_ubicaciones.sql` cubre el modelo (sin migración nueva) y
  registrar la decisión en `research.md`/`data-model.md`.
- [X] T004 Implementar `services/depositos.rs`: CRUD de depósitos con transacción y mapeo de
  `23505`; borrado con conteo de dependencias.
- [X] T005 Implementar ubicaciones: alta con depósito/padre activo del mismo depósito, cambio de
  depósito sin dependencias, detección de ciclos con CTE recursiva, borrado con dependencias.
- [X] T006 Implementar asignaciones: `ubicaciones_de_producto` y `reemplazar_ubicaciones_producto`
  (deduplicación, validación de activos, reemplazo transaccional).
- [X] T007 Agregar variantes y mapeos en `error.rs`
  (`deposito_nombre_duplicado`, `deposito_identificador_duplicado`, `ubicacion_duplicada`,
  `ubicacion_huerfana`, `ubicacion_con_dependencias`).

## Fase 3 — HTTP y contrato

- [X] T008 Handlers de depósitos (`/api/depositos`, `/{id}`, `/{id}/ubicaciones`,
  `/{id}/desactivar`, `/{id}/reactivar`) con `RequiereGestion` y `utoipa`.
- [X] T009 Handlers de ubicaciones (`/api/ubicaciones`, `/{id}`, `/{id}/desactivar`,
  `/{id}/reactivar`) con `RequiereGestion` y `utoipa`.
- [X] T010 Asignación en `handlers/productos.rs`: `GET/PUT /api/productos/{id}/ubicaciones`.
- [X] T011 Registrar rutas en `app.rs`, paths/schemas/tags en `docs.rs` y extender
  `test_openapi`. **Aceptación**: `test_openapi` 6/6.
- [X] T012 RED/GREEN: `tests/integration/test_depositos_http.rs` — CRUD, unicidad, borrado con
  dependencias, jerarquía, ciclo, cambio de depósito, asignación y RBAC/401. **Aceptación**: 6/6.

## Fase 4 — Frontend

- [X] T013 RED/GREEN: `schemas/depositos.ts` + tests (nullable, enum, `trim`, `padre_id: null`).
- [X] T014 RED/GREEN: `api/depositos.ts` + tests (query, métodos, body, errores 409).
- [X] T015 Crear `/admin/depositos/+page.svelte`: alta/edición de depósitos, árbol de ubicaciones
  con sangría, form de ubicación (tipo/padre), baja lógica, borrado y asignación por producto.
- [X] T016 Agregar el enlace `Depósitos` en la navegación GESTION.

## Fase 5 — Cierre

- [X] T017 Correr `cargo test` (111/111), `cargo sqlx prepare --check`, `npm test` (136/136),
  `npm run check` y `npm run build`.
- [X] T018 Verificar SC-005 por SQL (`deposito_id NOT NULL` + consulta de huérfanas = 0) y revisar
  los mensajes en español.
- [X] T019 Escribir `progress.md` y actualizar `specs/README.md` y `docs/api.md`.

## Trazabilidad

| Requisito | Tareas |
|---|---|
| FR-015 | T001–T004, T008, T013–T016 |
| FR-016 | T001, T005, T009, T012, T015 |
| FR-017 | T006, T010, T012, T015 |
| FR-018 | T004, T005, T007, T012 |
| SC-005 | T003, T005, T018 |
