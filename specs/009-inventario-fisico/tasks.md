# Tasks: 009-inventario-fisico

**Input**: Design documents from `/specs/009-inventario-fisico/`

**Prerequisites**: plan.md ✓, spec.md ✓, research.md ✓, data-model.md ✓, contracts/api.md ✓

**Tests**: Sí — la spec (escenarios 9.1–9.7), plan.md y la Constitución (Principio IV) los exigen.

**Organization**: Una sola historia de usuario (US9, P2). Tareas agrupadas por capa.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

- Backend: `backend/src/`, `backend/tests/`, `backend/migrations/`
- Frontend: `frontend/src/`
- Docs: `docs/`

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Migración de base de datos

- [X] T001 Create migration `backend/migrations/0011_inventario_fisico.sql` that creates tables `conteos` (id SERIAL PK, deposito_id INTEGER NOT NULL FK→depositos(id), estado VARCHAR(10) NOT NULL CHECK (estado IN ('ABIERTO','CERRADO','CANCELADO')), usuario_id INTEGER NOT NULL FK→usuarios(id), creado_en TIMESTAMPTZ NOT NULL DEFAULT now(), cerrado_en TIMESTAMPTZ NULL, cancelado_en TIMESTAMPTZ NULL, movimiento_ajuste_id INTEGER NULL FK→movimientos(id), observacion TEXT NULL CHECK (observacion IS NULL OR length(observacion) <= 500)) and `conteo_lineas` (id SERIAL PK, conteo_id INTEGER NOT NULL FK→conteos(id) ON DELETE CASCADE, producto_id INTEGER NOT NULL FK→productos(id), ubicacion_id INTEGER NULL FK→ubicaciones(id), cantidad_sistema INTEGER NOT NULL CHECK (cantidad_sistema >= 0), cantidad_contada INTEGER NULL CHECK (cantidad_contada IS NULL OR cantidad_contada >= 0), diferencia INTEGER NULL, UNIQUE NULLS NOT DISTINCT (conteo_id, producto_id, ubicacion_id))

- [X] T002 Apply migration to both databases and verify: run `cargo sqlx migrate run --source ../migrations` against `stockmante` and `stockmante_test`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Error de dominio y constantes compartidas — bloquean US9

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [X] T003 Add variants `ConteoCerrado(String)`, `ConteoCancelado(String)`, `ConteoNoEncontrado` to enum `AppError` in `backend/src/error.rs` with status() → 409/409/404 and codigo() → `"conteo_cerrado"` / `"conteo_cancelado"` / `"conteo_no_encontrado"`; wire into match arms for status(), codigo() and Display

- [X] T004 Add constant `pub const TABLA_CONTEOS: &str = "conteos";` to `backend/src/services/auditoria.rs`

- [X] T005 [P] Create test helper `crear_conteo_de_prueba` in `backend/tests/unit/common/mod.rs` (or extend) that seeds an active deposito with stock and returns IDs

**Checkpoint**: Foundation ready — user story implementation can now begin

---

## Phase 3: User Story 9 - Contar el inventario y ajustar diferencias (Priority: P2) 🎯 MVP

**Goal**: Abrir, cargar, cerrar y cancelar conteos con ajuste transaccional

**Independent Test**: POST /api/conteos → 201 con líneas; PATCH líneas → 200 con diferencias; POST cerrar → 200 con AJUSTE generado y stock = contado (SC-017); OPERACION → 403

### Tests for User Story 9 ⚠️

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [X] T006 [P] [US9] Unit tests for pure validation rules in `backend/tests/unit/test_conteo_validacion.rs`: cantidad_contada negativa → 422, deposito_id inválido → 422, estado inválido → 422, observacion > 500 → 422

- [X] T007 [P] [US9] Integration tests for POST /api/conteos and GET in `backend/tests/integration/test_conteos_http.rs`: 201 con líneas del stock actual; GET lista con filtros; GET detalle; 404 conteo_no_encontrado; OPERACION → 403

- [X] T008 [P] [US9] Integration tests for PATCH /api/conteos/{id}/lineas and cierre in `backend/tests/integration/test_conteos_http.rs`: cargar cantidades → 200 con diferencias; cerrar con diferencias → 200 + AJUSTE con líneas por diff≠0 + stock=contado (SC-017); cerrar sin diferencias → sin movimiento; cerrar cerrado → 409 conteo_cerrado; líneas sin contar → 422 lineas_sin_contar

- [X] T009 [P] [US9] Integration tests for cancelación y RBAC in `backend/tests/integration/test_conteos_http.rs`: cancelar → 200 CANCELADO sin ajuste; OPERACION → 403 en toda la superficie; depósito inactivo → 422

### Implementation for User Story 9

- [X] T010 [P] [US9] Create pure validation struct `ConteoNuevo` and function `validar_conteo` in `backend/src/models/conteo.rs`: validar deposito_id > 0, observacion <= 500, cantidad_contada >= 0; return Result

- [X] T011 [P] [US9] Add response structs to `backend/src/models/stock.rs`: `ConteoDetalle` (id, deposito_id, deposito_nombre, estado, usuario, creado_en, cerrado_en, cancelado_en, movimiento_ajuste_id, observacion, lineas: Vec<ConteoLinea>), `ConteoLinea` (id, producto_id, producto_nombre, sku, ubicacion_id, ubicacion_nombre, cantidad_sistema, cantidad_contada, diferencia), `ListaConteos` (items, total, pagina, por_pagina); flat row structs `ConteoFila`, `ConteoLineaFila` with sqlx::FromRow

