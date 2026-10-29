# Tasks: 008-transferencias-depositos

**Input**: Design documents from `/specs/008-transferencias-depositos/`

**Prerequisites**: plan.md ✓, spec.md ✓, research.md ✓, data-model.md ✓, contracts/api.md ✓

**Tests**: Sí — la spec (escenarios 8.1–8.5), plan.md y la Constitución (Principio IV) los exigen.

**Organization**: Una sola historia de usuario (US8, P1). Tareas agrupadas por capa.

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

- [X] T001 Create migration `backend/migrations/0010_transferencias.sql` that creates table `transferencias` (id SERIAL PK, producto_id INTEGER NOT NULL FK→productos(id), deposito_origen_id INTEGER NOT NULL FK→depositos(id), deposito_destino_id INTEGER NOT NULL FK→depositos(id), cantidad INTEGER NOT NULL CHECK (cantidad > 0), ubicacion_origen_id INTEGER NULL FK→ubicaciones(id), ubicacion_destino_id INTEGER NULL FK→ubicaciones(id), movimiento_salida_id INTEGER NOT NULL UNIQUE FK→movimientos(id), movimiento_entrada_id INTEGER NOT NULL UNIQUE FK→movimientos(id), usuario_id INTEGER NOT NULL FK→usuarios(id), observacion TEXT NULL CHECK (length(observacion) <= 500), creado_en TIMESTAMPTZ NOT NULL DEFAULT now(), CHECK (deposito_origen_id <> deposito_destino_id)); also ALTER TABLE movimientos ADD COLUMN transferencia_id INTEGER NULL REFERENCES transferencias(id)

- [X] T002 Apply migration to both databases and verify: run `cargo sqlx migrate run --source ../migrations` against `stockmante` and `stockmante_test`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Error de dominio y constantes compartidas — bloquean US8

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [X] T003 Add variant `TransferenciaMismoDeposito(String)` to enum `AppError` in `backend/src/error.rs` with `status()` → 422 and `codigo()` → `"transferencia_mismo_deposito"`; wire it into the `match` arms for `status()`, `codigo()`, and `Display`

- [X] T004 Add constant `pub const TABLA_TRANSFERENCIAS: &str = "transferencias";` and `pub const MOTIVO_TRANSFERENCIA: &str = "TRANSFERENCIA";` to `backend/src/services/auditoria.rs`

- [X] T005 [P] Create test helper `backend/tests/common/mod.rs` function `crear_transferencia_de_prueba` (or extend existing helpers) that seeds: an active product with stock in deposito origen, an active deposito destino, and returns the IDs needed by transfer tests

**Checkpoint**: Foundation ready — user story implementation can now begin

---

## Phase 3: User Story 8 - Mover stock entre depósitos sin perder unidades (Priority: P1) 🎯 MVP

**Goal**: Registrar transferencias atómicas entre depósitos y consultarlas

**Independent Test**: POST /api/transferencias devuelve 201 con stock correcto en origen/destino; GET /api/transferencias/{id} muestra los dos movimientos; suma global de stock constante (SC-016)

### Tests for User Story 8 ⚠️

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [X] T006 [P] [US8] Unit tests for pure validation rules in `backend/tests/unit/test_transferencia_validacion.rs`: origen=destino → 422 `transferencia_mismo_deposito`, cantidad<=0 → 422, producto_id<=0 → 422, observacion length>500 → 422, missing required field → 422

- [X] T007 [P] [US8] Integration tests for POST /api/transferencias in `backend/tests/integration/test_transferencias_http.rs`: (a) 201 with correct stock_origen_resultante and stock_destino_resultante; (b) 409 stock_insuficiente with literal message `"Stock insuficiente en Principal: 3 disponibles, 5 solicitadas."` and no stock change; (c) 422 transferencia_mismo_deposito with literal message; (d) 422 validacion for inactive destination deposit; (e) 409 producto_inactivo; (f) SC-016: global stock sum identical before and after successful transfer

- [X] T008 [P] [US8] Integration tests for GET /api/transferencias and GET /api/transferencias/{id} in `backend/tests/integration/test_transferencias_http.rs`: list with filters producto_id/deposito_origen_id/deposito_destino_id, date range, pagination; detail returns both movements with correct deposito/motivo; 404 for nonexistent id

