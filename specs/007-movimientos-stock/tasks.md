# Tareas: Movimientos de stock (007)

**Spec**: `spec.md` · **Plan**: `plan.md` · **Migración**: `0009_stock_movimientos.sql`

## Fase 1 — Esquema

- [x] T001 Crear `migrations/0009_stock_movimientos.sql` con `stock_por_deposito`,
  `movimientos`, `movimiento_lineas`, `productos.stock_actual` e índices.
  **Aceptación**: `sqlx migrate run` aplica sin error en `stockmante` y `stockmante_test`.
- [x] T002 Verificar el esquema real contra la migración (columnas, `CHECK`, `UNIQUE NULLS NOT
  DISTINCT`). **Aceptación**: `\d stock_por_deposito` coincide con el plan.

## Fase 2 — Dominio

- [x] T003 `models/movimiento.rs`: `TIPOS_MOVIMIENTO`, `MOTIVOS`, `validar_tipo`,
  `validar_motivo`, `validar_observacion`, `validar_lineas` (no vacía, máximo 200, delta por
  signo, sin duplicados producto+ubicación). **Aceptación**: tests unitarios verdes.
- [x] T004 `error.rs`: `StockInsuficiente { deposito, disponibles, solicitadas }` (409
  `stock_insuficiente`) y `ProductoInactivo` (409 `producto_inactivo`).
  **Aceptación**: mensajes literales de la spec.

## Fase 3 — Servicio

- [x] T005 `services/movimientos.rs::crear`: transacción única; valida depósito activo,
  bloquea productos, valida ubicación permitida, aplica deltas con `FOR UPDATE`, escribe
  líneas con `stock_resultante`, recalcula el cache y audita.
  **Aceptación**: escenarios 7.1–7.7 de la spec; rollback total ante cualquier fallo.
- [x] T006 `services/movimientos.rs::listar` y `obtener` con filtros y paginación.
  **Aceptación**: 200 con total, página y `por_pagina`.
- [x] T007 `services/stock.rs::listar` y `detalle` (global, por depósito y por ubicación).
  **Aceptación**: `stock_actual` coincide con la suma del desglose.

## Fase 4 — HTTP

- [x] T008 `handlers/movimientos.rs` con `POST /api/movimientos`, `GET /api/movimientos`,
  `GET /api/movimientos/{id}`; `AJUSTE` reservado a `GESTION`.
  **Aceptación**: 201/200/403/409/422 del contrato.
- [x] T009 `handlers/stock.rs` con `GET /api/stock` y `GET /api/stock/{producto_id}`.
- [x] T010 Router en `app.rs`, módulos en `handlers/mod.rs` y paths/schemas/tags en `docs.rs`.
  **Aceptación**: `test_openapi` incluye las cinco rutas.

## Fase 5 — Pruebas

- [x] T011 `tests/unit/test_movimiento_validacion.rs` (reglas puras).
- [x] T012 `tests/integration/test_movimientos_http.rs` (escenarios 7.1–7.8, SC-013/014/015).
- [x] T013 `cargo test` completo en verde; `cargo sqlx prepare --check` sin diferencias.

## Fase 6 — Frontend

- [x] T014 `lib/schemas/movimientos.ts` + `lib/api/movimientos.ts` + `lib/api/stock.ts`.
- [x] T015 `routes/movimientos/+page.svelte`: alta con líneas dinámicas, motivo condicionado y
  listado con filtros; `AJUSTE` oculto para `OPERACION`.
- [x] T016 Enlace en `routes/+layout.svelte` para ambas familias.
- [x] T017 Tests de schemas y cliente; `npm test`, `npm run check`, `npm run build`.

## Fase 7 — Cierre

- [x] T018 `docs/api.md` con las rutas nuevas.
- [x] T019 Commit de la unidad de trabajo con migración, código, tests y docs.

## Fase 8 — Ampliación 2026-11-07 (remediación de analyze/clarify)

Tests primero: cada test de T021–T024, T031 y T032 DEBE observarse en RED antes de T026
(Principio IV). Solo T011, T012, T025 y T033 quedan cubiertos por el waiver W-005. Sin migración nueva.
Orden: T020 → tests en RED (T021–T025, T031–T033) → T026 → T027–T029 y T034 → T030.

- [X] T020 Crear `specs/007-movimientos-stock/progress.md`: registrar que T011/T012 no dejaron
  evidencia de RED (no se reconstruye un RED artificial) y proponer con `/speckit constitution`
  el waiver W-005 con alcance T011, T012, T025 y T033 (constitución 1.3.1 → 1.3.2, PATCH).
  **Aceptación**: `progress.md` existe y W-005 está **aprobado y registrado** en
  «Waivers registrados» de `.specify/memory/constitution.md`; proponerlo no alcanza.
- [X] T021 Test RED en `tests/unit/test_movimiento_validacion.rs`: matriz tipo ↔ motivo (cada
  combinación admitida y una rechazada por tipo), motivos reservados `TRANSFERENCIA`,
  `INVENTARIO`, `IMPORTACION` con su mensaje, y `cantidad_entera` (entero válido, `2.5` con el
  mensaje de 7.11, valor fuera de `i32`) (FR-024, FR-028, 7.9, 7.11).
