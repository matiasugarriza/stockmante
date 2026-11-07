# Plan de implementación: Alertas de stock mínimo (011)

**Branch**: `011-alertas-stock-minimo` | **Fecha**: 2026-11-07 | **Spec**: [spec.md](./spec.md)
**Migración**: `0015_alertas_stock.sql` · **Research**: [research.md](./research.md) ·
**Modelo**: [data-model.md](./data-model.md) · **Contratos**: [contracts/api.md](./contracts/api.md) ·
**Validación**: [quickstart.md](./quickstart.md)

## Summary

Feature completa de alertas: tabla `notificaciones_stock` (dedup por producto/usuario/día), endpoint
`GET /api/alertas` (solo `GESTION`) con `faltante`, `POST /api/alertas/enviar` para disparar el
ciclo, un scheduler en `main.rs` cada `ALERTAS_INTERVALO_MINUTOS` y digest por correo a los usuarios
`GESTION` activos con reintento de las filas fallidas. El frontend agrega la sección "Bajo mínimo"
con faltante en `/admin/stock` y un badge de conteo para `GESTION`, sin polling.

Reemplaza el plan anterior, que declaraba "ninguna migración" y reducía el alcance a un panel
opcional: FR-042 y FR-043 quedaban sin implementar (hallazgos C1–C5 del análisis).

## Technical Context

**Language/Version**: Rust 2021 (edition 2021), backend `stockmante-backend`; TypeScript + Svelte 5
(runes) en `frontend/`.

**Primary Dependencies**: axum 0.8, sqlx 0.8 (`postgres`, `chrono`, `migrate`), tokio 1, chrono 0.4,
utoipa 6, lettre 0.11 (ya presentes). **Ninguna dependencia nueva**: el offset −03:00 se resuelve con
`chrono::FixedOffset` y el scheduler con `tokio::time` (R4, R5).

**Storage**: PostgreSQL. Tabla nueva `notificaciones_stock`; `productos` y `usuarios` solo se leen.

**Testing**: `cargo test` (unit + integración, cada archivo registrado en `[[test]]` de
`backend/Cargo.toml`), `cargo sqlx prepare --check`; frontend `vitest` + `npm run check`.

**Target Platform**: servidor Linux/Windows con PostgreSQL; SPA en navegador.

**Project Type**: web application (backend Rust + frontend SvelteKit).

**Performance Goals**: sin objetivo de throughput. El ciclo debe completarse en menos de un
`ALERTAS_INTERVALO_MINUTOS` (60 min por defecto); SC-019 exige que una alerta sea visible en
`GET /api/alertas` dentro de un ciclo.

**Constraints**: `AlertasIntervalo` inválido impide arrancar. El ciclo no debe romper la API ante un
SMTP caído (11.6). Cero correos duplicados por producto y día (SC-019).

**Scale/Scope**: catálogo de hasta ~10⁴ productos; un ciclo por minuto como máximo en tests
(`ALERTAS_INTERVALO_MINUTOS=1`), 60 por defecto. Aviso solo a usuarios `GESTION` activos.

## Constitution Check

*GATE: pasa antes de Phase 0; se re-evalúa después de Phase 1.*

| Principio | Cumplimiento | Nota |
|---|---|---|
| I. Simplicidad (NO NEGOCIABLE) | PASS | Una tabla, un servicio, un handler, un timer. Sin factory/strategy/colas. Sin dependencias nuevas. La consulta se duplica respecto de 010 a propósito: reusar el filtro rompería 11.7 y el contrato de 010 (R1). |
| II. Mantenibilidad (NO NEGOCIABLE) | PASS | Lógica en `services::alertas` (funciones nombrables), handler delgado, router en `app.rs`. El render del correo vive en el servicio, no en el transporte. Frontend con `$lib/api/alertas.ts` tipado + schema zod; sin `fetch` suelto. |
| III. Claridad de mensajes (NO NEGOCIABLE) | PASS | El correo enumera producto, SKU, stock, mínimo y faltante. 403 usa `sin_permisos` existente. Respuestas JSON sin adornos. `POST` devuelve conteos concretos (11.5). |
| IV. Testabilidad (NO NEGOCIABLE) | PASS | Reloj inyectado (`Reloj`) para la fecha local; transporte inyectado (`EnvioCorreo`) para el fallo; ciclo invocable sin scheduler. RED antes de GREEN en cada incremento. |
| V. Integridad de datos (NO NEGOCIABLE) | PASS | DDL verificado contra el esquema real (data-model.md). El reclamo de filas usa transacción + `FOR UPDATE SKIP LOCKED`; la restricción única impide duplicados. No se modifica stock. |
| Restricciones técnicas (stack fijo) | PASS | Rust/Axum/SQLx/PostgreSQL/SvelteKit; migración versionada en `/migrations`. |
| Verificación de esquema | PASS | `data-model.md` registra la verificación de `productos`, `usuarios` y la ausencia de `notificaciones_stock` antes del DDL. |
| Flujo SDD | PASS | Spec clarificada (5 respuestas, 2026-11-07) → plan → tasks → implementación. |

