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
- [x] T012 `tests/integration/test_movimientos_http.rs` (escenarios 7.1–7.8, SC-009/010/011).
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
