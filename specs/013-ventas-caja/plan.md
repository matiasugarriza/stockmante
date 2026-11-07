# Plan de implementación: Ventas de caja y clientes (013)

**Branch**: `013-ventas-caja` | **Fecha**: 2026-11-07 | **Spec**: [spec.md](./spec.md)

**Migración**: `migrations/0014_ventas.sql` (ya aplicada; esta fase no agrega migraciones)

**Input**: Feature specification from `/specs/013-ventas-caja/spec.md`

## Summary

Completar el ciclo de venta: cerrar los contratos que la clarificación fijó (precio solo del
catálogo, líneas repetidas agregadas por producto, permisos `GESTION` para anular y escribir
clientes, 409 `documento_duplicado`), agregar `stock_resultante` a la respuesta, implementar el POS
en `/caja` y las pantallas `/admin/ventas` y `/admin/clientes`, y cubrir todo con tests RED→GREEN.
El backend ya tiene la migración aplicada y endpoints operativos; el frontend y los tests son lo
que falta, junto con las correcciones de permisos y de agregación detectadas en el análisis.

## Technical Context

**Language/Version**: Rust estable, edición 2021 (backend) · TypeScript con Svelte 5 / SvelteKit 3
(frontend)

**Primary Dependencies**: axum 0.8, sqlx 0.8 (PostgreSQL, rust_decimal), utoipa 6, tokio, chrono ·
SvelteKit SPA (`adapter-static`), Tailwind CSS 4, Zod, Vitest

**Storage**: PostgreSQL; migraciones versionadas en `migrations/`; `stock_por_deposito` es la
fuente de verdad del stock y `productos.stock_actual` su cache (Principio V)

**Testing**: `cargo test` (targets declarados en `backend/Cargo.toml`, helpers en
`tests/unit/common/mod.rs`) · `npm test` (Vitest) para `$lib/api/*` y stores

**Target Platform**: servidor Linux/Windows para el backend; navegador para la SPA

**Project Type**: aplicación web (backend Rust + frontend SvelteKit)

**Performance Goals**: sin objetivos nuevos; se mantiene el estándar del proyecto (búsqueda y
listados paginados con respuesta sub-segundo en la BD local)

**Constraints**: cero `unwrap`/`panic!`/`expect` en rutas de producción; toda escritura de stock
en una transacción SQLx; `cargo sqlx prepare --check` limpio

**Scale/Scope**: un punto de venta por depósito, ventas de pocas líneas (≤ 50), catálogo de hasta
10 000 productos; 3 pantallas (`/caja`, `/admin/ventas`, `/admin/clientes`)

## Constitution Check

*GATE: debe pasar antes de la Fase 0 y se re-evalúa tras la Fase 1.*

| Principio | Gate | Cumple |
|---|---|---|
| I. Simplicidad | Sin dependencias nuevas; reusa `movimientos`, `stock_por_deposito`, `RequiereGestion` y los patrones de 012 | ✅ |
| II. Mantenibilidad | Lógica en `services::ventas`/`clientes`; frontend con `$lib/api` + schemas; `models/cliente.rs` separado (R5) | ✅ |
| III. Mensajes | Mensajes exactos de la spec para stock, anulación, precio y producto inactivo; la respuesta incluye `stock_resultante` (R9) | ✅ |
| IV. Testabilidad | RED primero: integración de ventas/clientes/RBAC y unitarios de validación; `agregar_lineas` puro | ✅ |
| V. Integridad | Una transacción por venta/anulación; guarda de stock por cantidad agregada; `StockInsuficiente` 409 y `DocumentoDuplicado` 409 mapeados | ✅ |
| SDD | Spec clarificada → verificación de esquema → plan/tasks → implementación incremental | ✅ |

Sin violaciones que justificar: **Complexity Tracking** vacío.

## Project Structure

### Documentation (this feature)

```text
specs/013-ventas-caja/
├── plan.md              # este archivo
├── research.md          # Fase 0: R1–R9
├── data-model.md        # Fase 1: entidades, estados y errores
├── quickstart.md        # Fase 1: validación end-to-end
├── contracts/api.md     # Fase 1: contratos HTTP y permisos
├── spec.md              # clarificada 2026-11-07
└── tasks.md             # Fase 2 (/speckit.tasks)
```

### Source Code (repository root)

