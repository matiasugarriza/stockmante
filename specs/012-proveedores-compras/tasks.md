# Tasks: Proveedores y compras (012)

**Input**: Design documents from `specs/012-proveedores-compras/`

**Prerequisites**: plan.md ✓, spec.md ✓ (aclarada 2026-11-07), research.md ✓, data-model.md ✓,
contracts/api.md ✓, quickstart.md ✓

**Tests**: obligatorios (Principio IV). En la fase 6 cada test se escribe **antes** que la
corrección, se corre contra el código actual y su fallo (RED) se registra en `progress.md`.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: se puede hacer en paralelo (archivos distintos, sin dependencias pendientes).
- **[US12]**: única historia de la feature.

## Path Conventions

- Migraciones: `migrations/` (raíz del repo)
- Backend: `backend/src/`, `backend/tests/unit/`, `backend/tests/integration/`
- Frontend: `frontend/src/`

---

## Phases 1–5: implementación inicial (histórico, completada)

Fases ya cerradas, conservadas como registro. Las descripciones de T001 y T003 se corrigieron el
2026-11-07 para reflejar lo que realmente se construyó (hallazgos I1 e I5 del análisis).

### Phase 1: Setup

- [X] T001 Create migration `migrations/0013_proveedores_compras.sql` (`proveedores`, `compras`, `compra_lineas`, `movimientos.compra_id`)
- [X] T002 Apply migration to the development and test databases

### Phase 2: Backend

- [X] T003 [P] [US12] Create models in `backend/src/models/proveedor.rs` (Proveedor, Compra, CompraLinea, CompraDetalle, ListaCompras) with validation
- [X] T004 [US12] Implement services `backend/src/services/proveedores.rs`, `backend/src/services/compras.rs` (CRUD + confirmación y anulación transaccionales)
- [X] T005 [US12] Create handlers `backend/src/handlers/proveedores.rs`, `backend/src/handlers/compras.rs` + router registration in `backend/src/app.rs` + OpenAPI in `backend/src/docs.rs`
- [X] T006 [US12] Tests unit + integration (RED no registrado: ver T025)
- [X] T007 [US12] Run `cargo test`, `cargo sqlx prepare --check`

### Phase 3: Frontend

- [X] T008 [P] [US12] Create `frontend/src/routes/admin/proveedores/+page.svelte`
- [X] T009 [P] [US12] Create `frontend/src/routes/admin/compras/+page.svelte`
- [X] T010 [US12] Schemas + API clients (incompleta al marcarse; cerrada por T015/T016)
- [X] T011 [US12] Run `npm test`, `npm run check`, `npm run build`

### Phase 4: Polish

- [X] T012 Update `docs/api.md`
- [X] T013 OpenAPI verify

### Phase 5: Frontend completo y RBAC (2026-11-06)

Cierra la brecha detectada: T010 estaba marcada pero los clientes y schemas no existían, las páginas
eran solo listados y los handlers no exigían GESTION.

- [X] T014 [US12] RBAC: `backend/src/handlers/proveedores.rs` y `backend/src/handlers/compras.rs` usan `RequiereGestion` (403 `sin_permisos` para OPERACION)
- [X] T015 [US12] Schemas Zod `frontend/src/lib/schemas/proveedores.ts` y `frontend/src/lib/schemas/compras.ts` + tests
- [X] T016 [US12] API clients `frontend/src/lib/api/proveedores.ts` y `frontend/src/lib/api/compras.ts` + tests
- [X] T017 [US12] Componentes `frontend/src/lib/components/proveedores/ProveedorForm.svelte` y `ProveedorTabla.svelte`
- [X] T018 [US12] Componentes `frontend/src/lib/components/compras/CompraForm.svelte` (alta/edición con líneas) y `CompraTabla.svelte`
- [X] T019 [US12] Páginas `admin/proveedores` y `admin/compras`: alta/edición, confirmar/anular/eliminar borrador, estado vacío y errores 401/403/parseo
- [X] T020 [US12] Tests backend: matriz RBAC ampliada en `backend/tests/integration/test_rbac_matriz.rs` + `backend/tests/integration/test_compras_http.rs` (flujo completo y 403 OPERACION) (RED no registrado: ver T025)
- [X] T021 [US12] Gates: `cargo test`, `cargo sqlx prepare --check`, `npm test`, `npm run check`, `npm run build`
- [X] T022 [US12] Cerrar advisories de la verificación independiente: validación de líneas en PUT de compras (vacías, cantidad negativa, producto repetido), unicidad de CUIT en edición (`409 cuit_duplicado`) y guard de producto repetido en `CompraForm`
- [X] T023 [US12] Tests de regresión para 12.1, 12.5, 12.6, FR-046 y validaciones de edición (A1-A4) (RED no registrado: ver T025)