- [X] T012 [US9] Implement service `backend/src/services/conteos.rs`: `crear` (abre conteo, captura líneas de stock_por_deposito del depósito, audita), `cargar_lineas` (actualiza cantidad_contada, recalcula diferencia contra cantidad_sistema vigente), `cerrar` (transacción: bloquea conteo FOR UPDATE, valida estado ABIERTO y líneas contadas, recalcula cantidad_sistema vigente, genera AJUSTE si hay diff, aplica deltas, recalcula cache, audita), `cancelar` (bloquea, pasa a CANCELADO, audita), `listar`, `obtener`

- [X] T013 [US9] Create handler `backend/src/handlers/conteos.rs`: DTOs `CrearConteoDto`, `CargarLineasDto`, `LineaContadaDto`; handlers POST/GET/PATCH/POST cerrar/POST cancelar; parsear fechas; utoipa::path annotations; exigir GESTION via AuthUsuario + familia check

- [X] T014 [US9] Register router: add `pub mod conteos;` to `backend/src/handlers/mod.rs`; add `.nest("/api/conteos", handlers::conteos::router())` to authenticated router in `backend/src/app.rs`; add schemas and paths to `backend/src/docs.rs`

- [X] T015 [US9] Run backend tests and make them pass: `cargo test`; fix until all unit and integration tests for conteos are green

- [X] T016 [P] [US9] Create Zod schema in `frontend/src/lib/schemas/conteos.ts`: `ConteoSchema`, `ConteoLineaSchema`, `ListaConteosSchema`, `CrearConteoSchema`, `CargarLineasSchema`; export types

- [X] T017 [P] [US9] Create typed API client in `frontend/src/lib/api/conteos.ts`: `crearConteo`, `listarConteos`, `obtenerConteo`, `cargarLineas`, `cerrarConteo`, `cancelarConteo`

- [X] T018 [US9] Create page `frontend/src/routes/conteos/+page.svelte`: abrir conteo por depósito, tabla de líneas con input cantidad_contada, mostrar diferencia, botones cerrar/cancelar, listado con filtros

- [X] T019 [US9] Add navigation link "Conteos" to `frontend/src/routes/+layout.svelte` (visible para GESTION)

- [X] T020 [US9] Run frontend tests and checks: `npm test`, `npm run check`, `npm run build`; add `conteos.test.ts`

**Checkpoint**: User Story 9 fully functional and testable independently

---

## Phase 4: Polish & Cross-Cutting Concerns

**Purpose**: Quality gates, docs, and final verification

- [X] T021 [P] Update API documentation table in `docs/api.md` with the six new routes

- [X] T022 Run full quality gates: `cargo test`, `cargo sqlx prepare --check` in `backend/`; `npm test`, `npm run check`, `npm run build` in `frontend/`

- [X] T023 Update OpenAPI spec in `backend/src/docs.rs` if any schema or path was missed; verify `cargo test test_openapi` passes

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion — BLOCKS US9
- **User Story 9 (Phase 3)**: Depends on Foundational phase completion
- **Polish (Phase 4)**: Depends on US9 completion

### Within User Story 9

- Tests (T006–T009) MUST be written and FAIL before implementation
- Models (T010–T011) before services (T012)
- Service before handlers (T013)
- Handler + router (T013–T014) before making tests pass (T015)
- Frontend tasks (T016–T020) can run after backend is green (T015)

### Parallel Opportunities

- T006, T007, T008, T009 — all test files, different scenarios
- T010 and T011 — model file and stock.rs structs
- T016 and T017 — frontend schema and API client
- T021 — docs in parallel with T022

---

## Parallel Example: User Story 9

```bash
# Launch all tests for US9 together (after T005):
Task: "Unit tests for conteo validation in backend/tests/unit/test_conteo_validacion.rs"
Task: "Integration tests for POST/GET /api/conteos in backend/tests/integration/test_conteos_http.rs"
Task: "Integration tests for PATCH/cerrar in backend/tests/integration/test_conteos_http.rs"
Task: "Integration tests for cancelar/RBAC in backend/tests/integration/test_conteos_http.rs"

# Launch model tasks together (after tests written):
Task: "Create ConteoNuevo validation in backend/src/models/conteo.rs"
Task: "Add ConteoDetalle structs to backend/src/models/stock.rs"

# Launch frontend tasks together (after backend green):
Task: "Create Zod schema in frontend/src/lib/schemas/conteos.ts"
Task: "Create API client in frontend/src/lib/api/conteos.ts"
```

---

## Implementation Strategy

### MVP First (User Story 9 Only)

1. Complete Phase 1: Setup (migration)
2. Complete Phase 2: Foundational (errors, constants)
3. Complete Phase 3: User Story 9 — tests first, then models, service, handler, frontend
4. **STOP and VALIDATE**: Run quickstart.md scenarios 1–7
5. Complete Phase 4: Polish

### Incremental Delivery

1. Migration + Foundation → schema ready
2. US9 backend (T010–T015) → API functional, tests green
3. US9 frontend (T016–T020) → UI functional
4. Polish (T021–T023) → quality gates green, docs updated

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- Verify tests fail before implementing (T006–T009 should RED before T010–T015)