**Post-diseño (Phase 1)**: sin violaciones nuevas. `docs/api.md` y `docs.rs` (OpenAPI) sí cambian:
el plan anterior decía "sin cambios de backend", lo que era incorrecto (hallazgo M5).

## Project Structure

### Documentation (this feature)

```text
specs/011-alertas-stock-minimo/
├── spec.md          # clarificado 2026-11-07
├── plan.md          # este archivo
├── research.md      # Phase 0 (R1–R8)
├── data-model.md    # Phase 1 (DDL, estados, DTOs)
├── contracts/
│   └── api.md       # Phase 1 (GET /api/alertas, POST /api/alertas/enviar)
├── quickstart.md    # Phase 1
└── tasks.md         # Phase 2
```

### Source Code (repository root)

```text
migrations/
└── 0015_alertas_stock.sql            # nueva

backend/src/
├── config.rs                         # + alertas_intervalo_minutos
├── app.rs                            # + nest /api/alertas
├── main.rs                           # + spawn del ciclo periódico
├── docs.rs                           # + paths y schemas OpenAPI
├── mail/
│   ├── envio.rs                      # + enviar_alerta_stock (CorreoDev)
│   └── smtp.rs                       # + enviar_alerta_stock (CorreoSmtp)
├── models/
│   ├── mod.rs                        # + pub mod alertas
│   └── alertas.rs                    # nuevo: AlertaItem, ListaAlertas, RespuestaCiclo
├── services/
│   ├── mod.rs                        # + pub mod alertas
│   └── alertas.rs                    # nuevo: fecha_local, detectar, usuarios_gestion, ciclo
└── handlers/
    ├── mod.rs                        # + pub mod alertas
    └── alertas.rs                    # nuevo: listar, enviar

backend/tests/
├── unit/test_alertas_validacion.rs   # nueva: fecha local, paginación, detección pura
└── integration/test_alertas_http.rs  # nueva: 11.1–11.7 + SC-019

frontend/src/
├── lib/api/alertas.ts                # nueva
├── lib/api/alertas.test.ts           # nueva
├── lib/schemas/alertas.ts            # nueva
├── routes/admin/stock/+page.svelte   # sección "Bajo mínimo" con faltante
└── routes/+layout.svelte             # badge GESTION vía /api/alertas

docs/api.md                           # filas de los dos endpoints
```

**Structure Decision**: web application con la capa `models → services → handlers` que ya usan
007–016. Un módulo nuevo por capa; no se toca `services/stock.rs` (010 queda intacto, R1).

## Decisiones de diseño

1. **Endpoint propio** `GET /api/alertas` con `faltante`; `/api/stock` sin cambios (R1, R2).
2. **Dedup por destinatario**: `UNIQUE (producto_id, usuario_id, fecha)`, fecha local derivada del
   reloj inyectado (clarificaciones 1 y 4).
3. **Estado explícito** `pendiente`/`enviada`/`fallida` + `intentos` + `error`; reintento solo de lo
   no enviado (clarificación 2).
4. **Reclamo sin doble envío**: `INSERT ... ON CONFLICT DO NOTHING` + `FOR UPDATE SKIP LOCKED` (R3).
5. **Scheduler en `main.rs`** con `tokio::time::interval` + `MissedTickBehavior::Skip`; `0` desactiva
   (R5). `construir_router` no arranca timers.
6. **Un correo por usuario y ciclo** con los productos pendientes de ese usuario (11.3).
7. **Frontend sin polling**: badge solo `GESTION`, refresh al montar y al navegar (clarificación 5).
8. **Sin dependencias nuevas**: `chrono::FixedOffset` (Argentina sin DST) y `tokio::time` (R4, R5).

## Superficie de API

| Método | Ruta | Permiso | Respuesta |
|---|---|---|---|
| GET | `/api/alertas` | GESTION | 200 lista paginada con `faltante` |
| POST | `/api/alertas/enviar` | GESTION | 200 `RespuestaCiclo` |

Detalle en [contracts/api.md](./contracts/api.md).

## DDL (resumen)

Tabla `notificaciones_stock` con `id`, `producto_id`, `usuario_id`, `fecha DATE`, `estado`,
`intentos`, `enviada_en TIMESTAMPTZ NULL`, `error TEXT NULL`, `UNIQUE (producto_id, usuario_id,
fecha)`, `CHECK` de estado y de `enviada_en` coherente, más índice parcial de pendientes. Columnas,
reglas y máquina de estados en [data-model.md](./data-model.md).

## Estrategia de pruebas

Principio IV: **RED antes de GREEN** en cada incremento; sin waiver preventivo.

**Unitarias** (`backend/tests/unit/test_alertas_validacion.rs`)

