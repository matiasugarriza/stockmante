# Tasks: 014-reportes-kardex

**Input**: Design documents from `/specs/014-reportes-kardex/`

**Prerequisites**: plan.md ✓, spec.md ✓ (clarificada 2026-11-07), research.md ✓, data-model.md ✓,
contracts/api.md ✓

**Tests**: obligatorios (Constitución, Principio IV). Escribir y ver fallar (RED) antes de
implementar; el gate final exige `cargo test`, `cargo sqlx prepare --check`, `npm test`,
`npm run check` y `npm run build` en verde.

**Organización**: una sola historia (US14, P2). Backend antes que frontend porque la pantalla
consume `$lib/api/reportes.ts`.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: puede ejecutarse en paralelo (archivo distinto, sin dependencias pendientes)
- **[Story]**: US14 (única historia de la spec)
- Cada tarea cita el FR/SC/R que cubre y una ruta de archivo concreta

## Path Conventions

- Backend: `backend/src/`, `backend/tests/`
- Frontend: `frontend/src/`
- Documentación: `docs/`

---

## Phase 1: Setup

- [X] T001 Verificar contra la BD real (MCP de PostgreSQL o `psql`) las tablas y columnas de
  `specs/014-reportes-kardex/data-model.md` (tipos, nullables y valores de los `CHECK` de
  `movimientos.tipo`/`motivo` y `ventas.estado`), en la BD de desarrollo y en la de test;
  registrar el resultado en `data-model.md` (Principio V, precondición de todo SQL)
- [X] T002 [P] Verificar que `backend/tests/unit/common/mod.rs` expone `app_de_test`,
  `sesion_gestion`, `sesion_operacion`, `sku_unico` y `actor_de_test`, y que los targets nuevos se
  declaran como `[[test]]` en `backend/Cargo.toml`

## Phase 2: Foundational (bloqueantes)

**⚠️ CRÍTICO**: los tests unitarios y los servicios dependen de estos módulos.

- [X] T003 Convertir `backend/src/services/reportes.rs` en `backend/src/services/reportes/mod.rs`
  (sin cambio de comportamiento todavía) y crear los módulos vacíos `rango.rs`, `calculo.rs` y
  `csv.rs` con sus firmas públicas de `research.md` (R2, R3, R5, R7); compila con `cargo check`

## Phase 3: User Story 14 — Entender qué pasó con el inventario (Priority: P2) 🎯 MVP

**Goal**: cuatro reportes correctos, solo para `GESTION`, con fechas locales inclusivas, CSV válido
y una pantalla para consultarlos y descargarlos.

**Independent Test**: `cargo test` de `test_reportes_calculo`, `test_reportes_http` y
`test_rbac_matriz`; la secuencia curl de `quickstart.md`.

### Tests for User Story 14 ⚠️ (escribir primero y ver fallar)

- [X] T004 [P] [US14] Crear `backend/tests/unit/test_reportes_calculo.rs` con casos puros:
  `parsear_dia` (AAAA-MM-DD, RFC 3339 tomado en -03:00, formato inválido → mensaje con el nombre
  del campo), `limites_utc` (`hasta=2026-10-31` → fin `2026-11-01T03:00:00Z`, `desde > hasta` →
  mensaje de 14.6), `exigir_rango` (falta uno → mensaje de Casos borde), `acumular_saldos`,
  `calcular_rotacion` (promedio 0 → índice 0; 4 salidas, 20→26 → 0.1739) y `a_csv` (coma, comilla
  y salto de línea entre comillas, BOM, `\r\n`); declarar el target en `backend/Cargo.toml`
  (FR-056, FR-058, FR-059, R2, R3, R5, R7)
- [X] T005 [P] [US14] Crear `backend/tests/integration/test_reportes_http.rs` con datos propios
  (sku únicos): kardex con entrada, venta, merma, ajuste y transferencia → saldo inicial, orden,
  saldo corrido y `saldo_final == stock_actual` sin filtros y `== stock_por_deposito` con
  `deposito_id` (14.1, SC-022); movimiento a las 23:30 UTC del último día incluido; producto
  inexistente 404; stock valorizado sin productos en cero y `total = Σ valor` (14.2); rotación que
  excluye transferencia y merma (14.3); ventas que excluyen anuladas y agrupan por día local
  (venta a las 01:00 UTC cuenta en el día anterior) (14.4); CSV de los cuatro con `Content-Type`,
  encabezados y `agrupar=producto` (14.5); 422 por rango invertido, faltante, fecha mal formada,
  `formato` y `agrupar` inválidos (14.6, Casos borde); declarar el target en `backend/Cargo.toml`
  (FR-056–FR-059, SC-022)