```text
backend/
├── src/
│   ├── models/
│   │   ├── cliente.rs           # nuevo (R5): Cliente, ListaClientes
│   │   ├── mod.rs               # registra `cliente`
│   │   └── venta.rs             # Venta, VentaLinea (+stock_resultante), DTOs
│   ├── services/
│   │   ├── clientes.rs          # 409 documento_duplicado (R4)
│   │   └── ventas.rs            # agregar_lineas + precio de catálogo (R1, R2)
│   ├── handlers/
│   │   ├── clientes.rs          # RequiereGestion en escrituras (R3)
│   │   └── ventas.rs            # RequiereGestion en anular (R3)
│   ├── error.rs                 # DocumentoDuplicado (R4)
│   └── docs.rs                  # respuestas 403/409 en OpenAPI
├── tests/
│   ├── unit/
│   │   ├── common/mod.rs
│   │   └── test_venta_validacion.rs
│   └── integration/
│       ├── test_ventas_http.rs
│       ├── test_clientes_http.rs
│       └── test_rbac_matriz.rs
└── Cargo.toml                   # [[test]] de los targets nuevos

frontend/
├── src/
│   ├── lib/
│   │   ├── api/{ventas,clientes}.ts (+ tests)
│   │   └── schemas/{ventas,clientes}.ts
│   └── routes/
│       ├── +layout.svelte       # enlaces Ventas y Clientes (GESTION)
│       ├── caja/+page.svelte    # POS
│       └── admin/
│           ├── ventas/+page.svelte
│           └── clientes/+page.svelte

docs/
└── api.md                       # sección ventas/clientes
```

**Structure Decision**: aplicación web con backend y frontend separados; el backend sigue
`models → services (transacción) → handlers → app.rs` y el frontend centraliza HTTP en `$lib/api`
con schemas Zod, como el resto de los módulos.

## Fase 0 — Research (completada)

[research.md](./research.md) registra las decisiones con su rationale y alternativas:

- **R1** precio solo de `productos.precio_venta`; `precio_unitario` en el request → 422.
- **R2** líneas repetidas se agregan por producto antes de validar y persistir (SC-021).
- **R3** `RequiereGestion` en anular venta y en escrituras de clientes; lecturas con sesión.
- **R4** `AppError::DocumentoDuplicado` → 409 `documento_duplicado`.
- **R5** migración `0014_ventas.sql` y `models/cliente.rs`.
- **R6** alcance: POS + `/admin/ventas` + `/admin/clientes` + enlaces de navegación.
- **R7** tests RED primero y gates completos.
- **R8** trazabilidad FR en tasks y `docs/api.md`.
- **R9** `stock_resultante` en `VentaLinea` desde `movimiento_lineas`.

## Fase 1 — Design & Contracts

- [data-model.md](./data-model.md): entidades, máquina de estados `CONFIRMADA → ANULADA`, efectos
  sobre stock y catálogo de errores. **Sin cambios de esquema.**
- [contracts/api.md](./contracts/api.md): `POST/GET /api/ventas`, `GET /api/ventas/{id}`,
  `POST /api/ventas/{id}/anular` y CRUD de `/api/clientes`, con matriz de permisos.
- [quickstart.md](./quickstart.md): escenarios curl y de UI que prueban 13.1–13.9, FR-049–FR-055 y
  SC-021.

### Superficie de API (resumen)

| Método | Ruta | Permiso | Éxito | Errores propios |
|---|---|---|---|---|
| POST | `/api/ventas` | OPERACION/GESTION | 201 `VentaDetalle` | 409 `stock_insuficiente`, 422 `validacion`, 404 |
| GET | `/api/ventas` | sesión | 200 `ListaVentas` | 401 |
| GET | `/api/ventas/{id}` | sesión | 200 `VentaDetalle` | 404 |
| POST | `/api/ventas/{id}/anular` | **GESTION** | 200 `VentaDetalle` | 409 `venta_ya_anulada`, 403, 404 |
| GET | `/api/clientes` | sesión | 200 `ListaClientes` | 401 |
| GET | `/api/clientes/{id}` | sesión | 200 `Cliente` | 404 |
| POST/PUT | `/api/clientes`, `/api/clientes/{id}` | **GESTION** | 201/200 `Cliente` | 409 `documento_duplicado`, 422, 403 |
| POST | `/api/clientes/{id}/activar`, `/desactivar` | **GESTION** | 200 `Cliente` | 403, 404 |

## Cambios de esquema

