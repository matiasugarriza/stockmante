# Tasks: 009-inventario-fisico

**Input**: Design documents from `/specs/009-inventario-fisico/`

**Prerequisites**: plan.md ✓, spec.md ✓ (clarificada 2026-11-07), research.md ✓, data-model.md ✓, contracts/api.md ✓

**Tests**: Sí — escenarios 9.1–9.7, SC-017 y el Principio IV de la Constitución los exigen.
**Atención**: el `tasks.md` anterior marcaba T006–T009 (tests de backend) y T021 (docs) como
completadas sin que esos artefactos existieran. Aquí se **reabren**; lo verificado se marca `[X]`
y lo ausente queda `[ ]`.

**Organization**: Una sola historia de usuario (US9, P2). Tareas agrupadas por capa.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

- Backend: `backend/src/`, `backend/tests/`
- Migraciones: **`migrations/` en la raíz del repo** (no `backend/migrations/`)
- Frontend: `frontend/src/`
- Docs: `docs/`

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Esquema de base de datos

- [X] T001 Verify migration `migrations/0011_inventario_fisico.sql` creates `conteos` (id, deposito_id, estado CHECK IN ('ABIERTO','CERRADO','CANCELADO'), usuario_id, creado_en, cerrado_en, cancelado_en, movimiento_ajuste_id, observacion CHECK `length(observacion) <= 500`) and `conteo_lineas` (conteo_id ON DELETE CASCADE, producto_id, ubicacion_id, cantidad_sistema CHECK >= 0, cantidad_contada CHECK NULL OR >= 0, diferencia, UNIQUE NULLS NOT DISTINCT (conteo_id, producto_id, ubicacion_id) as `uq_conteo_linea_producto_ubicacion`) — matches `data-model.md`

- [ ] T002 Apply migration to both databases and verify: run `cargo sqlx migrate run --source ../migrations` against `stockmante` and `stockmante_test`, then confirm the tables exist with `\d conteos` and `\d conteo_lineas`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Error de dominio y helper de test — bloquean US9

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [X] T003 Verify existing `AppError` variants `ConteoCerrado`, `ConteoCancelado`, `ConteoNoEncontrado` in `backend/src/error.rs` with status 409/409/404 and `codigo()` `conteo_cerrado`/`conteo_cancelado`/`conteo_no_encontrado`

- [ ] T004 Add variants `LineasSinContar` and `DepositoInactivo` to enum `AppError` in `backend/src/error.rs`: `status()` → 422 for both, `codigo()` → `"lineas_sin_contar"` / `"deposito_inactivo"`, `Display` con mensajes accionables ("Hay líneas sin cantidad contada; cargalas antes de cerrar." / "El depósito está inactivo; activalo antes de abrir un conteo."), and wire every match arm (status, codigo, Display, From)

- [X] T005 Verify constant `pub const TABLA_CONTEOS: &str = "conteos";` in `backend/src/services/auditoria.rs`

- [ ] T006 Add helper `crear_conteo_de_prueba` to `backend/tests/unit/common/mod.rs` that seeds an active deposito with stock (`stock_por_deposito`) for a fresh producto and returns `(deposito_id, producto_id, ubicacion_id, cantidad)`; reuse `sku_unico`/`nombre_categoria_unico`

**Checkpoint**: Foundation ready — user story implementation can now begin

---

## Phase 3: User Story 9 - Contar el inventario y ajustar diferencias (Priority: P2) 🎯 MVP

**Goal**: Abrir, cargar, cerrar y cancelar conteos con ajuste transaccional

**Independent Test**: POST /api/conteos → 201 con líneas; PATCH líneas → 200 con diferencias; POST cerrar → 200 con un AJUSTE y stock = contado (SC-017); OPERACION → 403 en las 6 rutas

### Tests for User Story 9 ⚠️

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation (T011–T014)**