- [X] T006 [US14] Agregar a `backend/tests/integration/test_rbac_matriz.rs` las filas de los
  cuatro reportes en JSON y CSV: `OPERACION` → 403 `sin_permisos`, `GESTION` → 200 (14.7, SC-031)
- [X] T007 [US14] Correr `cargo test` y registrar en el PR o en `progress.md` que T004–T006 fallan
  (RED) por los motivos esperados (Principio IV)

### Implementation for User Story 14

- [X] T008 [P] [US14] Implementar `backend/src/services/reportes/rango.rs`: `parsear_dia`,
  `limites_utc` con `FixedOffset::west_opt(3 * 3600)` y límites semiabiertos, `exigir_rango`;
  mensajes exactos de `data-model.md#errores` (FR-059, R2)
- [X] T009 [P] [US14] Implementar `backend/src/services/reportes/calculo.rs`: `acumular_saldos` y
  `calcular_rotacion` (R3, R5)
- [X] T010 [P] [US14] Implementar `backend/src/services/reportes/csv.rs`: `a_csv` RFC 4180 con BOM
  y `\r\n` (FR-059, R7)
- [X] T011 [US14] En `backend/src/services/reportes/mod.rs` reescribir `kardex` con límites
  semiabiertos, `deposito_id` opcional, `desde`/`hasta` en la respuesta y `acumular_saldos`
  (FR-056, R5)
- [X] T012 [US14] En `backend/src/services/reportes/mod.rs` reescribir `stock_valorizado` desde
  `SUM(stock_por_deposito.cantidad)` con `HAVING > 0`, activos o no (FR-057, R4)
- [X] T013 [US14] En `backend/src/services/reportes/mod.rs` reescribir `rotacion` como una sola
  consulta agregada con `FILTER`, `SALIDA` sin `TRANSFERENCIA`, rango obligatorio y
  `calcular_rotacion`; sin `Utc::now()` (FR-058, R3)
- [X] T014 [US14] En `backend/src/services/reportes/mod.rs` reescribir `ventas`: solo
  `CONFIRMADA`, rango obligatorio, día local con
  `to_char(creado_en AT TIME ZONE 'UTC' - INTERVAL '3 hours', 'YYYY-MM-DD')`; sin `Utc::now()` y sin
  la importación de `normalizar_paginacion` (FR-058, R6, R9)
- [X] T015 [US14] Reescribir `backend/src/handlers/reportes.rs`: `RequiereGestion` en los cuatro
  handlers; `Formato`/`Agrupar` con 422 de R8; fechas vía `rango::parsear_dia`; CSV de los cuatro
  reportes con `a_csv`, `Content-Type: text/csv; charset=utf-8` y `Content-Disposition` con nombre
  `<reporte>_<desde>_<hasta>.csv` (FR-059, SC-031, R1, R7, R8)
- [X] T016 [US14] Actualizar `backend/src/docs.rs` y los `#[utoipa::path]` de
  `backend/src/handlers/reportes.rs`: parámetros de `stock-valorizado`, respuestas 403/404/422 con
  `ErrorRespuesta` y la variante `text/csv` (FR-059)
- [X] T017 [US14] Correr `cargo test` hasta que T004–T006 queden en verde; `EXPLAIN ANALYZE` de
  kardex y rotación en la BD de desarrollo y anotar el resultado en `research.md` (R3)

### Frontend for User Story 14

- [X] T018 [P] [US14] Crear `frontend/src/lib/schemas/reportes.ts` (Zod) con `KardexReporte`,
  `StockValorizadoReporte`, `RotacionReporte` y `VentasReporte` según `data-model.md`