---

## Phase 6: Cierre de hallazgos del análisis (2026-11-07)

**Objetivo**: cumplir los Principios III, IV y V y las clarificaciones del 2026-11-07, sin migraciones.

**Prueba independiente**: `quickstart.md` completo en verde, más el test concurrente de SC-021.

### Fundacional (bloquea el resto de la fase)

- [ ] T024 [US12] Verificar el esquema contra la BD en ejecución (MCP de PostgreSQL o `sqlx-cli`): `proveedores`, `compras`, `compra_lineas`, `movimientos.compra_id`, `stock_por_deposito`, `productos.costo NUMERIC(12,2)`; registrar el resultado y cualquier diferencia en `specs/012-proveedores-compras/data-model.md` (C2)
- [ ] T025 [US12] Proponer el waiver W-003 (Principio IV, alcance T006/T020/T023 de 012, sin reconstruir el RED) con `/speckit constitution`, y crear `specs/012-proveedores-compras/progress.md` para registrar el RED de la fase 6 (C1, R7)

### Integridad de la anulación (C3, SC-021)

- [ ] T026 [US12] RED: test concurrente en el archivo nuevo `backend/tests/integration/test_compras_concurrencia.rs` (registrar `[[test]]` en `backend/Cargo.toml`): una compra confirmada de 10 unidades, con una anulación y una `SALIDA` de 007 por 5 unidades en paralelo → una 200 y otra 409 `stock_insuficiente`, ningún 500, saldo ≥ 0
- [ ] T027 [US12] GREEN: en `backend/src/services/compras.rs::anular`, bloquear `depositos` y `productos ORDER BY id FOR UPDATE` antes de leer stock, y descontar con `UPDATE ... SET cantidad = cantidad - $n WHERE ... AND cantidad >= $n RETURNING cantidad`; sin fila → `StockInsuficiente` con el saldo bloqueado (R1; depende de T026)

### Errores y validaciones (U1, U4, U5, R5)

- [ ] T028 [P] [US12] RED: tests unitarios en el archivo nuevo `backend/tests/unit/test_compras_validacion.rs` (registrar `[[test]]`): `validar_numero` (> 40 caracteres), `validar_costo` (> 9.999.999.999,99), `validar_estado` (`PAGADA`) y `parsear_fecha` (`07-11-2026`) devuelven `Validacion`; más tests HTTP en `backend/tests/integration/test_compras_http.rs` para 12.2b (409 `proveedor_inactivo` al crear, editar y confirmar), 12.5b (409 `compra_no_anulable` en borrador y en anulada) y 12.8 (422 en filtros)
- [ ] T029 [US12] GREEN: agregar `AppError::CompraNoAnulable(String)` y `AppError::ProveedorInactivo(String)` (409, códigos `compra_no_anulable` y `proveedor_inactivo`) en `backend/src/error.rs`; usarlos en `backend/src/services/compras.rs` (`validar_destinos`, `confirmar`, `anular`) con los mensajes de `contracts/api.md` (R3; depende de T028)
- [ ] T030 [US12] GREEN: agregar `validar_numero`, `validar_costo` y `validar_estado` en `backend/src/models/proveedor.rs`, aplicarlos en `crear`/`actualizar` de `backend/src/services/compras.rs`, y convertir `parsear_fecha` de `backend/src/handlers/compras.rs` en `Result` con mensaje que nombra el parámetro (R4, R5; depende de T028)