- [ ] T007 [P] [US9] Create `backend/tests/unit/test_conteo_validacion.rs` for pure rules in `backend/src/models/conteo.rs`: `deposito_id <= 0` → `validacion`, `cantidad_contada < 0` → `validacion`, `observacion` > 500 caracteres → `validacion`, `observacion` en blanco normaliza a `None`, `observacion` con espacios se recorta

- [ ] T008 [P] [US9] Create `backend/tests/integration/test_conteos_http.rs` (alta y consulta): `POST /api/conteos` → 201 con `estado=ABIERTO` y una línea por producto+ubicación con `cantidad_contada: null` (9.1); depósito inactivo → 422 `deposito_inactivo`; depósito sin filas de stock → 422 `validacion` sin crear conteo; `GET /api/conteos` con filtros → 200 página; `GET /api/conteos/{id}` → 200; id inexistente → 404 `conteo_no_encontrado`; `OPERACION` → 403 `sin_permisos`

- [ ] T009 [P] [US9] Extend `backend/tests/integration/test_conteos_http.rs` (carga y cierre): `PATCH /api/conteos/{id}/lineas` → 200 con `diferencia = cantidad_contada − cantidad_sistema` del snapshot (9.2 + aclaración 3); cerrar con diff → 200 + **un** `AJUSTE` (motivo `INVENTARIO`) con líneas solo para diff ≠ 0 y stock = contado (9.3, SC-017); cerrar sin diff → 200 sin movimiento (9.4); cerrar un `CERRADO` → 409 `conteo_cerrado` (9.5) y sin duplicar el AJUSTE; cerrar con líneas sin contar → 422 `lineas_sin_contar`

- [ ] T010 [P] [US9] Extend `backend/tests/integration/test_conteos_http.rs` (cancelación y RBAC): cancelar → 200 `CANCELADO` sin ajuste; cargar/cerrar/cancelar un `CANCELADO` → 409 `conteo_cancelado` (9.6); `OPERACION` → 403 `sin_permisos` en las 6 rutas (9.7)

### Implementation for User Story 9

- [X] T011 [P] [US9] Verify pure validation in `backend/src/models/conteo.rs`: `ConteoNuevo`, `validar_conteo` (deposito_id > 0, observacion ≤ 500, trim + vacío → `None`), `LineaContada`, `validar_cantidad_contada`

- [X] T012 [P] [US9] Verify response structs in `backend/src/models/stock.rs`: `ConteoDetalle`, `ConteoLinea`, `ListaConteos`, `ConteoFila`, `ConteoLineaFila` with `sqlx::FromRow`

- [ ] T013 [US9] Remove the two `unwrap()` on `cantidad_contada` in `backend/src/services/conteos.rs:234,346` (Principio II) and return `AppError::LineasSinContar`; the earlier null-check at line ~220 must remain the single gate

- [ ] T014 [US9] In `backend/src/services/conteos.rs`: return `AppError::DepositoInactivo` (not generic `Validacion`) when the deposit is inactive (line ~68); after loading `stock_por_deposito` in `crear`, if zero rows, return `Validacion("El depósito no tiene stock registrado para contar.")` without creating the conteo (aclaración 5)

- [X] T015 [US9] Verify service `backend/src/services/conteos.rs` exposes `crear`, `cargar_lineas`, `cerrar`, `cancelar`, `listar`, `obtener`, with the close in a single `pool.begin()` transaction and `FOR UPDATE`

- [X] T016 [US9] Verify handler `backend/src/handlers/conteos.rs`: DTOs `CrearConteoDto` (with `observacion: Option<String>`), `CargarLineasDto`, `LineaContadaDto`; `exigir_gestion` on all six handlers; utoipa annotations

- [X] T017 [US9] Verify router registration: `pub mod conteos;` in `backend/src/handlers/mod.rs`, `.nest("/api/conteos", handlers::conteos::router())` in `backend/src/app.rs:53`, paths/schemas in `backend/src/docs.rs:74-79,160-161`