- [X] T009 [P] [US8] Integration tests for ubicacion origen/destino in `backend/tests/integration/test_transferencias_http.rs`: (a) with ubicacion_origen_id and ubicacion_destino_id → 201 and stock lands in correct ubicacion; (b) ubicacion belonging to wrong deposito → 422; (c) ubicacion not permitted for producto (producto_ubicacion_permitida) → 422

### Implementation for User Story 8

- [X] T010 [P] [US8] Create pure validation struct `TransferenciaNueva` and function `validar_transferencia` in `backend/src/models/transferencia.rs`: check deposito_origen_id != deposito_destino_id, cantidad > 0, producto_id > 0, deposito IDs > 0; return `Result<(), AppError>`

- [X] T011 [P] [US8] Add response structs to `backend/src/models/stock.rs`: `TransferenciaDetalle` (id, producto_id, producto_nombre, sku, deposito_origen: DepositoBreve, deposito_destino: DepositoBreve, cantidad, stock_origen_resultante: i32, stock_destino_resultante: i32, usuario: UsuarioBreve, creado_en: DateTime<Utc>, movimiento_salida_id: i32, movimiento_entrada_id: i32, observacion: Option<String>) with Serialize + ToSchema; `ListaTransferencias` (items, total, pagina, por_pagina); flat row structs `TransferenciaFila` and `TransferenciaLineaFila` with sqlx::FromRow

- [X] T012 [US8] Implement service `backend/src/services/transferencias.rs`: function `crear` that opens `pool.begin()`, locks depositos by id ASC FOR UPDATE, locks producto FOR UPDATE, locks stock_por_deposito by deposito_id ASC FOR UPDATE, validates stock sufficient in origin (reuse stock_insuficiente error), validates depositos active, validates ubicacion belongs to correct deposito and is permitted, decrements origin stock, increments destination stock, inserts movimiento SALIDA (deposito_id=origen, motivo=TRANSFERENCIA, transferencia_id=link), inserts movimiento ENTRADA (deposito_id=destino, motivo=TRANSFERENCIA), inserts one line each in movimiento_lineas, inserts transferencia row, recalculates productos.stock_actual = SUM(stock_por_deposito.cantidad), calls auditoria::registrar with TABLA_TRANSFERENCIAS, commits, returns TransferenciaDetalle

- [X] T013 [US8] Implement service functions `listar` and `obtener` in `backend/src/services/transferencias.rs`: `listar` with filters producto_id/deposito_origen_id/deposito_destino_id/desde/hasta/pagina/por_pagina (default 20, max 100); `obtener` by id returning TransferenciaDetalle or 404 recurso_no_encontrado

- [X] T014 [US8] Create handler `backend/src/handlers/transferencias.rs`: DTO `CrearTransferenciaDto` (producto_id, deposito_origen_id, deposito_destino_id, cantidad, ubicacion_origen_id: Option<i32>, ubicacion_destino_id: Option<i32>, observacion: Option<String>); POST handler requiring OPERACION or GESTION family via AuthUsuario; GET list handler; GET by id handler; parse fecha desde/hasta reusing the same ISO 8601 parser pattern from movimientos.rs; add utoipa::path annotations for all three endpoints

- [X] T015 [US8] Register router: add `pub mod transferencias;` to `backend/src/handlers/mod.rs`; add `.nest("/api/transferencias", handlers::transferencias::router())` to the authenticated router in `backend/src/app.rs`; add TransferenciaDetalle and ListaTransferencias to the OpenAPI schemas and the three paths to the OpenAPI paths in `backend/src/docs.rs`

- [X] T016 [US8] Run backend tests and make them pass: `cargo test` in `backend/`; fix any failures in models/services/handlers until all unit and integration tests for transferencias are green

- [X] T017 [P] [US8] Create Zod schema in `frontend/src/lib/schemas/transferencias.ts`: `TransferenciaSchema` (id, producto_id, producto_nombre, sku, deposito_origen: DepositoBreve, deposito_destino: DepositoBreve, cantidad, stock_origen_resultante, stock_destino_resultante, usuario: UsuarioBreve, creado_en: string, movimiento_salida_id, movimiento_entrada_id, observacion nullable) and `CrearTransferenciaInputSchema`; export types