- [X] T019 [US14] Escribir primero `frontend/src/lib/api/reportes.test.ts` (query string con
  fechas, `deposito_id`, `agrupar`; parseo con los schemas; error 422 propagado como `ApiError`;
  `urlCsv` arma `formato=csv`) y verlo fallar; luego crear `frontend/src/lib/api/reportes.ts` con
  `obtenerKardex`, `obtenerStockValorizado`, `obtenerRotacion`, `obtenerVentas` y `urlCsv`
  (Principio IV, R10)
- [X] T020 [P] [US14] Crear `frontend/src/lib/components/reportes/Kardex.svelte` (selector de
  producto, depósito opcional y fechas; tabla con saldo inicial, líneas y saldo final; enlace CSV)
  (FR-056)
- [X] T021 [P] [US14] Crear `frontend/src/lib/components/reportes/StockValorizado.svelte` (tabla y
  total; enlace CSV) (FR-057)
- [X] T022 [P] [US14] Crear `frontend/src/lib/components/reportes/Rotacion.svelte` (fechas
  obligatorias; tabla; enlace CSV) (FR-058)
- [X] T023 [P] [US14] Crear `frontend/src/lib/components/reportes/Ventas.svelte` (fechas
  obligatorias; totales, por día y por producto; enlaces CSV por día y por producto) (FR-058)
- [X] T024 [US14] Crear `frontend/src/routes/admin/reportes/+page.svelte` con selector de reporte
  y los cuatro componentes; mostrar el mensaje del backend en 403/422 (Principio III, R10)
- [X] T025 [US14] Agregar el enlace "Reportes" al bloque `GESTION` de
  `frontend/src/routes/+layout.svelte` (FR-059)

## Phase 4: Polish & Cross-Cutting

- [X] T026 [P] Agregar la sección "Reportes (Módulo 3, spec 014)" a `docs/api.md` con las cuatro
  rutas, permisos, parámetros, CSV y errores (según `contracts/api.md`)
- [X] T027 Correr los gates: `cargo test`, `cargo sqlx prepare --check`, `npm test`,
  `npm run check`, `npm run build`; validar `quickstart.md` (curl y UI)
- [X] T028 Actualizar el estado de 014 en `specs/README.md` y en `spec.md` a "Implementado y
  verificado" con fecha, solo si T027 está en verde

---

## Dependencies & Execution Order

- **Phase 1 → Phase 2 → Phase 3 → Phase 4.**
- T004–T006 dependen de T003 (firmas) y deben fallar (T007) antes de T008–T016.
- T008–T010 en paralelo; T011–T014 dependen de T008–T010 y comparten `mod.rs` (secuenciales).
- T015 depende de T011–T014; T016 de T015; T017 cierra el backend.
- Frontend (T018–T025) después de T017; T020–T023 en paralelo tras T019.
- T026 puede ir en paralelo con el frontend; T027–T028 al final.

## Parallel Example

```text
# RED en paralelo
Task: "Unitarios en backend/tests/unit/test_reportes_calculo.rs"
Task: "Integración en backend/tests/integration/test_reportes_http.rs"

# GREEN en paralelo
Task: "Rango en backend/src/services/reportes/rango.rs"
Task: "Cálculo en backend/src/services/reportes/calculo.rs"
Task: "CSV en backend/src/services/reportes/csv.rs"
```

## Implementation Strategy

### MVP First

1. Phase 1 (T001–T002) y Phase 2 (T003).
2. Tests RED (T004–T007) y backend en verde (T008–T017).
3. Validar con `quickstart.md` (curl) antes del frontend.

### Incremental Delivery

1. Backend corregido → permisos y datos correctos por API (cierra el riesgo de seguridad).
2. Pantalla `/admin/reportes` → uso por el gerente.
3. `docs/api.md` y gates → cierre.

## Notes

- Las tareas del `tasks.md` anterior marcadas `[X]` **no estaban hechas**: tests, frontend,
  `$lib/api/reportes.ts` y `docs/api.md` no existían, y el backend tenía permisos incompletos. Todo
  se reabre acá con entregables verificables.
- Sin migraciones: la feature es de solo lectura.
- Commit después de cada tarea o grupo lógico; no cerrar una tarea sin su test en verde.