- [ ] T018 [US9] Update utoipa `responses(...)` in `backend/src/handlers/conteos.rs` to document the specific 409/422 codes (`conteo_cerrado`, `conteo_cancelado`, `lineas_sin_contar`, `deposito_inactivo`) instead of a generic "Cuerpo inválido"

- [ ] T019 [US9] Run backend tests and make them pass: `cd backend && cargo test`; fix until `test_conteo_validacion` and `test_conteos_http` are green

- [X] T020 [P] [US9] Verify Zod schema `frontend/src/lib/schemas/conteos.ts`: `ConteoSchema`, `ConteoLineaSchema`, `ListaConteosSchema`, `CrearConteoSchema`, `CargarLineasSchema` with exported types

- [X] T021 [P] [US9] Verify typed API client `frontend/src/lib/api/conteos.ts`: `crearConteo`, `listarConteos`, `obtenerConteo`, `cargarLineas`, `cerrarConteo`, `cancelarConteo`

- [ ] T022 [US9] Create `frontend/src/lib/schemas/conteos.test.ts` covering valid parse, `cantidad_contada` nullability, `observacion` optional and rejection of negative quantities (Principio IV: schemas have no tests today)

- [X] T023 [US9] Verify page `frontend/src/routes/conteos/+page.svelte` (abrir por depósito, inputs de `cantidad_contada`, diferencia, cerrar/cancelar, listado con filtros)

- [X] T024 [US9] Verify navigation link "Conteos" in `frontend/src/routes/+layout.svelte:57` (visible para `GESTION`)

- [ ] T025 [US9] Extend `frontend/src/lib/api/conteos.test.ts` to cover the specific error codes (`lineas_sin_contar`, `deposito_inactivo`, `conteo_cancelado`) since they change what the SPA can distinguish

- [ ] T026 [US9] Run frontend tests and checks: `cd frontend && npm test && npm run check && npm run build`

**Checkpoint**: User Story 9 fully functional and testable independently

---

## Phase 4: Polish & Cross-Cutting Concerns

**Purpose**: Quality gates, docs, and final verification

- [ ] T027 [P] Update API documentation table in `docs/api.md` with the six `/api/conteos` routes and the `GESTION` row of the RBAC matrix (currently absent)

- [X] T028 [P] Align `specs/009-inventario-fisico/contracts/api.md` and `data-model.md` with the clarified spec (specific error codes, `observacion`, `diferencia` baseline, empty-deposit rule) — done in the 2026-11-07 session

- [ ] T029 [P] Extend `backend/tests/unit/test_openapi.rs`: add `/api/conteos`, `/api/conteos/{id}`, `/api/conteos/{id}/lineas`, `/api/conteos/{id}/cerrar`, `/api/conteos/{id}/cancelar` to `esperadas`, and `ConteoDetalle` / `ListaConteos` to the schema list

- [ ] T030 Run full quality gates: `cd backend && cargo test && cargo sqlx prepare --check`; `cd frontend && npm test && npm run check && npm run build`; verify no `unwrap`/`expect`/`panic!` in `backend/src/services/conteos.rs` and `backend/src/handlers/conteos.rs`

- [ ] T031 Run `quickstart.md` scenarios 1–9 end to end (including depósito inactivo, depósito sin stock, líneas sin contar y conteo cancelado)

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: T001 verificado; T002 depende de acceso a las bases
- **Foundational (Phase 2)**: depende de Setup — BLOQUEA US9
- **User Story 9 (Phase 3)**: depende de Foundational
- **Polish (Phase 4)**: depende de US9

### Within User Story 9

- Tests (T007–T010) MUST be written and FAIL before T013–T014
- T011/T012 (modelos) antes de T013/T014 (servicio)
- T013/T014 antes de T019 (verde de backend)
- Frontend (T020–T026) puede ir después de T019
- T022/T025 son tests: escribirlos antes de tocar schemas/cliente si el cambio lo justifica

### Parallel Opportunities

