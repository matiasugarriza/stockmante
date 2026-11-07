# Tasks: 008-transferencias-depositos

**Input**: Design documents from `/specs/008-transferencias-depositos/`

**Prerequisites**: plan.md ✓, spec.md ✓, research.md ✓, data-model.md ✓, contracts/api.md ✓

**Tests**: Sí — la spec (escenarios 8.1–8.5), plan.md y la Constitución (Principio IV) los exigen.

**Organization**: Una sola historia de usuario (US8, P1). Tareas agrupadas por capa.

**Revisión 2026-11-07** (`/speckit analyze`): T005–T009, T016, T023 y T024 estaban marcadas como
hechas sin que existieran sus archivos (helper, `test_transferencia_validacion.rs`,
`test_transferencias_http.rs`, rutas en `test_openapi.rs`), y por eso los quality gates no podían
estar en verde. Se reabren con la ruta corregida. La Fase 5 agrega la remediación del resto de
los hallazgos (plan.md, «Ampliación 2026-11-07»).

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

- Backend: `backend/src/`, `backend/tests/unit/`, `backend/tests/integration/` (cada test es un
  `[[test]]` en `backend/Cargo.toml`; helpers en `backend/tests/unit/common/mod.rs`)
- Migraciones: `migrations/` en la raíz del repo
- Frontend: `frontend/src/`
- Docs: `docs/`

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Migración de base de datos

- [X] T001 Create migration `migrations/0010_transferencias.sql` (implemented with `INT GENERATED ALWAYS AS IDENTITY` instead of `SERIAL`, same as 0009) that creates table `transferencias` (id PK, producto_id INTEGER NOT NULL FK→productos(id), deposito_origen_id INTEGER NOT NULL FK→depositos(id), deposito_destino_id INTEGER NOT NULL FK→depositos(id), cantidad INTEGER NOT NULL CHECK (cantidad > 0), ubicacion_origen_id INTEGER NULL FK→ubicaciones(id), ubicacion_destino_id INTEGER NULL FK→ubicaciones(id), movimiento_salida_id INTEGER NOT NULL UNIQUE FK→movimientos(id), movimiento_entrada_id INTEGER NOT NULL UNIQUE FK→movimientos(id), usuario_id INTEGER NOT NULL FK→usuarios(id), observacion TEXT NULL CHECK (length(observacion) <= 500), creado_en TIMESTAMPTZ NOT NULL DEFAULT now(), CHECK (deposito_origen_id <> deposito_destino_id)); also ALTER TABLE movimientos ADD COLUMN transferencia_id INTEGER NULL REFERENCES transferencias(id)

- [X] T002 Apply migration to both databases and verify: run `cargo sqlx migrate run --source ../migrations` against `stockmante` and `stockmante_test`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Error de dominio y constantes compartidas — bloquean US8

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [X] T003 Add variant `TransferenciaMismoDeposito(String)` to enum `AppError` in `backend/src/error.rs` with `status()` → 422 and `codigo()` → `"transferencia_mismo_deposito"`; wire it into the `match` arms for `status()`, `codigo()`, and `Display`

- [X] T004 Add constant `pub const TABLA_TRANSFERENCIAS: &str = "transferencias";` and `pub const MOTIVO_TRANSFERENCIA: &str = "TRANSFERENCIA";` to `backend/src/services/auditoria.rs`

- [X] T005 [P] *(reabierta 2026-11-07: el helper no existe)* Create test helper in `backend/tests/unit/common/mod.rs`, function `crear_transferencia_de_prueba` (or extend existing helpers), that seeds: an active product with stock in deposito origen, an active deposito destino, and returns the IDs needed by transfer tests

**Checkpoint**: Foundation ready — user story implementation can now begin

---

## Phase 3: User Story 8 - Mover stock entre depósitos sin perder unidades (Priority: P1) 🎯 MVP

**Goal**: Registrar transferencias atómicas entre depósitos y consultarlas

**Independent Test**: POST /api/transferencias devuelve 201 con stock correcto en origen/destino; GET /api/transferencias/{id} muestra los dos movimientos; suma global de stock constante (SC-016)

