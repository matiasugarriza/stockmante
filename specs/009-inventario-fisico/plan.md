# Plan de implementación: Inventario físico por conteo (009)

**Rama**: `main` · **Fecha**: 2026-11-07 · **Spec**: [`spec.md`](./spec.md) · **Migración**: `migrations/0011_inventario_fisico.sql`

**Feature dir**: `specs/009-inventario-fisico/`

## Summary

El gerente abre un conteo de un depósito, carga las cantidades contadas y lo cierra; el cierre
recalcula las cantidades del sistema contra el stock vigente y, en **una** transacción, genera un
único movimiento `AJUSTE` (motivo `INVENTARIO`) con una línea por diferencia distinta de cero,
dejando `stock_por_deposito` igual a lo contado (SC-017). Un conteo cerrado o cancelado no se
reabre. Toda la superficie (`/api/conteos`) está reservada a `GESTION`.

Este plan **reemplaza** la versión anterior e incorpora las 5 aclaraciones de la sesión
2026-11-07 y la remediación de los hallazgos del análisis (`/speckit-analyze`): códigos de error
específicos, contrato de `observacion`, base de cálculo de `diferencia`, alta sobre depósito vacío,
y la obligación de restaurar los tests de backend y la documentación que figuraban como hechos sin
estarlo.

## Technical Context

**Language/Version**: Rust (edition 2021) en backend; TypeScript + SvelteKit (SPA) en frontend.

**Primary Dependencies**: Axum, SQLx (queries verificadas en compilación), utoipa (OpenAPI),
Zod y Tailwind CSS. No se agregan dependencias nuevas (Principio I).

**Storage**: PostgreSQL; migraciones SQL versionadas en `/migrations` (raíz del repo).

**Testing**: `cargo test` (unitarios en `backend/tests/unit`, integración en
`backend/tests/integration` sobre `stockmante_test`) y `npm test` (Vitest) en frontend.

**Target Platform**: servidor Linux con la API Rust; navegador para la SPA.

**Project Type**: web-service (backend) + SPA (frontend).

**Performance Goals**: sin objetivo numérico en la spec (operación de `GESTION`, no interactiva
masiva). Supuesto operativo: un conteo de hasta ~1.000 líneas se abre y se cierra sin paginación
de líneas.

**Constraints**: el cierre es una única transacción SQLx; prohibido `unwrap`/`expect`/`panic!` en
producción; códigos de error y mensajes estables (Principio III); el stock nunca queda negativo.

**Scale/Scope**: 6 rutas HTTP, 2 tablas nuevas, 1 pantalla de conteo. Conteo total por depósito
(los conteos parciales están fuera de alcance).

## Constitution Check

*GATE: debe pasar antes de Phase 0 y se re-evalúa tras el diseño.*

| Principio | Gate | Estado |
|---|---|---|
| I. Simplicidad | Sin dependencias ni abstracciones nuevas; servicio dedicado solo porque el cierre multi-línea lo exige | PASS |
| II. Mantenibilidad | Handlers delgados, lógica en `services::conteos`; **sin `unwrap`** en producción | **FAIL actual → corregir** (`services/conteos.rs:234,346`) |
| III. Claridad de mensajes | Mensajes específicos y códigos estables (`lineas_sin_contar`, `deposito_inactivo`, `conteo_cerrado`, `conteo_cancelado`, `conteo_no_encontrado`) | PASS tras clarificación |
| IV. Testabilidad | Lógica pura en `models::conteo`; tests unitarios e integración | **FAIL actual → corregir** (los 4 archivos/casos de test de US9 no existen) |
| V. Integridad de datos | Cierre en `pool.begin()`, `FOR UPDATE`, recálculo de `productos.stock_actual` en la misma transacción, guarda de no-negativo | PASS |

**Resultado**: el plan no introduce violaciones nuevas; incluye como trabajo obligatorio reparar
las dos violaciones detectadas (II y IV) antes de considerar 009 cerrado. No se registra waiver:
el Principio IV exige observación del RED y no hay excusa preventiva.

## Clarifications aplicadas (sesión 2026-11-07)

| # | Decisión | Impacto en el diseño |
|---|---|---|
| 1 | Cierre con líneas sin contar → **422 `lineas_sin_contar`**; alta sobre depósito inactivo → **422 `deposito_inactivo`** | Nuevas variantes `AppError::LineasSinContar` y `AppError::DepositoInactivo`; FR-035 y contrato alineados |
| 2 | Conteo `CANCELADO` → **409 `conteo_cancelado`** (distinto de `conteo_cerrado`) | `cargar_lineas`, `cerrar` y `cancelar` distinguen el estado; escenario 9.6 extendido |
| 3 | `diferencia` al cargar = `cantidad_contada − cantidad_sistema` **snapshot de apertura** | `PATCH /lineas` no recalcula contra stock vigente; el recálculo ocurre solo al cerrar |
| 4 | `observacion` **opcional**, máximo 500 | DTO `CrearConteoDto` con `Option<String>`; validación pura en `models::conteo` |
| 5 | Depósito sin filas de stock → **422 `validacion`** sin crear el conteo | `crear` cuenta filas de `stock_por_deposito` dentro de la transacción y aborta con mensaje específico |

