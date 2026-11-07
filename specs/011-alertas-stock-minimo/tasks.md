# Tasks: 011-alertas-stock-minimo

**Input**: Design documents from `/specs/011-alertas-stock-minimo/`

**Prerequisites**: plan.md ✓, spec.md ✓ (clarificado 2026-11-07), research.md ✓, data-model.md ✓,
contracts/api.md ✓, quickstart.md ✓

**Tests**: OBLIGATORIOS (Constitución P-IV: RED antes de GREEN, sin waiver). Cada bloque de
implementación tiene su test escrito antes.

**Organization**: una sola user story (US11) en dos incrementos entregables: **consulta** (MVP) y
**digest**.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: paralelizable (archivos distintos, sin dependencia de una tarea incompleta)
- **[Story]**: `[US11]` en las fases de historia; setup, foundational y polish no llevan etiqueta

## Path Conventions

- Backend: `backend/src/`, `backend/tests/`
- Migraciones: `migrations/`
- Frontend: `frontend/src/`
- Docs: `docs/`

> **Nota**: las tareas T001–T003 del `tasks.md` anterior (panel "Bajo mínimo" y badge) quedan
> **reemplazadas**. El badge ya existe en `+layout.svelte` y el filtro en `/admin/stock`, pero
> ninguno cumple la spec: faltan `faltante`, el endpoint `GET /api/alertas` y todo el backend.
> T016 y T017 de este archivo modifican ese código existente.

---

## Phase 1: Setup

**Purpose**: esquema, migración y configuración base.

- [X] T001 [P] Verificar el esquema real de la BD en ejecución (MCP de PostgreSQL o `psql`): columnas y tipos de `productos` (`stock_actual INT`, `stock_minimo INT`, `activo BOOLEAN`) y `usuarios` (`familia_rol`, `activo`, `email`), y confirmar que `notificaciones_stock` no existe. Registrar el resultado en la tabla "Verificación de esquema" de `specs/011-alertas-stock-minimo/data-model.md`.
- [X] T002 Crear `migrations/0015_alertas_stock.sql` con la tabla `notificaciones_stock` (DDL exacto de `data-model.md`): columnas `id/producto_id/usuario_id/fecha/estado/intentos/enviada_en/error`, `CHECK (estado IN ('pendiente','enviada','fallida'))`, `CHECK (intentos >= 0)`, `UNIQUE (producto_id, usuario_id, fecha)`, `CHECK (estado <> 'enviada' OR enviada_en IS NOT NULL)` e índice parcial `idx_notificaciones_pendientes (fecha, estado) WHERE estado <> 'enviada'`. Depende de T001.
- [X] T003 [P] RED: agregar a `backend/tests/unit/test_config.rs` los casos de `ALERTAS_INTERVALO_MINUTOS`: ausente → 60, `"0"` → desactivado, `"15"` → 15, `"abc"` → `AppError::Validacion` (deben fallar).
- [X] T004 GREEN: en `backend/src/config.rs` agregar `alertas_intervalo_minutos: u64` a `AppConfig`, parsearlo en `desde_entorno()` con default 60 y `0` como desactivado, y devolver error de configuración ante valor no numérico. Depende de T003.

---

## Phase 2: Foundational

**Purpose**: DTOs, transporte de correo y lecturas del servicio. Bloquea ambas fases de US11.

**⚠️ CRITICAL**: ninguna tarea de US11 puede empezar antes de completar esta fase.

