# Tasks: 012-proveedores-compras

**Input**: Design documents from `/specs/012-proveedores-compras/`

**Prerequisites**: plan.md ✓, spec.md ✓

## Format: `[ID] [P?] [Story] Description`

## Path Conventions

- Backend: `backend/src/`, `backend/tests/`, `backend/migrations/`
- Frontend: `frontend/src/`

---

## Phase 1: Setup

- [ ] T001 Create migration `backend/migrations/0013_proveedores_compras.sql` (proveedores, ordenes_compra, orden_compra_lineas, recepciones)
- [ ] T002 Apply migration to both databases

## Phase 2: Backend

- [ ] T003 [P] [US12] Create models `backend/src/models/proveedor.rs`, `compra.rs` with validation
- [ ] T004 [US12] Implement services `proveedores.rs`, `compras.rs` (CRUD + recepción transaccional)
- [ ] T005 [US12] Create handlers `proveedores.rs`, `compras.rs` + router registration + OpenAPI
- [ ] T006 [US12] Tests unit + integration
- [ ] T007 [US12] Run `cargo test`, `cargo sqlx prepare --check`

## Phase 3: Frontend

- [ ] T008 [P] [US12] Create `frontend/src/routes/admin/proveedores/+page.svelte`
- [ ] T009 [P] [US12] Create `frontend/src/routes/admin/compras/+page.svelte`
- [ ] T010 [US12] Schemas + API clients
- [ ] T011 [US12] Run `npm test`, `npm run check`, `npm run build`

## Phase 4: Polish

- [ ] T012 Update `docs/api.md`
- [ ] T013 OpenAPI verify

---

## Notes

- Implementación pesada; se recomienda por etapas (proveedores primero, luego compras).