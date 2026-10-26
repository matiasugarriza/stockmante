# Tasks — RBAC transversal y auditoría (006)

**Spec**: [spec.md](./spec.md) | **Plan**: [plan.md](./plan.md) | **Contratos**:
[contracts/api.md](./contracts/api.md) | **Creado**: 2026-10-26

**Tests**: sí. Matriz HTTP transversal + verificación SQL de auditoría + guard estructural.

## Fase 1 — Auditoría

- [X] T001 Crear `migrations/0008_auditoria.sql` con `auditoria(id, tabla_ref, registro_id,
  accion, usuario_id, cuando)` e índices; aplicar en `stockmante` y `stockmante_test`.
  **Aceptación**: migración 8 aplicada en ambas bases.
- [X] T002 Crear `services/auditoria.rs` con constantes de tabla/acción y
  `registrar(&mut Transaction, ...)`.
- [X] T003 Instrumentar usuarios: alta, edición, baja, reactivación y contraseña temporal dentro
  de la transacción; el actor viaja como `actor_id`.
- [X] T004 Instrumentar productos: alta, edición (incluye precio), baja y reactivación con el
  mismo patrón transaccional.
- [X] T005 Pasar el actor desde `RequiereGestion(pub Usuario)` en los handlers de usuarios y
  productos.

## Fase 2 — Verificación

- [X] T006 Agregar `actor_de_test` en `tests/unit/common/mod.rs` y actualizar las llamadas
  directas a los servicios en `test_usuarios_email`, `test_usuarios_alta` y `test_ultimo_admin`.
- [X] T007 RED/GREEN: `tests/integration/test_rbac_matriz.rs` con 4 tests: matriz completa
  (OPERACION → 403, GESTION → no 401/403), auditoría de productos, auditoría de usuarios y 403
  que no audita. **Aceptación**: 4/4.
- [X] T008 Registrar `test_rbac_matriz` en `Cargo.toml`.

## Fase 3 — Frontend

- [X] T009 Crear `frontend/src/routes/admin/+layout.svelte`: OPERACION → `/caja`; solo GESTION
  renderiza las páginas de administración.

## Fase 4 — Cierre

- [X] T010 Correr `cargo test` (115/115), `npm test` (136/136), `npm run check` (0/0).
- [X] T011 Escribir artefactos (`plan.md`, `research.md`, `data-model.md`,
  `contracts/api.md`, `quickstart.md`, `tasks.md`, `progress.md`) y actualizar la spec
  (`0006` → `0008`), `specs/README.md` y `.specify/feature.json`.

## Trazabilidad

| Requisito | Tareas |
|---|---|
| FR-003 (verificación) | T007 |
| FR-019 (auditoría) | T001–T005, T007 |
| SC-007 | T007 (matriz de `DELETE`), tests de 005 |
| SC-008 | T007 |
| US6 escenario 3 | T003–T005, T007 |
| Caso borde 403 sin auditoría | T007 |