- [X] T005 [P] Crear `backend/src/models/alertas.rs` con `AlertaItem` (`producto_id: i32`, `nombre: String`, `sku: String`, `stock_actual: i64`, `stock_minimo: i32`, `faltante: i64`), `ListaAlertas` (`items`, `total`, `pagina`, `por_pagina`) y `RespuestaCiclo` (`fecha: NaiveDate`, `productos: i64`, `destinatarios: i64`, `enviados: i64`, `fallidos: i64`), todos con `Serialize` y `ToSchema`; registrar `pub mod alertas;` en `backend/src/models/mod.rs`.
- [X] T006 [P] Extender el trait `EnvioCorreo` en `backend/src/mail/envio.rs` con `async fn enviar_alerta_stock(&self, destino: &str, nombre: &str, cuerpo: &str) -> Result<(), String>` (sin implementación por defecto); implementarlo en `CorreoDev` (registra en `bandeja()`) y en `CorreoSmtp` (`backend/src/mail/smtp.rs`, con `lettre` y asunto `Stockmante — productos bajo el stock mínimo`); agregar el método al fake `CorreoQueFalla` de `backend/tests/unit/test_usuarios_alta.rs` para que el workspace siga compilando.
- [X] T007 [P] Crear `backend/src/services/alertas.rs` con las lecturas: `fecha_local(&dyn Reloj) -> NaiveDate` (offset fijo `FixedOffset::west_opt(3 * 3600)`), `detectar(pool, filtros) -> ListaAlertas` (productos `activo = TRUE AND stock_minimo > 0 AND stock_actual < stock_minimo`, `faltante = (stock_minimo - stock_actual)::bigint`, orden `nombre, id`, paginación normalizada) y `usuarios_gestion(pool) -> Vec<UsuarioDestino>` (`familia_rol = 'GESTION' AND activo`); registrar `pub mod alertas;` en `backend/src/services/mod.rs`. Depende de T002, T005.
- [X] T008 Registrar `[[test]]` en `backend/Cargo.toml`: `test_alertas_validacion` → `tests/unit/test_alertas_validacion.rs` y `test_alertas_http` → `tests/integration/test_alertas_http.rs` (crear los archivos vacíos con `mod common;` para que la suite compile).

**Checkpoint**: migración aplicable, DTOs, transporte y lecturas listos.

---

## Phase 3: User Story 11 (a) — Consulta de alertas (Priority: P2) 🎯 MVP

**Goal**: un gerente ve los productos bajo mínimo con su faltante, y `OPERACION` recibe 403.

**Independent Test**: con un producto `stock_minimo=10, stock_actual=4`, `GET /api/alertas` con
sesión `GESTION` devuelve `faltante: 6`; al reponer a 12 desaparece; con `OPERACION` responde 403.

### Tests (escribir primero, deben fallar)

- [X] T009 [P] [US11] RED: `backend/tests/unit/test_alertas_validacion.rs` — `fecha_local` con `RelojFalso` en `02:00Z` da el día anterior en −03:00 y en `12:00Z` el mismo día; normalización de paginación del listado (0 y 101 fuera de rango).
- [X] T010 [P] [US11] RED: `backend/tests/integration/test_alertas_http.rs` — escenarios 11.1 (`faltante = 6`), 11.2 (repuesto a 12 no aparece) y 11.4 (`stock_minimo = 0` no aparece), creando productos con `pool_de_test`.
- [X] T011 [US11] RED: en el mismo archivo, escenario 11.7 para `GET /api/alertas`: `GESTION` → 200, `OPERACION` → 403 `sin_permisos`. (Sin `[P]`: comparte `test_alertas_http.rs` con T010.)

### Implementation