### Tests for User Story 8 ⚠️

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [X] T006 [P] [US8] *(reabierta: hoy solo hay tests inline en `models/transferencia.rs`)* Unit tests for pure validation rules in `backend/tests/unit/test_transferencia_validacion.rs`, through `stockmante::models::transferencia::validar_transferencia`: origen=destino → 422 `transferencia_mismo_deposito`, cantidad<=0 → 422, producto_id<=0 → 422, observacion length>500 → 422, missing required field → 422

- [X] T007 [US8] *(reabierta: el archivo no existe)* Integration tests for POST /api/transferencias in `backend/tests/integration/test_transferencias_http.rs`: (a) 201 with correct stock_origen_resultante and stock_destino_resultante; (b) 409 stock_insuficiente with literal message `"Stock insuficiente en Principal: 3 disponibles, 5 solicitadas."` and no stock change; (c) 422 transferencia_mismo_deposito with literal message; (d) 422 validacion for inactive destination deposit; (e) 409 producto_inactivo; (f) SC-016: global stock sum identical before and after successful transfer

- [X] T008 [US8] *(reabierta)* Integration tests for GET /api/transferencias and GET /api/transferencias/{id} in `backend/tests/integration/test_transferencias_http.rs`: list with filters producto_id/deposito_origen_id/deposito_destino_id, date range, pagination; detail returns `movimiento_salida_id` and `movimiento_entrada_id` (the full movements are checked in T030, plan decision 11); 404 for nonexistent id (message in T033)

- [X] T009 [US8] *(reabierta)* Integration tests for ubicacion origen/destino in `backend/tests/integration/test_transferencias_http.rs`: (a) with ubicacion_origen_id and ubicacion_destino_id → 201 and stock lands in correct ubicacion; (b) ubicacion belonging to wrong deposito → 422; (c) ubicacion not permitted for producto (producto_ubicacion_permitida) → 422

### Implementation for User Story 8

- [X] T010 [P] [US8] Create pure validation struct `TransferenciaNueva` and function `validar_transferencia` in `backend/src/models/transferencia.rs`: check deposito_origen_id != deposito_destino_id, cantidad > 0, producto_id > 0, deposito IDs > 0; return `Result<(), AppError>`

- [X] T011 [P] [US8] Add response structs to `backend/src/models/stock.rs`: `TransferenciaDetalle` (id, producto_id, producto_nombre, sku, deposito_origen: DepositoBreve, deposito_destino: DepositoBreve, cantidad, stock_origen_resultante: i32, stock_destino_resultante: i32, usuario: UsuarioBreve, creado_en: DateTime<Utc>, movimiento_salida_id: i32, movimiento_entrada_id: i32, observacion: Option<String>) with Serialize + ToSchema; `ListaTransferencias` (items, total, pagina, por_pagina); flat row structs `TransferenciaFila` and `TransferenciaLineaFila` with sqlx::FromRow

- [X] T012 [US8] Implement service `backend/src/services/transferencias.rs`: function `crear` that opens `pool.begin()`, locks depositos by id ASC FOR UPDATE, locks producto FOR UPDATE, locks stock_por_deposito by deposito_id ASC FOR UPDATE, validates stock sufficient in origin (reuse stock_insuficiente error), validates depositos active, validates ubicacion belongs to correct deposito and is permitted, decrements origin stock, increments destination stock, inserts movimiento SALIDA (deposito_id=origen, motivo=TRANSFERENCIA, transferencia_id=link), inserts movimiento ENTRADA (deposito_id=destino, motivo=TRANSFERENCIA), inserts one line each in movimiento_lineas, inserts transferencia row, recalculates productos.stock_actual = SUM(stock_por_deposito.cantidad), calls auditoria::registrar with TABLA_TRANSFERENCIAS, commits, returns TransferenciaDetalle

- [X] T013 [US8] Implement service functions `listar` and `obtener` in `backend/src/services/transferencias.rs`: `listar` with filters producto_id/deposito_origen_id/deposito_destino_id/desde/hasta/pagina/por_pagina (default 20, max 100); `obtener` by id returning TransferenciaDetalle or 404 recurso_no_encontrado