### Resultado concreto al confirmar y anular (C4, Principio III)

- [ ] T031 [US12] RED: en `backend/tests/integration/test_compras_http.rs`, confirmar y anular devuelven `movimiento` con `tipo`, `motivo`, `deposito` y `lineas[].stock_resultante` correctos; SC-020: exactamente una `ENTRADA` con `compra_id` tras confirmar
- [ ] T032 [US12] GREEN: agregar `ResultadoCompra { compra, lineas, movimiento: MovimientoDetalle }` en `backend/src/models/proveedor.rs`; `confirmar` y `anular` de `backend/src/services/compras.rs` lo devuelven usando `services::movimientos::obtener` después del commit; actualizar `responses(...)` en `backend/src/handlers/compras.rs` y el schema en `backend/src/docs.rs` + `backend/tests/unit/test_openapi.rs` (R2; depende de T031 y T027)
- [ ] T033 [P] [US12] RED → GREEN frontend: `ResultadoCompraSchema` en `frontend/src/lib/schemas/compras.ts` (reusa el schema de movimiento de `frontend/src/lib/schemas/movimientos.ts`) y `confirmar`/`anular` en `frontend/src/lib/api/compras.ts` tipados con él, con sus tests en `compras.test.ts`; una función pura que arma el aviso («Compra #7 confirmada en Depósito Central: Cemento 50 kg → 24 u.») con su test (depende de T032)
- [ ] T034 [US12] UI: `frontend/src/routes/admin/compras/+page.svelte` muestra el aviso de T033 al confirmar y al anular; `CompraForm.svelte` rotula `numero` como «Nº factura/remito del proveedor» (máx. 40) y `CompraTabla.svelte` muestra «Compra #id»; mostrar los mensajes de `compra_no_anulable` y `proveedor_inactivo` tal como llegan (clarificación 1; depende de T033)

### Polish

- [ ] T035 [P] Actualizar `docs/api.md` (sección 012): códigos `compra_no_anulable` y `proveedor_inactivo`, 422 de filtros y límites, forma de `ResultadoCompra`
- [ ] T036 Gates: `cargo test`, `cargo sqlx prepare --check`, `npm test`, `npm run check`, `npm run build`; recorrer `quickstart.md`; registrar los resultados en `progress.md`

---

## Dependencies & Execution Order

- **Fases 1–5**: completadas.
- **T024, T025**: bloquean la fase 6. T024 antes de cualquier SQL; T025 puede avanzar en paralelo
  porque no toca código.
- **Bloque C3** (T026 → T027) y **bloque de errores** (T028 → T029, T030): independientes entre sí,
  pero T027, T029 y T030 tocan `services/compras.rs`, así que se aplican en serie.
- **Bloque C4**: T031 → T032 (después de T027, para que el movimiento devuelto refleje la
  anulación corregida) → T033 → T034.
- **Polish**: T035 en paralelo con T034; T036 al final.

## Parallel Example

```text
Después de T024:
  T026 (test concurrente)       ‖  T028 (tests de validación)  ‖  T025 (waiver)
Después de T032:
  T033 (frontend schema/API)    ‖  T035 (docs/api.md)
```

## Implementation Strategy

1. **Integridad primero**: T024 → T026 → T027 elimina el único riesgo de 500 y de saldo inconsistente
   en producción.
2. **Contrato de errores**: T028 → T029 → T030, de bajo riesgo y aditivo.
3. **Resultado visible**: T031 → T034, que cierra el Principio III.
4. **Cierre**: T035, T036 y aprobación de W-003.

## Notes

- Sin migraciones nuevas: si T024 detecta una diferencia con la BD real, se detiene la fase y se
  actualiza `data-model.md` antes de seguir.
- No reconstruir el RED de T006/T020/T023: lo cubre W-003 (R7).
- Commit por tarea o por par RED/GREEN, con la referencia del hallazgo (C3, C4, U1…).
