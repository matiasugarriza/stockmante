# Tasks: 011-alertas-stock-minimo

**Input**: Design documents from `/specs/011-alertas-stock-minimo/`

**Prerequisites**: plan.md ✓, spec.md ✓

## Format: `[ID] [P?] [Story] Description`

## Path Conventions

- Frontend: `frontend/src/`

---

## Phase 1: Frontend

- [ ] T001 [P] [US11] Add "Bajo mínimo" highlighted section to `frontend/src/routes/admin/stock/+page.svelte` (filter bajo_minimo=true, mostrar diferencia y mínimo)
- [ ] T002 [P] [US11] Add alert badge to `frontend/src/routes/+layout.svelte` (conteo de productos bajo mínimo, visible para GESTION)
- [ ] T003 [US11] Run `npm test`, `npm run check`, `npm run build` in `frontend/`

---

## Notes

- Backend ya soporta `GET /api/stock?bajo_minimo=true` (010).
- Sin migración propia.