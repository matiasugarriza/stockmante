# Tasks: 010-consulta-stock

**Input**: Design documents from `/specs/010-consulta-stock/`

**Prerequisites**: plan.md ✓, spec.md ✓

## Format: `[ID] [P?] [Story] Description`

## Path Conventions

- Frontend: `frontend/src/`

---

## Phase 1: Frontend

- [X] T001 [P] [US10] Create page `frontend/src/routes/admin/stock/+page.svelte`: listado paginado con filtros q/deposito/categoria/bajo_minimo, columnas stock global, mínimo, valorizado a costo (solo GESTION)
- [X] T002 [P] [US10] Add stock disponible panel to `frontend/src/routes/caja/+page.svelte` (stock por depósito de venta)
- [X] T003 [US10] Add navigation link "Stock" to `frontend/src/routes/+layout.svelte` (visible para GESTION)
- [X] T004 [US10] Run `npm test`, `npm run check`, `npm run build` in `frontend/`

---

## Dependencies & Execution Order

- T001, T002 paralelos (archivos distintos)
- T003 después de T001
- T004 al final

---

## Notes

- Backend ya implementado y testeado (handlers/stock.rs, services/stock.rs).
- No migración propia.
