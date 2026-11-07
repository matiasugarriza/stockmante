# Plan de implementación: Reportes, kardex y valorización (014)

**Branch**: `014-reportes-kardex` | **Fecha**: 2026-11-07 | **Spec**: [spec.md](./spec.md)

**Migración**: ninguna (solo lectura).

**Input**: Feature specification from `/specs/014-reportes-kardex/spec.md`

## Summary

Cerrar los reportes de gestión: los cuatro endpoints de `/api/reportes` ya existen pero sin
control de rol en tres de ellos, con fechas que pierden el último día, reloj oculto en el
servicio, rotación que cuenta transferencias, CSV incompleto y sin escapar, y sin tests ni
frontend. Este plan aplica las cuatro clarificaciones (todo `GESTION`, salidas sin
transferencias, rango obligatorio en rotación/ventas, día local UTC-03:00), extrae la lógica pura
(rango, acumulado, rotación, CSV) para probarla sin I/O, reemplaza el N+1 de rotación por una
consulta agregada y agrega `/admin/reportes` con su cliente tipado.

## Technical Context

**Language/Version**: Rust estable, edición 2021 (backend) · TypeScript con Svelte 5 / SvelteKit
(frontend)

**Primary Dependencies**: axum 0.8, sqlx 0.8 (PostgreSQL, rust_decimal), utoipa, chrono · SvelteKit
SPA, Tailwind CSS 4, Zod, Vitest. **Sin dependencias nuevas** (R7).

**Storage**: PostgreSQL, solo lectura de `movimientos`, `movimiento_lineas`,
`stock_por_deposito`, `productos`, `depositos`, `ventas`, `venta_lineas` (ver
[data-model.md](./data-model.md))

**Testing**: `cargo test` (targets `[[test]]` en `backend/Cargo.toml`, helpers en
`tests/unit/common/mod.rs`) · `npm test` (Vitest) para `$lib/api/reportes.ts`

**Target Platform**: servidor para el backend; navegador para la SPA

**Project Type**: aplicación web (backend Rust + frontend SvelteKit)

**Performance Goals**: sin SLA nuevo; rotación en una consulta (no N+1) y verificación con
`EXPLAIN ANALYZE` en la BD de desarrollo (R3)

**Constraints**: cero `unwrap`/`panic!`/`expect` en producción; sin `Utc::now()` en el servicio
(R2); `cargo sqlx prepare --check` limpio

**Scale/Scope**: catálogo de hasta 10 000 productos; rangos de semanas a meses; 1 pantalla con 4
componentes

## Constitution Check

*GATE: debe pasar antes de la Fase 0 y se re-evalúa tras la Fase 1.*

| Principio | Gate | Cumple |
|---|---|---|
| I. Simplicidad | Sin dependencias ni tablas nuevas; CSV con función propia de ~20 líneas (R7); sin paginación (R9) | ✅ |
| II. Mantenibilidad | Lógica pura en `services/reportes/{rango,csv,calculo}.rs`; handlers delgados; un componente Svelte por reporte; HTTP solo en `$lib/api/reportes.ts` | ✅ |
| III. Mensajes | 422 específicos para rango, fechas, faltantes y parámetros; 404 con nombre de recurso; JSON sin adornos | ✅ |
| IV. Testabilidad | Rango, acumulado, rotación y CSV puros y con unitarios; sin reloj oculto (R2); RED primero (R12); `reportes.test.ts` | ✅ |
| V. Integridad | Solo lectura (sin transacciones de escritura); valorización desde `stock_por_deposito` (R4); esquema verificado antes del SQL (T001) | ✅ |
| SDD | Spec clarificada → verificación de esquema → plan/tasks → implementación incremental | ✅ |

Re-evaluación tras la Fase 1: sin cambios. **Complexity Tracking** vacío.

## Project Structure

### Documentation (this feature)

```text
specs/014-reportes-kardex/
├── plan.md              # este archivo
├── research.md          # Fase 0: R1–R12
├── data-model.md        # Fase 1: tablas leídas, respuestas y errores
├── quickstart.md        # Fase 1: validación end-to-end
├── contracts/api.md     # Fase 1: contratos HTTP
├── spec.md              # clarificada 2026-11-07
└── tasks.md             # Fase 2 (/speckit.tasks)
```

### Source Code (repository root)

```text
backend/
├── src/
│   ├── services/
│   │   ├── reportes.rs          # se convierte en services/reportes/mod.rs
│   │   └── reportes/
│   │       ├── mod.rs           # consultas: kardex, stock_valorizado, rotacion, ventas
│   │       ├── rango.rs         # parsear_dia, limites_utc, exigir_rango (R2)
│   │       ├── calculo.rs       # acumular_saldos, calcular_rotacion (R3, R5)
│   │       └── csv.rs           # a_csv RFC 4180 + BOM (R7)
│   ├── handlers/reportes.rs     # RequiereGestion, Formato/Agrupar, CSV (R1, R7, R8)
│   └── docs.rs                  # respuestas 403/404/422 y parámetros en OpenAPI
├── tests/
│   ├── unit/test_reportes_calculo.rs
│   └── integration/
│       ├── test_reportes_http.rs
│       └── test_rbac_matriz.rs  # filas de /api/reportes (SC-031)
└── Cargo.toml                   # [[test]] de los targets nuevos

frontend/src/
├── lib/
│   ├── api/reportes.ts (+ reportes.test.ts)
│   ├── schemas/reportes.ts
│   └── components/reportes/{Kardex,StockValorizado,Rotacion,Ventas}.svelte
└── routes/
    ├── +layout.svelte           # enlace Reportes (GESTION)
    └── admin/reportes/+page.svelte

docs/api.md                      # sección Reportes (spec 014)
```