- [X] T014 [US8] Create handler `backend/src/handlers/transferencias.rs`: DTO `CrearTransferenciaDto` (producto_id, deposito_origen_id, deposito_destino_id, cantidad, ubicacion_origen_id: Option<i32>, ubicacion_destino_id: Option<i32>, observacion: Option<String>); POST handler requiring OPERACION or GESTION family via AuthUsuario; GET list handler; GET by id handler; parse fecha desde/hasta reusing the same ISO 8601 parser pattern from movimientos.rs; add utoipa::path annotations for all three endpoints

- [X] T015 [US8] Register router: add `pub mod transferencias;` to `backend/src/handlers/mod.rs`; add `.nest("/api/transferencias", handlers::transferencias::router())` to the authenticated router in `backend/src/app.rs`; add TransferenciaDetalle and ListaTransferencias to the OpenAPI schemas and the three paths to the OpenAPI paths in `backend/src/docs.rs`

- [X] T016 [US8] *(reabierta: depende de T006–T009)* Run backend tests and make them pass: `cargo test` in `backend/`; fix any failures in models/services/handlers until all unit and integration tests for transferencias are green

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

- [X] T023 *(reabierta; se cierra junto con T036)* Run full quality gates: `cargo test`, `cargo sqlx prepare --check` in `backend/`; `npm test`, `npm run check`, `npm run build` in `frontend/`; fix any regressions

- [X] T024 *(reabierta: `test_openapi.rs` no cubre las rutas nuevas, ver T032)* Update OpenAPI spec in `backend/src/docs.rs` if any schema or path was missed; verify `cargo test --test test_openapi` passes

---

## Phase 5: Ampliación 2026-11-07 (remediación de `/speckit analyze`)

**Purpose**: cerrar los hallazgos C1–C3, H1–H4, M1–M6 y L1–L3 (plan.md, decisiones 9–15).

**Tests**: la implementación ya existe, así que los tests de comportamiento ya cubierto pueden
quedar verdes en la primera corrida. No se fabrica un RED artificial: cada test se corre antes
de tocar el código y el resultado (RED o verde inmediato) se anota en `progress.md`. Los tests de
T028 (desborde) y T033 (mensaje 404) cubren comportamiento que cambia y DEBEN observarse en RED.

