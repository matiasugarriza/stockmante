# Tasks: 012-proveedores-compras

**Input**: Design documents from `/specs/012-proveedores-compras/`

**Prerequisites**: plan.md ✓, spec.md ✓

## Format: `[ID] [P?] [Story] Description`

## Path Conventions

- Backend: `backend/src/`, `backend/tests/`, `backend/migrations/`
- Frontend: `frontend/src/`

---

## Phase 1: Setup

- [X] T001 Create migration `backend/migrations/0013_proveedores_compras.sql` (proveedores, ordenes_compra, orden_compra_lineas, recepciones)
- [X] T002 Apply migration to both databases

## Phase 2: Backend

- [X] T003 [P] [US12] Create models `backend/src/models/proveedor.rs`, `compra.rs` with validation
- [X] T004 [US12] Implement services `proveedores.rs`, `compras.rs` (CRUD + recepción transaccional)
- [X] T005 [US12] Create handlers `proveedores.rs`, `compras.rs` + router registration + OpenAPI
- [X] T006 [US12] Tests unit + integration
- [X] T007 [US12] Run `cargo test`, `cargo sqlx prepare --check`

## Phase 3: Frontend

- [X] T008 [P] [US12] Create `frontend/src/routes/admin/proveedores/+page.svelte`
- [X] T009 [P] [US12] Create `frontend/src/routes/admin/compras/+page.svelte`
- [X] T010 [US12] Schemas + API clients
- [X] T011 [US12] Run `npm test`, `npm run check`, `npm run build`

## Phase 4: Polish

- [X] T012 Update `docs/api.md`
- [X] T013 OpenAPI verify

---

## Phase 5: Frontend completo y RBAC (2026-11-06)

Cierra la brecha detectada: T010 estaba marcado pero los clientes/schemas no existían,
las páginas eran solo listados y los handlers no exigían GESTION.

- [X] T014 [US12] RBAC: `handlers/proveedores.rs` y `handlers/compras.rs` usan `RequiereGestion` (403 `sin_permisos` para OPERACION)
- [X] T015 [US12] Schemas Zod `frontend/src/lib/schemas/proveedores.ts` y `compras.ts` + tests
- [X] T016 [US12] API clients `frontend/src/lib/api/proveedores.ts` y `compras.ts` + tests
- [X] T017 [US12] Componentes `ProveedorForm.svelte` y `ProveedorTabla.svelte`
- [X] T018 [US12] Componentes `CompraForm.svelte` (alta/edición con líneas) y `CompraTabla.svelte`
- [X] T019 [US12] Páginas `admin/proveedores` y `admin/compras`: alta/edición, confirmar/anular/eliminar borrador, estado vacío y errores 401/403/parseo
- [X] T020 [US12] Tests backend: matriz RBAC ampliada + `test_compras_http.rs` (flujo completo y 403 OPERACION)
- [X] T021 [US12] Gates: `cargo test`, `cargo sqlx prepare --check`, `npm test`, `npm run check`, `npm run build`
- [X] T022 [US12] Cerrar advisories de la verificación independiente: validación de líneas en PUT de compras (vacías/cantidad negativa/producto repetido), unicidad de CUIT en edición (`409 cuit_duplicado`) y guard de producto repetido en `CompraForm`
- [X] T023 [US12] Tests de regresión para 12.1, 12.5, 12.6, FR-046 y validaciones de edición (A1-A4)

---

## Notes

- Implementación pesada; se recomienda por etapas (proveedores primero, luego compras).
