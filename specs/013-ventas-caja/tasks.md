# Tasks: 013-ventas-caja

**Input**: Design documents from `/specs/013-ventas-caja/`

**Prerequisites**: plan.md ✓, spec.md ✓

## Format: `[ID] [P?] [Story] Description`

## Path Conventions

- Backend: `backend/src/`, `backend/tests/`, `backend/migrations/`
- Frontend: `frontend/src/`

---

## Phase 1: Setup

- [ ] T001 Create migration `backend/migrations/0014_ventas_caja.sql` (clientes, ventas, venta_lineas)
- [ ] T002 Apply migration to both databases

## Phase 2: Backend

- [ ] T003 [P] [US13] Create models `cliente.rs`, `venta.rs` with validation
- [ ] T004 [US13] Implement services `clientes.rs`, `ventas.rs` (CRUD + venta transaccional con salida de stock)
- [ ] T005 [US13] Create handlers `clientes.rs`, `ventas.rs` + router + OpenAPI
- [ ] T006 [US13] Tests unit + integration
- [ ] T007 [US13] Run `cargo test`, `cargo sqlx prepare --check`

## Phase 3: Frontend

- [ ] T008 [P] [US13] Update `frontend/src/routes/caja/+page.svelte` with cart + checkout
- [ ] T009 [P] [US13] Create `frontend/src/routes/admin/ventas/+page.svelte`
- [ ] T010 [P] [US13] Create `frontend/src/routes/admin/clientes/+page.svelte`
- [ ] T011 [US13] Schemas + API clients
- [ ] T012 [US13] Run `npm test`, `npm run check`, `npm run build`

## Phase 4: Polish

- [ ] T013 Update `docs/api.md`
- [ ] T014 OpenAPI verify

---

## Notes

- Alto riesgo: salida de stock transaccional; validar stock antes de confirmar.