- [X] T012 [US11] GREEN: completar `fecha_local` y la normalización de paginación en `backend/src/services/alertas.rs` hasta que T009 pase. Depende de T007, T009.
- [X] T013 [US11] Crear `backend/src/handlers/alertas.rs` con `listar` (usa `RequiereGestion`, `AppEstado.pool` y `services::alertas::detectar`), registrar `pub mod alertas;` en `backend/src/handlers/mod.rs` y `nest("/api/alertas", handlers::alertas::router())` en `backend/src/app.rs` dentro del router autenticado. Depende de T005, T007, T010, T011.
- [X] T014 [P] [US11] Crear `frontend/src/lib/schemas/alertas.ts` (schema zod de `AlertaItem` y `ListaAlertas`) y `frontend/src/lib/api/alertas.ts` con `listarAlertas(params)` tipado, siguiendo el patrón de `$lib/api/stock.ts`.
- [X] T015 [P] [US11] Test de `frontend/src/lib/api/alertas.test.ts`: URL y query params correctos, parseo del schema y propagación de error HTTP.
- [X] T016 [US11] En `frontend/src/routes/admin/stock/+page.svelte`, sección destacada "Bajo mínimo" que use `listarAlertas` y muestre mínimo, stock actual y faltante; mantener el filtro existente de 010 sin cambios.
- [X] T017 [US11] En `frontend/src/routes/+layout.svelte`, que el badge use `listarAlertas` (no `listarStock`), se llame solo si `familia_rol === 'GESTION'` y se refresque al montar y al cambiar de ruta; sin polling.

**Checkpoint**: consulta completa y testeable; el digest todavía no existe.

---

## Phase 4: User Story 11 (b) — Digest y ejecución (Priority: P2)

**Goal**: un correo resumen diario por usuario `GESTION`, sin duplicados, con disparo manual y
reintento tras un fallo de correo.

**Independent Test**: con 3 productos bajo mínimo y 2 usuarios `GESTION`, `POST /api/alertas/enviar`
devuelve `productos: 3, destinatarios: 2, enviados: 6`; un segundo `POST` el mismo día devuelve
`enviados: 0` y no agrega correos a la bandeja.

### Tests (escribir primero, deben fallar)

- [X] T018 [P] [US11] RED: `backend/tests/integration/test_alertas_http.rs` — escenario 11.3 + SC-019: dos ciclos el mismo día → un correo por usuario con los 3 productos y una sola fila por `(producto_id, usuario_id, fecha)`.
- [X] T019 [US11] RED: escenario 11.5: `POST /api/alertas/enviar` con `GESTION` → 200 con conteos; `OPERACION` → 403; sin usuarios `GESTION` activos → 200 con `enviados: 0`. (Sin `[P]`: comparte archivo con T018.)
- [X] T020 [US11] RED: escenario 11.6 con `CorreoQueFalla`: el `POST` responde 200 con `fallidos > 0`, las filas quedan `estado = 'fallida'` con `error` e `intentos = 1`, y un segundo ciclo con `CorreoDev` envía solo esas filas sin duplicar las ya enviadas. (Sin `[P]`: comparte archivo con T018/T019.)

### Implementation

- [X] T021 [US11] GREEN: implementar `ciclo(pool, correo, reloj)` en `backend/src/services/alertas.rs`: detectar, `INSERT ... ON CONFLICT (producto_id, usuario_id, fecha) DO NOTHING`, reclamo con `SELECT ... WHERE fecha = $1 AND estado <> 'enviada' FOR UPDATE SKIP LOCKED` dentro de `pool.begin()`, render del cuerpo por usuario (nombre, SKU, stock, mínimo, faltante), envío por `enviar_alerta_stock` y `UPDATE` a `enviada`/`fallida` con `enviada_en`, `intentos` y `error`. Depende de T007, T018, T019, T020.
- [X] T022 [US11] Agregar `enviar` en `backend/src/handlers/alertas.rs` (`RequiereGestion`, sin cuerpo, devuelve `RespuestaCiclo`) y la ruta `POST /api/alertas/enviar`. Depende de T021.
- [X] T023 [US11] En `backend/src/main.rs`, si `config.alertas_intervalo_minutos > 0`, `tokio::spawn` de un bucle con `tokio::time::interval` y `MissedTickBehavior::Skip` que llama a `services::alertas::ciclo`; con `0` no se crea la tarea (y `construir_router` no arranca ningún timer). Depende de T021.

**Checkpoint**: US11 completa; SC-019 verificable.

---

## Phase 5: Polish & Cross-Cutting

**Purpose**: documentación viva, OpenAPI y quality gates.

