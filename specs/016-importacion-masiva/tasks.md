# Tasks: 016-importacion-masiva

**Input**: Design documents from `/specs/016-importacion-masiva/`

**Prerequisites**: plan.md ✓, spec.md ✓

## Format: `[ID] [P?] [Story] Description`

## Path Conventions

- Backend: `backend/src/`, `backend/tests/`
- Frontend: `frontend/src/`

---

## Phase 1: Backend

- [ ] T001 [US16] Create service `backend/src/services/importacion.rs` (parse CSV, validar, crear productos + stock inicial como movimiento)
- [ ] T002 [US16] Create handler `backend/src/handlers/importacion.rs` (multipart) + router + OpenAPI
- [ ] T003 [US16] Tests unit + integration
- [ ] T004 [US16] Run `cargo test`, `cargo sqlx prepare --check`

## Phase 2: Frontend

- [ ] T005 [P] [US16] Create `frontend/src/routes/admin/importacion/+page.svelte` (upload + reporte)
- [ ] T006 [US16] Run `npm test`, `npm run check`, `npm run build`

## Phase 3: Polish

- [ ] T007 Update `docs/api.md`