**Structure Decision**: aplicación web; el backend sigue `services → handlers → app.rs` (las
rutas ya están montadas en `app.rs`) y el frontend centraliza HTTP en `$lib/api` con schemas Zod,
como el resto de los módulos.

## Fase 0 — Research (completada)

[research.md](./research.md):

- **R1** `RequiereGestion` en los cuatro handlers.
- **R2** día local UTC-03:00, límites semiabiertos, sin reloj en el servicio.
- **R3** rotación en una consulta, `SALIDA` sin `TRANSFERENCIA`.
- **R4** valorización desde `stock_por_deposito`, productos con stock > 0.
- **R5** kardex con saldo inicial/final y filtro de depósito; SC-022.
- **R6** ventas `CONFIRMADA`, día local sin `AT TIME ZONE '-03'`.
- **R7** CSV propio RFC 4180 + BOM para los cuatro reportes.
- **R8** `formato`/`agrupar` cerrados con 422.
- **R9** sin paginación.
- **R10** `/admin/reportes` con cuatro componentes y descarga por enlace.
- **R11** consultas en tiempo de ejecución como el resto del proyecto.
- **R12** tests RED primero.

## Fase 1 — Design & Contracts

- [data-model.md](./data-model.md): tablas leídas, tipos de entrada, respuestas y catálogo de
  errores. **Sin cambios de esquema.**
- [contracts/api.md](./contracts/api.md): `GET /api/reportes/{kardex,stock-valorizado,rotacion,ventas}`.
- [quickstart.md](./quickstart.md): escenarios curl y de UI para 14.1–14.7, casos borde, SC-022 y
  SC-031.

### Superficie de API (resumen)

| Método | Ruta | Permiso | Rango | CSV |
|---|---|---|---|---|
| GET | `/api/reportes/kardex` | **GESTION** | opcional | ✅ |
| GET | `/api/reportes/stock-valorizado` | **GESTION** | — | ✅ |
| GET | `/api/reportes/rotacion` | **GESTION** | obligatorio | ✅ |
| GET | `/api/reportes/ventas` | **GESTION** | obligatorio | ✅ (`agrupar`) |

## Riesgos

- **Alto — permisos**: hoy kardex, rotación y ventas responden a `OPERACION`; el test RBAC (SC-031)
  debe fallar antes del cambio (R1).
- **Medio — zona horaria**: `AT TIME ZONE '-03'` invierte el signo en PostgreSQL; usar la resta de
  intervalo y probar una venta de las 22:00 locales (R6).
- **Medio — SC-022**: si algún flujo cambió stock sin `movimiento_lineas`, el kardex no
  reconstruye `stock_actual`; el test de integración lo detecta.
- **Bajo — volumen**: sin paginación; acotado por rango (Supuestos).

## Dependencias y orden

1. 007, 008, 012, 013 implementadas.
2. Verificación de esquema contra la BD real antes de tocar SQL (T001).
3. Tests RED → backend en verde → frontend.
4. Alimenta 017 (asistente) con consultas agregadas.

## Correcciones respecto del plan anterior

| Hallazgo del análisis | Corrección |
|---|---|
| C1 sin tests (P-IV) | R12: unitarios + integración + RBAC, RED primero |
| C2 tasks marcadas en falso | todas las tareas vuelven a `[ ]` con entregables verificables |
| C3 sin test del cliente | `reportes.test.ts` (R10) |
| C4 reloj oculto | R2: rango obligatorio, sin `Utc::now()` |
| I1 permisos contradictorios | Clarificación + R1 |
| I2 valorización por depósito / compras | eliminado; el plan sigue la spec |
| I3 paginación | R9: sin paginación |
| I4 último día excluido / saldo vs `stock_actual` | R2 + escenario 14.1 corregido |
| A1 SC-022 suma doble | reformulado en la spec |
| A2 qué es "salida" | Clarificación + R3 |
| F1 CSV incompleto | R7: CSV en los cuatro, ventas con `agrupar` |
| U1 CSV sin escapar | R7 |
| U2 fechas/parámetros inválidos silenciosos | R8 + Casos borde |
| U3 sin casos borde | sección Casos borde en la spec |
| U4 N+1 en rotación | R3 |
| D1 consultas no verificadas en compilación | R11 (decisión transversal, fuera de 014) |
| D2 artefactos faltantes | research, data-model, contracts, quickstart creados |
| L1 estado de spec | corregido |
| L2 variables sin uso | desaparecen con R1 |

## Complexity Tracking

> Sin violaciones de la constitución que justificar.