- T007, T008, T009, T010 — archivos de test distintos
- T011 y T012 — archivos distintos
- T020 y T021 — schema y cliente
- T027, T028, T029 — docs y OpenAPI en archivos distintos
- T004 y T006 — error.rs y common/mod.rs, sin dependencia entre sí

---

## Parallel Example: User Story 9

```bash
# Tras T006, lanzar los tests de US9 (deben fallar):
Task: "Unit tests de validación en backend/tests/unit/test_conteo_validacion.rs"
Task: "Integration tests de alta/consulta en backend/tests/integration/test_conteos_http.rs"
Task: "Integration tests de carga/cierre en backend/tests/integration/test_conteos_http.rs"
Task: "Integration tests de cancelación/RBAC en backend/tests/integration/test_conteos_http.rs"

# Implementación del servicio (secuencial, mismo archivo):
Task: "Quitar unwrap y devolver lineas_sin_contar en backend/src/services/conteos.rs"
Task: "DepositoInactivo + rechazo de depósito sin stock en backend/src/services/conteos.rs"
```

---

## Implementation Strategy

### MVP First (User Story 9 Only)

1. Phase 1: verificar migración y aplicarla en ambas bases
2. Phase 2: variantes de error nuevas + helper de test
3. Phase 3: tests primero (rojo), luego servicio, luego verde de backend, luego frontend
4. **STOP and VALIDATE**: correr `quickstart.md` 1–9
5. Phase 4: docs, OpenAPI y gates completos

### Incremental Delivery

1. Migración + Fundacional → esquema y errores listos
2. US9 backend (T013–T019) → API funcional, tests verdes
3. US9 frontend (T022, T025–T026) → UI con códigos nuevos
4. Polish (T027–T031) → gates verdes y docs actualizadas

---

## Reconciliación con el tasks.md anterior

| Anterior | Estado previo | Ahora | Motivo |
|---|---|---|---|
| T001 | `[X]` | T001 `[X]` | archivo existe en `migrations/` (ruta corregida) |
| T002 | `[X]` | T002 `[ ]` | no verificable desde el repo; requiere ambas bases |
| T003 | `[X]` | T003 `[X]` + **T004 `[ ]`** | 3 de 5 variantes existían; faltan `lineas_sin_contar` y `deposito_inactivo` |
| T004 | `[X]` | T005 `[X]` | constante verificada |
| T005 | `[X]` | **T006 `[ ]`** | helper `crear_conteo_de_prueba` no existe |
| T006–T009 | `[X]` | **T007–T010 `[ ]`** | **los archivos de test no existen** |
| T010 | `[X]` | T011 `[X]` | validación pura verificada |
| T011 | `[X]` | T012 `[X]` | structs de respuesta verificados |
| T012 | `[X]` | T015 `[X]` + **T013/T014 `[ ]`** | servicio existe, pero conserva 2 `unwrap` y códigos genéricos |
| T013 | `[X]` | T016 `[X]` + **T018 `[ ]`** | handlers existen; falta documentar códigos en utoipa |
| T014 | `[X]` | T017 `[X]` | router y OpenAPI registrados |
| T015 | `[X]` | T019 `[ ]` | nunca hubo suite de conteos que poner en verde |
| T016–T019 | `[X]` | T020/T021/T023/T024 `[X]` | artefactos de frontend verificados |
| T020 | `[X]` | T026 `[ ]` + **T022/T025 `[ ]`** | falta test de schemas y de códigos nuevos |
| T021 | `[X]` | **T027 `[ ]`** | `docs/api.md` no tiene rutas de conteos |
| T022 | `[X]` | T030 `[ ]` | gates no cubrían conteos |
| T023 | `[X]` | **T029 `[ ]`** | `test_openapi.rs` no incluye rutas de conteos |

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Commit after each task or logical group
- Verify T007–T010 fail before implementing T013–T014 (Principio IV, sin waiver)
- Migraciones en `migrations/`, no en `backend/migrations/`
