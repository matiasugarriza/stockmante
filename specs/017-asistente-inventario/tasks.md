# Tasks: 017-asistente-inventario

**Input**: Design documents from `/specs/017-asistente-inventario/`

**Prerequisites**: plan.md ✓, spec.md ✓

## Format: `[ID] [P?] [Story] Description`

## Path Conventions

- Backend: `backend/src/`, `backend/tests/`
- Frontend: `frontend/src/`

---

## Phase 1: Backend

- [ ] T001 [US17] Create service `backend/src/services/asistente.rs` (cliente LLM + tool dispatch)
- [ ] T002 [US17] Create handler `backend/src/handlers/asistente.rs` + router + OpenAPI
- [ ] T003 [US17] Tests (mock LLM)
- [ ] T004 [US17] Run `cargo test`, `cargo sqlx prepare --check`

## Phase 2: Frontend

- [ ] T005 [P] [US17] Create `frontend/src/routes/asistente/+page.svelte` (chat widget)
- [ ] T006 [US17] Run `npm test`, `npm run check`, `npm run build`

## Phase 3: Polish

- [ ] T007 Update `docs/api.md`