- [X] T025 Create `specs/008-transferencias-depositos/progress.md`: record that T006–T009 were marked done without files and without RED evidence, and propose waiver W-004 with `/speckit constitution` (scope: the T007–T009 tests that pass on the first run). **Aceptación**: `progress.md` exists and the waiver has been proposed.
- [X] T026 Register `[[test]]` entries `test_transferencia_validacion` (`tests/unit/test_transferencia_validacion.rs`) and `test_transferencias_http` (`tests/integration/test_transferencias_http.rs`) in `backend/Cargo.toml`. Prerequisite for T006–T009 and T027–T033.
- [X] T027 [US8] Integration test in `test_transferencias_http.rs`: a successful transfer writes exactly one `auditoria` row with `tabla_ref = 'transferencias'`, `accion = 'CREAR'`, `registro_id` = transfer id and `usuario_id` = session user (FR-033).
- [X] T028 [US8] Integration tests in `test_transferencias_http.rs`, plan decisions 9–10: (a) 20 units only in ubicación A-01 of origin, transfer without `ubicacion_origen_id` → 409 `stock_insuficiente` with "0 disponibles" and no change; (b) `stock_origen_resultante`/`stock_destino_resultante` equal `movimiento_lineas.stock_resultante` of each movement; (c) destination row at `i32::MAX` → 422 `validacion`, no writes (**RED**: today the sum is unchecked).
- [X] T029 [US8] Fix `backend/src/services/transferencias.rs`: replace both `.expect(...)` (lines 93 and 96) with `ok_or_else(|| AppError::Validacion(...))?`; compute `stock_destino_resultante` with `checked_add`, same as `services/movimientos.rs`. **Aceptación**: `grep -n "unwrap()\|expect(\|panic!"` over `services/transferencias.rs` and `handlers/transferencias.rs` returns nothing; T028(c) green.
- [X] T030 [US8] Integration test in `test_transferencias_http.rs` (escenario 8.5, decision 11): `GET /api/movimientos/{movimiento_salida_id}` returns `SALIDA` in origin and `GET /api/movimientos/{movimiento_entrada_id}` returns `ENTRADA` in destination, both with `motivo = 'TRANSFERENCIA'` and one line of ±cantidad.
- [X] T031 [US8] Integration test in `test_transferencias_http.rs` (SC-016 on failures): for stock insuficiente, inactive destination and same deposit, `SUM(stock_por_deposito.cantidad)` and `productos.stock_actual` do not change, and there are no new rows in `transferencias`, `movimientos` or `auditoria`.
- [X] T032 [P] Extend `backend/tests/unit/test_openapi.rs`: the three `/api/transferencias` paths and the `TransferenciaDetalle`/`ListaTransferencias` schemas appear in the generated OpenAPI document. Closes T024.
- [X] T033 [US8] 404 message: test (**RED**) in `test_transferencias_http.rs` expecting `"No existe la transferencia {id}. Revisá el número o buscala en el listado de transferencias."`; then update `obtener` in `services/transferencias.rs` (Principio III, contracts/api.md).
- [X] T034 *(hecha 2026-11-07: clarify confirmó las decisiones 9 y 11 sin cambios)* Run `/speckit clarify 008-transferencias-depositos` to confirm plan decisions 9 (stock per ubicación row) and 11 (movements by id), then update `spec.md`: status «Implementado; remediación 2026-11-07 en curso», a "Clarificaciones" section, and the "sin idempotencia" assumption (L1). Update the 008 row in `specs/README.md`. If clarify changes a decision, reopen T028/T030 and adjust the plan.
- [X] T035 [P] `docs/api.md`: in the transfers section, state that stock is validated and reported per ubicación row (decision 9) and that the detail references the movements by id (decision 11).
- [X] T036 Quality gates: `cargo test` (all binaries, including `test_transferencia_validacion` and `test_transferencias_http`) and `cargo sqlx prepare --check` in `backend/`; `npm test`, `npm run check`, `npm run build` in `frontend/`. Mark T016, T023 and T024 done only after this passes. Apply the `CARGO_TARGET_DIR` workaround if the backend is running.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion — BLOCKS US8
- **User Story 8 (Phase 3)**: Depends on Foundational phase completion
- **Polish (Phase 4)**: Depends on US8 completion
- **Ampliación (Phase 5)**: T025 → T026 + T005 → tests (T006–T009, T027, T028, T030, T031, T032, T033 test) → fixes (T029, T033 fix) → T034, T035 → T036 (closes T016, T023, T024)

### Within User Story 8

- Tests (T006–T009) MUST be written and FAIL before implementation
- Models (T010–T011) before services (T012–T013)
- Services before handlers (T014)
- Handler + router (T014–T015) before making tests pass (T016)
- Frontend tasks (T017–T021) can run after backend is green (T016)

### Parallel Opportunities

- T006 (unit file) in parallel with the integration tests; T007–T009, T027, T028, T030, T031 and T033 share `test_transferencias_http.rs`, so they run one after another
- T032 (`test_openapi.rs`) and T035 (`docs/api.md`) in parallel with any Phase 5 task
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
- Verify tests fail before implementing (T006–T009 should RED before T010–T015). Revisión 2026-11-07: ese RED no se observó; ver T025 y W-004.

## Trazabilidad

| Requisito / hallazgo | Tareas |
|---|---|
| FR-031 | T001, T007, T009, T010, T012, T028, T029 |
| FR-032 | T008, T013, T014, T030, T033 |
| FR-033 | T004, T012, T014, T027 |
| SC-016 | T007(f), T031 |
| C1 / Principio IV (tests y RED) | T005–T009, T025, T026 |
| C2 / Principio II (`expect`) | T029 |
| C3 / H4 (estado real) | este archivo, T034, T036 |
| H1 (stock por ubicación) | T028, T034, T035 |
| M5 (detalle de movimientos) | T030, T034, T035 |
| M1, M2, M3, M4, M6, L1–L3 | plan.md, data-model.md, research.md, contracts/api.md, T034 |