**Ninguno.** `0014_ventas.sql` ya crea `clientes`, `ventas`, `venta_lineas` y la secuencia;
`movimientos.venta_id` y los `CHECK` de stock existen desde 0009/0014. La agregación de líneas, el
rechazo de precio y `stock_resultante` son cambios de servicio/DTO.

## Archivos

- `migrations/0014_ventas.sql` (sin cambios; verificar aplicación en ambas BD — T001)
- `backend/src/models/cliente.rs` (nuevo), `backend/src/models/mod.rs`,
  `backend/src/models/venta.rs`
- `backend/src/services/ventas.rs`, `backend/src/services/clientes.rs`
- `backend/src/handlers/ventas.rs`, `backend/src/handlers/clientes.rs`
- `backend/src/error.rs`, `backend/src/docs.rs`
- `backend/tests/unit/test_venta_validacion.rs`,
  `backend/tests/integration/test_ventas_http.rs`,
  `backend/tests/integration/test_clientes_http.rs`,
  `backend/tests/integration/test_rbac_matriz.rs`, `backend/Cargo.toml`
- `frontend/src/lib/api/ventas.ts`, `clientes.ts` y sus tests;
  `frontend/src/lib/schemas/ventas.ts`, `clientes.ts`
- `frontend/src/routes/caja/+page.svelte`, `frontend/src/routes/admin/ventas/+page.svelte`,
  `frontend/src/routes/admin/clientes/+page.svelte`, `frontend/src/routes/+layout.svelte`
- `docs/api.md`

## Riesgos

- **Alto — descuento de stock**: toda venta/anulación en una transacción; agregación por producto
  antes de validar (R2) para no violar `CHECK (cantidad >= 0)` ni devolver 500.
- **Alto — permisos**: si `anular_venta` o las escrituras de clientes quedan con `AuthUsuario`,
  FR-055 se incumple y el test RBAC falla (R3).
- **Medio — precio**: aceptar `precio_unitario` habilitaría venta bajo costo; el rechazo 422 debe
  cubrir también el valor igual al de lista (R1).
- **Medio — concurrencia**: mantener el orden de bloqueo de 007 (depósitos → productos → stock).
- **Bajo — documento duplicado**: el `UNIQUE` global incluye inactivos; la baja lógica no libera el
  documento (decisión R4).

## Dependencias y orden

1. 003 (productos y precios), 004 (`/caja` búsqueda), 005 (depósitos), 007 (movimientos).
2. Verificación de esquema contra la BD real antes de tocar SQL (T001).
3. Backend completo y en verde antes de las pantallas; el POS consume `$lib/api/ventas.ts`.
4. Alimenta 014 (reportes de ventas) y 015 (auditoría ampliada).

## Correcciones respecto del plan anterior

| Hallazgo del análisis | Corrección en este plan |
|---|---|
| G1 FR-055 sin cobertura | R3: `RequiereGestion` + tests RBAC |
| C2 P-IV sin tests | R7: tests RED de ventas, clientes y RBAC |
| I1 tasks marcadas en falso | todas las tareas vuelven a `[ ]` con entregables verificables |
| U1 líneas duplicadas → 500 | R2: agregación por producto y Escenario 13.9 |
| A1 precio arbitrario | R1: solo `productos.precio_venta` |
| C3 sin `$lib/api` de ventas/clientes | R6: clientes API tipados + schemas + tests |
| I2 estado de spec desactualizado | corregido en la clarificación |
| I3 migración con tres nombres | R5: `migrations/0014_ventas.sql` |
| I4 `models/cliente.rs` inexistente | R5: se crea el archivo |
| I5 anulación fuera del plan | incluida en contrato y tasks |
| I6/I7 pantallas admin fuera de spec | FR-054 las exige y se implementan |
| I8 dependencia 005 omitida | listada en Dependencias |
| A2 documento duplicado sin contrato | R4: 409 `documento_duplicado` |
| A3 paginación sin límites | `normalizar_paginacion` (50/100) documentado |
| A4/A5 alcance y roles ambiguos | Clarifications: matriz rol→familia y alcance de UI |
| U3 sin trazabilidad | tasks cita FR/SC en cada tarea |
| U4 cliente/depósito inactivo | 422 `validacion` documentado |
| D1 duplicación spec/plan | el plan referencia FR en lugar de reexplicar |

## Complexity Tracking

> Sin violaciones de la constitución que justificar.
