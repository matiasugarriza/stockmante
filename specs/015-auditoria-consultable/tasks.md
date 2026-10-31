# Tasks: 015-auditoria-consultable

**Input**: Design documents from `/specs/015-auditoria-consultable/`

**Prerequisites**: plan.md ✓, spec.md ✓

## Format: `[ID] [P?] [Story] Description`

## Path Conventions

- Backend: `backend/src/`, `backend/tests/`
- Frontend: `frontend/src/`

---

## Phase 1: Backend

- [X] T001 [US15] Add `listar` to `backend/src/services/auditoria.rs` (filtros tabla_ref, usuario_id, desde, hasta, paginación)
- [X] T002 [US15] Create handler `backend/src/handlers/auditoria.rs` + router + OpenAPI
- [X] T003 [US15] Tests unit + integration
- [X] T004 [US15] Run `cargo test`, `cargo sqlx prepare --check`

## Phase 2: Frontend

- [X] T005 [P] [US15] Create `frontend/src/routes/admin/auditoria/+page.svelte`
- [X] T006 [US15] Schemas + API client
- [X] T007 [US15] Run `npm test`, `npm run check`, `npm run build`

## Phase 3: Polish

- [X] T008 Update `docs/api.md`