## Decisiones de diseño

1. **Tablas `conteos` + `conteo_lineas`**: cabecera con estado; líneas con `cantidad_sistema`
   (snapshot), `cantidad_contada` (nullable) y `diferencia`. Única (conteo, producto, ubicación).
2. **Una transacción para el cierre** (Principio V): bloqueo `FOR UPDATE`, recálculo del stock
   vigente, un único `AJUSTE` con líneas por diff ≠ 0, aplicación de deltas, recálculo de
   `productos.stock_actual` y auditoría, todo en `pool.begin()`.
3. **Servicio dedicado** `services::conteos.rs`: no se reutiliza `movimientos::crear` por línea
   (generaría N movimientos y rompería la atomicidad).
4. **Estados**: `ABIERTO → CERRADO` (ajusta), `ABIERTO → CANCELADO` (no ajusta). No reabrible.
5. **Idempotencia del cierre**: el primer cierre muta; reintentos devuelven 409 sin re-aplicar.
6. **Sin `unwrap`**: el servicio usa `ok_or`/match sobre `cantidad_contada` y devuelve
   `AppError::LineasSinContar`.
7. **`GET` listado y detalle** también para `GESTION`; `OPERACION` recibe 403 en toda la
   superficie.
8. **Cancelación** no toca stock.
9. **`diferencia` con dos momentos**: snapshot en carga, vigente en cierre (ver clarificación 3).
10. **Errores como fuente única**: la tabla de `spec.md` es la referencia; `contracts/api.md` y
    `docs/api.md` se derivan de ella (evita la deriva detectada).

## Superficie de API

| Método | Ruta | Permiso | Respuesta |
|---|---|---|---|
| POST | `/api/conteos` | `GESTION` | 201 conteo `ABIERTO` con líneas |
| GET | `/api/conteos?deposito_id=&estado=&desde=&hasta=&pagina=&por_pagina=` | `GESTION` | 200 página |
| GET | `/api/conteos/{id}` | `GESTION` | 200 detalle con diferencias |
| PATCH | `/api/conteos/{id}/lineas` | `GESTION` | 200 conteo con diferencias |
| POST | `/api/conteos/{id}/cerrar` | `GESTION` | 200 conteo `CERRADO` (+ ajuste) |
| POST | `/api/conteos/{id}/cancelar` | `GESTION` | 200 conteo `CANCELADO` |

**Errores** (fuente: `spec.md`)

| Código | Status | Cuándo |
|---|---|---|
| `conteo_cerrado` | 409 | El conteo ya está cerrado. |
| `conteo_cancelado` | 409 | El conteo está cancelado. |
| `conteo_no_encontrado` | 404 | No existe el conteo. |
| `lineas_sin_contar` | 422 | Al cerrar, alguna línea tiene `cantidad_contada` nula. |
| `deposito_inactivo` | 422 | El depósito está inactivo. |
| `validacion` (existente) | 422 | `deposito_id` inexistente, depósito sin filas de stock, línea ajena, `observacion` > 500, `cantidad_contada` negativa. |

## Project Structure

### Documentación (esta feature)

```text
specs/009-inventario-fisico/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/api.md
└── tasks.md
```

### Código (raíz del repo)

```text
migrations/
└── 0011_inventario_fisico.sql

backend/
├── src/
│   ├── models/{conteo.rs, stock.rs}
│   ├── services/{conteos.rs, auditoria.rs}
│   ├── handlers/conteos.rs
│   ├── error.rs
│   ├── app.rs
│   └── docs.rs
└── tests/
    ├── unit/{test_conteo_validacion.rs, test_openapi.rs}
    ├── unit/common/mod.rs
    └── integration/test_conteos_http.rs

frontend/
└── src/
    ├── lib/{schemas/conteos.ts, api/conteos.ts}
    ├── lib/api/conteos.test.ts
    └── routes/conteos/+page.svelte

docs/api.md
```

**Structure Decision**: se respeta la estructura vigente del repo (backend Axum con capas
`models → services → handlers`, frontend SvelteKit SPA). La migración vive en **`migrations/` de la
raíz**, no en `backend/migrations/` (corrige el hallazgo M1 del análisis).

## Archivos a tocar