- [X] T024 [P] Registrar `handlers::alertas::listar`/`enviar` y los schemas `AlertaItem`, `ListaAlertas`, `RespuestaCiclo` en `backend/src/docs.rs`, y extender las listas esperadas de `backend/tests/unit/test_openapi.rs` con `/api/alertas`, `/api/alertas/enviar` y esos schemas.
- [X] T025 [P] Agregar a `docs/api.md` las filas de `GET /api/alertas` y `POST /api/alertas/enviar` con permiso `GESTION` y códigos 200/401/403.
- [ ] T026 Ejecutar la validación manual de `specs/011-alertas-stock-minimo/quickstart.md` (migración, 11.1–11.7, scheduler con `ALERTAS_INTERVALO_MINUTOS=1` y `0`).
- [X] T027 Frontend: `npm test`, `npm run check` y `npm run build` en `frontend/` en verde.
- [X] T028 Backend: `cargo test` en verde y `cargo sqlx prepare --check` sin diferencias (incluye las queries nuevas de `services/alertas.rs`).
- [ ] T029 Re-ejecutar `/speckit-analyze 011-alertas-stock-minimo` y confirmar que los hallazgos C1–C5 y H1–H6 quedaron cerrados.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: T001 sin dependencias; T002 depende de T001; T003→T004 en cadena (RED antes de GREEN).
- **Foundational (Phase 2)**: depende de Setup; **bloquea** las dos fases de US11.
- **US11 (a) (Phase 3)**: depende de Foundational; entrega el MVP.
- **US11 (b) (Phase 4)**: depende de Foundational; reutiliza `detectar` y `usuarios_gestion` de (a).
- **Polish (Phase 5)**: depende de (a) y (b).

### Task Dependencies

- T002 ← T001 · T004 ← T003
- T007 ← T002, T005 · T008 ← T005, T007
- T012 ← T007, T009 · T013 ← T005, T007, T010, T011
- T021 ← T007, T018, T019, T020 · T022 ← T021 · T023 ← T021
- T024/T025 ← T013, T022

### Parallel Opportunities

- T001 y T003 pueden ejecutarse juntas (archivos distintos).
- En Foundational: T005, T006 y T007 son paralelizables salvo la dependencia declarada.
- En US11 (a): T009, T010 y T011 (tests, archivos en teoría compartidos: T010 y T011 usan el mismo `test_alertas_http.rs`, así que conviene encadenarlos o repartir bloques distintos del archivo); T014 y T015 son paralelizables.
- En US11 (b): T018, T019 y T020 comparten `test_alertas_http.rs`; escribirlos en secuencia dentro del archivo. T024 y T025 son paralelizables.

```bash
# Ejemplo: tests de la fase de consulta
Task: "RED unit fecha_local + paginación en backend/tests/unit/test_alertas_validacion.rs"
Task: "RED integración 11.1/11.2/11.4 en backend/tests/integration/test_alertas_http.rs"
```

## Implementation Strategy

### MVP primero

1. Phase 1 + Phase 2 completas (migración, DTOs, transporte, lecturas, config).
2. Phase 3 completa (US11 a): consulta con `faltante`, 403 para `OPERACION`, panel y badge.
3. **PARAR y validar**: `cargo test test_alertas_http` + revisar `/admin/stock`.
4. Recién entonces Phase 4 (digest, `POST`, scheduler) y Phase 5.

### Entrega incremental

- (a) consulta: valor inmediato sin correo ni scheduler.
- (b) digest: agrega el aviso proactivo; no rompe (a).
- (a) y (b) son independientes del orden de 012 en adelante.

## Notes

- Tests antes de implementación en cada bloque, sin excepción (P-IV).
- Sin `unwrap()`/`expect()`/`panic!` en producción; errores como `Result<T, AppError>`.
- `GET /api/stock?bajo_minimo=true` no se modifica en ninguna tarea.
- Commit por tarea o grupo lógico; `cargo sqlx prepare` tras cada query nueva.
