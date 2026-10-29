# Tasks: 014-reportes-kardex

**Input**: Design documents from `/specs/014-reportes-kardex/`

**Prerequisites**: plan.md ✓, spec.md ✓

## Format: `[ID] [P?] [Story] Description`

## Path Conventions

- Backend: `backend/src/`, `backend/tests/`
- Frontend: `frontend/src/`

---

## Phase 1: Backend

- [ ] T001 [P] [US14] Create service `backend/src/services/reportes.rs` (kardex, valorización, resúmenes)
- [ ] T002 [US14] Create handler `backend/src/handlers/reportes.rs` + router + OpenAPI
- [ ] T003 [US14] Tests unit + integration
- [ ] T004 [US14] Run `cargo test`, `cargo sqlx prepare --check`

## Phase 2: Frontend

- [ ] T005 [P] [US14] Create `frontend/src/routes/admin/reportes/+page.svelte`
- [ ] T006 [US14] Schemas + API client
- [ ] T007 [US14] Run `npm test`, `npm run check`, `npm run build`

## Phase 3: Polish

- [ ] T008 Update `docs/api.md`

---

## Notes

- Sin migración; solo lectura. Depende de 012 y 013.