| Archivo | Cambio |
|---|---|
| `migrations/0011_inventario_fisico.sql` | tablas `conteos`, `conteo_lineas` (ya existe; solo verificar) |
| `backend/src/error.rs` | + `LineasSinContar` (422), + `DepositoInactivo` (422), `ConteoCerrado` y `ConteoCancelado` ya existen |
| `backend/src/models/conteo.rs` | validación pura: `deposito_id > 0`, `observacion <= 500`, `cantidad_contada >= 0` |
| `backend/src/services/conteos.rs` | quitar los 2 `unwrap()`, usar los códigos nuevos, abortar alta con depósito sin stock |
| `backend/src/handlers/conteos.rs` | DTO con `observacion` opcional; sin cambios de permisos |
| `backend/tests/unit/test_conteo_validacion.rs` | **crear** (no existe) |
| `backend/tests/integration/test_conteos_http.rs` | **crear** (no existe) |
| `backend/tests/unit/common/mod.rs` | helper `crear_conteo_de_prueba` |
| `frontend/src/lib/schemas/conteos.test.ts` | **crear** (tests de Zod) |
| `frontend/src/lib/api/conteos.test.ts` | ya existe; ampliar a los códigos nuevos |
| `docs/api.md` | agregar las 6 rutas + matriz RBAC (hoy ausentes) |
| `specs/009-inventario-fisico/contracts/api.md` | alinear códigos con la spec |
| `specs/009-inventario-fisico/data-model.md` | quitar la fila contradictoria y precisar `diferencia` |

## Estrategia de pruebas

1. **Unitarias (puras, `test_conteo_validacion.rs`)**: `cantidad_contada` negativa → 422;
   `deposito_id` ≤ 0 → 422; `observacion` > 500 → 422; normalización de `observacion` vacía a
   `None`.
2. **Integración (`test_conteos_http.rs`, sobre `stockmante_test`)**:
   - abrir → 201 con líneas del stock actual y `cantidad_contada: null` (9.1);
   - abrir en depósito inactivo → 422 `deposito_inactivo`;
   - abrir en depósito sin stock → 422 `validacion` y sin fila creada (clarificación 5);
   - cargar → 200, `diferencia = contada − sistema` del snapshot (9.2 + clarificación 3);
   - cerrar con diff → 200, **un** `AJUSTE` con líneas por diff ≠ 0, stock = contado (9.3, SC-017);
   - cerrar sin diff → 200, sin movimiento (9.4);
   - cerrar cerrado → 409 `conteo_cerrado` (9.5);
   - cerrar/cargar/cancelar cancelado → 409 `conteo_cancelado` (9.6);
   - cerrar con líneas nulas → 422 `lineas_sin_contar`;
   - cancelar → 200 `CANCELADO`, sin ajuste;
   - `OPERACION` → 403 `sin_permisos` en las 6 rutas (9.7);
   - verificar que un cierre repetido no duplica el `AJUSTE`.
3. **Frontend**: `conteos.test.ts` para schemas y `api/conteos.test.ts` para el cliente.

**RED primero** (Principio IV): T-V1 y T-V2 deben escribirse y fallar antes de tocar el servicio.

## Remediación de hallazgos del análisis

| ID | Hallazgo | Acción en este plan |
|---|---|---|
| C1 | `unwrap()` en `services/conteos.rs:234,346` | Eliminarlos; `LineasSinContar` como error de dominio |
| C2 | T006–T009 marcadas `[X]` sin archivos de test | Reabrir como tareas nuevas en `tasks.md` |
| H1 | `validacion` vs `lineas_sin_contar` | Clarificación 1 → código específico |
| H2 | `deposito_inactivo` inexistente | Clarificación 1 → variante nueva |
| H3 | `conteo_cerrado` vs `conteo_cancelado` | Clarificación 2 |
| M1 | Path `backend/migrations/` | Corregido a `migrations/` |
| M2 | `observacion` fuera del contrato | Clarificación 4 |
| M3 | Base de `diferencia` | Clarificación 3 |
| M5 | Tabla de errores triplicada con deriva | Spec como fuente única; contrato y docs se alinean |
| M6 | Sin tests de schemas | Tarea nueva de `conteos.test.ts` |
| M7 | `docs/api.md` sin rutas | Tarea nueva de documentación |

## Orden de ejecución

1. `error.rs`: variantes nuevas + mapeos.
2. `models/conteo.rs`: validación pura.
3. Tests unitarios e integración **en rojo** (T-V1, T-V2).
4. `services/conteos.rs`: quitar `unwrap`, códigos nuevos, alta sobre depósito vacío.
5. Verde de backend + `cargo sqlx prepare --check`.
6. Frontend: schema test + ampliar cliente + pantalla.
7. `docs/api.md` y alineación de `contracts/api.md`.
8. Gates completos.

## Riesgos

- **Alto, datos**: el cierre muta stock en todas las líneas con diff; toda la escritura en una
  transacción y con `FOR UPDATE`.
- **Concurrencia**: doble cierre/cancelación simultáneos resueltos por el bloqueo de fila.
- **Regresión de códigos**: cambiar `validacion` por códigos específicos puede romper clientes que
  comparan el código; el único consumidor es la SPA de este repo y el contrato se actualiza junto.

## Complexity Tracking

Sin violaciones que justificar: no se agregan proyectos, capas ni dependencias. El servicio
dedicado `services::conteos.rs` ya estaba justificado por la atomicidad del cierre multi-línea.