- [X] T018 [P] [US8] Create typed API client in `frontend/src/lib/api/transferencias.ts`: `crearTransferencia(input: CrearTransferenciaInput): Promise<TransferenciaDetalle>`, `listarTransferencias(params): Promise<ListaTransferencias>`, `obtenerTransferencia(id: number): Promise<TransferenciaDetalle>`; reuse the existing fetch helper pattern from `frontend/src/lib/api/` modules

- [X] T019 [US8] Create page `frontend/src/routes/transferencias/+page.svelte`: form to create transfer (producto selector, deposito origen selector, deposito destino selector, cantidad input, ubicacion origen/destino optional selectors, observacion textarea); on submit calls crearTransferencia and shows result with stock resultante in both depositos (Principio III); list view with filters (producto, deposito origen, deposito destino, fechas) and pagination; detail modal or expanded row showing both movements

- [X] T020 [US8] Add navigation link "Transferencias" to `frontend/src/routes/+layout.svelte` sidebar/menu, visible for OPERACION and GESTION families

- [X] T021 [US8] Run frontend tests and checks: `npm test`, `npm run check`, `npm run build` in `frontend/`; add `transferencias.test.ts` covering Zod schema parsing and API client mock responses

**Checkpoint**: User Story 8 fully functional and testable independently

---

## Phase 4: Polish & Cross-Cutting Concerns

**Purpose**: Quality gates, docs, and final verification

- [X] T022 [P] Update API documentation table in `docs/api.md` with the three new routes (POST /api/transferencias, GET /api/transferencias, GET /api/transferencias/{id}) including required permissions and error codes

- [X] T023 Run full quality gates: `cargo test`, `cargo sqlx prepare --check` in `backend/`; `npm test`, `npm run check`, `npm run build` in `frontend/`; fix any regressions

- [X] T024 Update OpenAPI spec in `backend/src/docs.rs` if any schema or path was missed; verify `cargo test test_openapi` passes

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion — BLOCKS US8
- **User Story 8 (Phase 3)**: Depends on Foundational phase completion
- **Polish (Phase 4)**: Depends on US8 completion

### Within User Story 8

- Tests (T006–T009) MUST be written and FAIL before implementation
- Models (T010–T011) before services (T012–T013)
- Services before handlers (T014)
- Handler + router (T014–T015) before making tests pass (T016)
- Frontend tasks (T017–T021) can run after backend is green (T016)

### Parallel Opportunities

- T006, T007, T008, T009 — all test files, different scenarios, no dependencies between them
- T010 and T011 — model file and stock.rs structs, different files
- T017 and T018 — frontend schema and API client, different files
- T022 — docs can be updated in parallel with T023 verification

---

## Parallel Example: User Story 8

```bash
# Launch all tests for US8 together (after T005):
Task: "Unit tests for transferencia validation in backend/tests/unit/test_transferencia_validacion.rs"
Task: "Integration tests for POST /api/transferencias in backend/tests/integration/test_transferencias_http.rs"
Task: "Integration tests for GET /api/transferencias in backend/tests/integration/test_transferencias_http.rs"
Task: "Integration tests for ubicacion in backend/tests/integration/test_transferencias_http.rs"

# Launch model tasks together (after tests written):
Task: "Create TransferenciaNueva validation in backend/src/models/transferencia.rs"
Task: "Add TransferenciaDetalle structs to backend/src/models/stock.rs"

# Launch frontend tasks together (after backend green):
Task: "Create Zod schema in frontend/src/lib/schemas/transferencias.ts"
Task: "Create API client in frontend/src/lib/api/transferencias.ts"
```

---

## Implementation Strategy

### MVP First (User Story 8 Only)

1. Complete Phase 1: Setup (migration)
2. Complete Phase 2: Foundational (errors, constants)
3. Complete Phase 3: User Story 8 — tests first, then models, service, handler, frontend
4. **STOP and VALIDATE**: Run quickstart.md scenarios 1–5 manually or via integration tests
5. Complete Phase 4: Polish

### Incremental Delivery

1. Migration + Foundation → schema ready
2. US8 backend (T010–T016) → API functional, tests green
3. US8 frontend (T017–T021) → UI functional
4. Polish (T022–T024) → quality gates green, docs updated

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- Verify tests fail before implementing (T006–T009 should RED before T010–T015)