- [X] T022 Test RED en `tests/integration/test_movimientos_http.rs`: `ENTRADA` con `VENTA` y
  movimiento con motivo reservado → 422 `validacion` sin cabecera, líneas, auditoría ni cambio de
  stock (7.9, SC-015).
- [X] T023 Test RED: `AJUSTE` negativo mayor que el stock con sesión `GESTION` → 409
  `stock_insuficiente` con el mensaje literal nuevo y stock intacto (7.4, SC-013).
- [X] T024 Test RED de mensajes con siguiente paso (FR-028, Principio III): ubicación inactiva →
  422 con el mensaje nuevo; depósito y producto inexistentes → 422 nombrando el id; aserciones
  de depósito inactivo, ubicación inválida, `producto_inactivo` (7.7) y `stock_insuficiente`
  (7.2) actualizadas a los literales del plan.
- [X] T025 Test de concurrencia: dos altas simultáneas del mismo producto en ubicaciones distintas
  terminan con `stock_actual` igual a la suma de `stock_por_deposito` (SC-014, decisión 4 del
  plan). Verifica un bloqueo ya implementado: cubierto por W-005; si pasa en la primera corrida
  se registra en `progress.md`.
- [X] T031 Test RED de precedencia (7.10, FR-028): `OPERACION` + `AJUSTE` + motivo `VENTA` +
  producto inactivo → 403 `sin_permisos`; `GESTION` + `ENTRADA` + `VENTA` + producto inactivo →
  422 por el motivo. En ambos casos no se escribe nada.
- [X] T032 Test RED de cantidad no entera (7.11): `cantidad_delta: 2.5` → 422 con
  `"La cantidad de cada línea debe ser un número entero distinto de cero."` y nada escrito.
- [X] T033 Test del filtro `categoria_id` de `GET /api/stock` en `test_movimientos_http.rs`: solo
  devuelve productos de esa categoría (FR-029; `q` y `bajo_minimo` ya están cubiertos). Verifica
  un comportamiento ya implementado: cubierto por W-005; si pasa en la primera corrida se
  registra en `progress.md`.
- [X] T026 Implementar (depende de T021–T024, T031, T032 en RED): `validar_motivo_para_tipo` y
  `cantidad_entera` en `models/movimiento.rs`; `cantidad_delta: serde_json::Number` con
  `#[schema(value_type = i32)]` en `handlers/movimientos.rs`; en `services/movimientos.rs::crear`,
  el 403 de `AJUSTE` justo después de `validar_tipo` y los mensajes reescritos (depósito,
  producto, ubicación, cantidad resultante); en `error.rs`, los textos nuevos de
  `StockInsuficiente` y `ProductoInactivo`. Sin `unwrap`/`expect`/`panic!`. **Aceptación**:
  T021–T024, T031 y T032 en verde; `test_asistente_http.rs` actualizado al literal nuevo; los
  flujos de 008, 009, 012, 013 y 016 siguen en verde.
- [X] T027 [P] Frontend (después de T026): `MOTIVOS_POR_TIPO` en `lib/schemas/movimientos.ts` con
  test, y selector de motivo filtrado por tipo en `routes/movimientos/+page.svelte` (sin motivos
  reservados); alinear los fixtures de `movimientos.test.ts`, `transferencias.test.ts` y
  `asistente.test.ts` al literal nuevo de `stock_insuficiente`.
- [X] T028 [P] Asistente (017, después de T026): limitar `motivo` de `proponer_movimiento` a la
  matriz en `asistente/src/tools.ts`, con test que rechace una combinación incoherente.
- [X] T029 [P] `docs/api.md` (después de T026): matriz de motivos del alta manual, orden de
  evaluación de errores, 422 nuevos y textos nuevos de `stock_insuficiente`/`producto_inactivo`.
- [X] T034 [P] Alinear el literal de `stock_insuficiente` (y de `producto_inactivo` donde
  aparezca) en `specs/008-transferencias-depositos/spec.md` (8.2) y su `contracts/api.md`,
  `specs/013-ventas-caja/spec.md`, `contracts/api.md` y `quickstart.md`, y
  `specs/017-asistente-inventario/spec.md` (después de T026).
- [X] T030 Quality gates (requiere W-005 registrado por T020): `cargo test`,
  `cargo sqlx prepare --check`, `npm test`, `npm run check`, `npm run build` (frontend) y
  `npm test` + `npm run check` (asistente).

## Trazabilidad

| Requisito | Tareas |
|---|---|
| FR-024 | T003, T005, T008, T011, T021, T022, T026, T027, T028 |
| FR-025 | T001, T002, T005, T013 |
| FR-026 | T005, T008, T012 |
| FR-027 | T008, T012, T015, T031 |
| FR-028 | T003, T005, T011, T012, T021, T024, T026, T029, T031, T032 |
| FR-029 | T006, T007, T009, T010, T012, T033 |
| FR-030 | T005, T008, T012, T015 |
| SC-013 | T005, T012, T013, T023 |
| SC-014 | T005, T012, T025 |
| SC-015 | T005, T012, T022, T031, T032 |
| Principio III (mensajes compartidos en 008/013/017) | T024, T026, T027, T034 |
| Principio IV (evidencia de RED, W-005) | T020, T021–T025, T031–T033 |
| Soporte (esquema, router/OpenAPI, gates, frontend/docs) | T001–T002, T010, T013, T016–T019, T029, T030 |