- `fecha_local` con `RelojFalso` en 02:00 UTC → día anterior en −03:00 (borde de día).
- `ALERTAS_INTERVALO_MINUTOS` vacío → 60; `0` → desactivado; `"abc"` → error de configuración
  (extiende `test_config.rs`).
- Normalización de paginación del listado.

**Integración** (`backend/tests/integration/test_alertas_http.rs`, con `pool_de_test`/`app_de_test`)

| Escenario | Assert |
|---|---|
| 11.1 | `GET /api/alertas` 200 con `stock_actual`, `stock_minimo`, `faltante = 6` |
| 11.2 | tras reponer a 12, el producto no aparece |
| 11.4 | `stock_minimo = 0` nunca aparece |
| 11.3 + SC-019 | dos ciclos el mismo día → 1 correo por usuario con 3 productos y sin filas duplicadas |
| 11.5 | `POST` 200 con `productos`/`destinatarios`/`enviados` correctos |
| 11.6 | `CorreoQueFalla`: 200, filas `fallida` con `error` e `intentos = 1`; siguiente ciclo envía solo las fallidas |
| 11.7 | `OPERACION` → 403 `sin_permisos` en GET y POST |
| sin destinatarios | 0 usuarios `GESTION` activos → 200 y `enviados: 0` |

**Frontend** (`frontend/src/lib/api/alertas.test.ts`)

- URL y query params correctos, parseo del schema zod, propagación de error, conteo para el badge.

## Archivos afectados

| Archivo | Cambio |
|---|---|
| `migrations/0015_alertas_stock.sql` | nuevo: tabla `notificaciones_stock` |
| `backend/src/config.rs` | `alertas_intervalo_minutos` + parseo (0 desactiva) |
| `backend/src/models/alertas.rs`, `models/mod.rs` | DTOs de alerta y respuesta del ciclo |
| `backend/src/services/alertas.rs`, `services/mod.rs` | detección, destinatarios, fecha local, ciclo con reclamo y reintento, render del correo |
| `backend/src/handlers/alertas.rs`, `handlers/mod.rs` | `listar` (GET) y `enviar` (POST) |
| `backend/src/app.rs` | `nest("/api/alertas", ...)` dentro de la superficie autenticada |
| `backend/src/main.rs` | `tokio::spawn` del ciclo si el intervalo > 0 |
| `backend/src/mail/envio.rs`, `mail/smtp.rs` | `enviar_alerta_stock` en el trait y en las dos implementaciones |
| `backend/tests/unit/test_usuarios_alta.rs` | `CorreoQueFalla` implementa el método nuevo |
| `backend/tests/unit/test_config.rs` | casos del intervalo |
| `backend/tests/unit/test_openapi.rs` | rutas y schemas nuevos en el documento |
| `backend/Cargo.toml` | `[[test]]` de los dos archivos nuevos |
| `backend/src/docs.rs` | paths `listar`/`enviar` y schemas `AlertaItem`, `ListaAlertas`, `RespuestaCiclo` |
| `frontend/src/lib/api/alertas.ts`, `lib/schemas/alertas.ts` (+ test) | cliente tipado y schema |
| `frontend/src/routes/admin/stock/+page.svelte` | sección "Bajo mínimo" con faltante |
| `frontend/src/routes/+layout.svelte` | badge vía `/api/alertas`, solo `GESTION`, refresh al navegar |
| `docs/api.md` | filas de `GET /api/alertas` y `POST /api/alertas/enviar` |

## Riesgos

- **Doble envío por concurrencia** (ciclo programado + `POST`). Mitigado con `ON CONFLICT DO
  NOTHING` + `FOR UPDATE SKIP LOCKED` (R3) y cubierto por la prueba de dos ciclos.
- **Fuga de temporizador en tests**. Mitigado: el timer solo se crea en `main.rs` (R5).
- **Offset fijo −03:00**. Válido mientras Argentina no aplique DST; si cambiara, se ajusta en una
  función (`fecha_local`) sin tocar el resto.
- **Crecimiento de `notificaciones_stock`**: una fila por producto/usuario/día. Con 10³ productos y
  10 gestores son ~3,6 M filas al año; el índice parcial mantiene barato el ciclo. Purga fuera de
  alcance (no la pide la spec).
- **Correo en dev**: sin `SMTP_URL` se usa `CorreoDev`; valida el flujo pero no la entrega real.

## Complexity Tracking

Sin violaciones que justificar. La única duplicación deliberada es la query de detección respecto
de `services/stock.rs`: reusar el filtro exigiría cambiar el contrato de 010 y dejaría pasar
`OPERACION`, contradiciendo 11.7 (R1).

## Fuera de alcance (confirmado por la spec)

Notificaciones a `OPERACION`, suscripciones, umbrales por depósito, WhatsApp/SMS/push, sugerencia de
cantidad de compra (012/014), historial purgado y reintentos con backoff exponencial.
