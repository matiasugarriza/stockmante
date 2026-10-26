# Implementation Plan: Catálogo de productos y categorías

**Branch**: `main` | **Date**: 2026-10-26 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/003-catalogo-productos/spec.md`

**Estado**: plan v2 — reemplaza el plan del WIP deshecho (2026-10-25). Diffs respecto del anterior:
sin `DELETE` en ningún caso, `activo` fuera del `PUT`, `descripcion` de categoría, filtros `q` y
`categoria_id`, endpoints `/reactivar`, migración `0006` en lugar de `0004`.

## Summary

CRUD completo de categorías planas (nombre, descripción, estado) y de productos (nombre, SKU
normalizado y único global, código de barras opcional único, categoría, unidad de medida, costo,
precio, stock mínimo), con listado paginado y filtrable por texto, categoría y estado. El retiro es
**siempre** baja lógica (`activo = FALSE`): no existe borrado físico ni ruta `DELETE`. La escritura
queda bajo `RequiereGestion`; la lectura de catálogo queda disponible para ambas familias. Backend
Rust/Axum/SQLx + frontend SvelteKit SPA/Tailwind, alineado a la constitución.

## Technical Context

**Language/Version**: Rust edition 2021 (axum 0.8, sqlx 0.8); TypeScript 6, SvelteKit 3, Svelte 5
(modo runes forzado), Vite 8, Tailwind 4, Vitest 4.

**Primary Dependencies**: axum, axum-extra (cookies), sqlx (`postgres`, `chrono`, `uuid`, `migrate`),
tokio, serde, thiserror, `rust_decimal` (nueva, con `serde-float`) + feature `rust_decimal` de SQLx,
Zod 4, `@sveltejs/adapter-static`.

**Storage**: PostgreSQL 18. Tablas `categorias` y `productos` creadas en `0002` (propiedad de esta
spec) + migración nueva `0006_catalogo_descripcion_codigo_barras.sql`.

**Testing**: `cargo test` (targets unitarios + integración HTTP contra `stockmante_test`), `cargo
sqlx prepare --check`, `npm test` (vitest), `npm run check`, `npm run build`.

**Target Platform**: servidor Linux/Windows con PostgreSQL 18; SPA servida como estáticos.

**Project Type**: aplicación web (backend `backend/` + frontend `frontend/`).

**Performance Goals**: listado de gestión paginado (20 por defecto, máximo 100) sobre catálogos de
miles de productos; sin objetivos de latencia propios (los de caja son de 004).

**Constraints**: sin `unwrap`/`panic!`/`expect` en producción; toda falla como `Result<T, AppError>`;
unicidad resuelta por índice (mapeo `23505`), no por pre-chequeo; importes `NUMERIC(12,2)` con
`rust_decimal`; alias de imports frontend `#lib`, no `$lib`.

**Scale/Scope**: ~10³ productos por instalación; 2 tablas, 14 endpoints, 2 pantallas de
administración.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Gate (Principio) | Estado | Evidencia / decisión |
|---|---|---|
| I — Simplicidad / YAGNI | ✅ PASS | Categorías planas; sin jerarquía, sin tabla de historial propia, sin capa de repositorios. Se agrega **una** dependencia (`rust_decimal`) justificada por el contrato numérico. Se elimina `AppError::ProductoConReferencias`, que queda sin uso. |
| II — Mantenibilidad | ✅ PASS | `services/categorias.rs` y `services/productos.rs` con la lógica; handlers delgados; `parsear_activo` centralizado en `handlers/mod.rs`. Frontend: `#lib/api/*.ts` + `#lib/schemas/*.ts`, sin `fetch` en componentes. |
| III — Claridad en Mensajes | ✅ PASS | Códigos específicos: 409 `sku_duplicado`, `codigo_barras_duplicado`, `categoria_duplicada`; 422 `validacion` con mensaje por campo en español; 405 sin cuerpo (método inexistente, ver R15). |
| IV — Testabilidad Primero | ✅ PASS | RED→GREEN obligatorio (sin waiver desde 003). Dominio puro testeable sin BD; integración HTTP; vitest de schemas y clientes. |
| V — Integridad de Datos / Errores | ✅ PASS | Unicidad por índice; baja lógica conserva filas e IDs; `UPDATE … RETURNING` atómico por sentencia; errores de dominio mapeados a `AppError`, infraestructura a 500 genérico. Sin movimientos de stock en esta spec. |

**Resultado pre-diseño**: PASS, sin violaciones.

## Project Structure

### Documentation (this feature)

```text
specs/003-catalogo-productos/
├── plan.md              # Este archivo
├── research.md          # Fase 0
├── data-model.md        # Fase 1 (ya reescrito en el replanteo)
├── quickstart.md        # Fase 1
├── contracts/
│   ├── api.md
│   └── validation.md
├── spec.md
├── tasks.md             # Fase 2 (/speckit.tasks) — generado, T001–T037 marcadas
└── progress.md          # Bitácora de la implementación v2 y su evidencia
```

### Source Code (repository root)

```text
backend/src/
├── models/producto.rs          # Categoria, CategoriaBreve, Producto, ListaProductos
├── services/categorias.rs      # validación, CRUD, baja/reactivación
├── services/productos.rs       # validación, CRUD, filtros, baja/reactivación
├── handlers/categorias.rs      # router /api/categorias
├── handlers/productos.rs       # router /api/productos
├── handlers/mod.rs             # parsear_activo(valor, por_defecto) pub(crate)
├── error.rs                    # + CategoriaDuplicada; − ProductoConReferencias
└── app.rs                      # nest de los dos routers en el grupo autenticado

backend/tests/
├── unit/test_sku.rs
├── unit/test_producto_validacion.rs
├── unit/test_categoria_normalizacion.rs
├── integration/test_categorias_http.rs
├── integration/test_productos_http.rs
└── integration/test_catalogo_baja_logica.rs

frontend/src/
├── lib/schemas/categorias.ts
├── lib/schemas/productos.ts
├── lib/api/categorias.ts
├── lib/api/productos.ts
├── lib/components/categorias/{CategoriaForm,CategoriaTabla}.svelte
├── lib/components/productos/{ProductoForm,ProductoTabla,FiltrosCatalogo}.svelte
└── routes/admin/{categorias,productos}/+page.svelte

migrations/
└── 0006_catalogo_descripcion_codigo_barras.sql
```

**Structure Decision**: se mantiene el layout existente de 001/002 (`models` / `services` /
`handlers` por módulo, router propio nestado en `app.rs`). No se introducen capas nuevas.

## Diseño de la API

Router: `handlers::categorias::router()` y `handlers::productos::router()` nestados bajo
`/api/categorias` y `/api/productos` dentro del grupo `autenticado` de `app.rs` (gate de contraseña
temporal FR-008a).

| Método | Ruta | Guard | Éxito | Notas |
|---|---|---|---|---|
| GET | `/api/categorias` | `AuthUsuario` | 200 `[Categoria]` | orden por nombre ascendente; `?activo` default `true` |
| POST | `/api/categorias` | `RequiereGestion` | 201 | `{nombre, descripcion?}` |
| GET | `/api/categorias/{id}` | `AuthUsuario` | 200 | incluye inactivas |
| PUT | `/api/categorias/{id}` | `RequiereGestion` | 200 | `activo` → 422; `{}` → 422 |
| DELETE | `/api/categorias/{id}` | — | 405 | **sin ruta**: Axum responde 405 + `Allow` |
| POST | `/api/categorias/{id}/desactivar` | `RequiereGestion` | 204 | idempotente |
| POST | `/api/categorias/{id}/reactivar` | `RequiereGestion` | 204 | idempotente |
| GET | `/api/productos` | `RequiereGestion` | 200 `ListaProductos` | `?q=&categoria_id=&activo=&pagina=&por_pagina=` |
| POST | `/api/productos` | `RequiereGestion` | 201 | |
| GET | `/api/productos/{id}` | `AuthUsuario` | 200 | incluye inactivos |
| PUT | `/api/productos/{id}` | `RequiereGestion` | 200 | `activo` → 422; `codigo_barras: ""` limpia |
| DELETE | `/api/productos/{id}` | — | 405 | **sin ruta** |
| POST | `/api/productos/{id}/desactivar` | `RequiereGestion` | 204 | idempotente |
| POST | `/api/productos/{id}/reactivar` | `RequiereGestion` | 204 | idempotente |

Fuera de este plan: `GET /api/productos/buscar` (004) y la asignación de ubicaciones (005).

## Migración y esquema (0006)

Una sola migración con cuatro sentencias, sin datos:

1. `categorias`: agregar `descripcion VARCHAR(255) NULL`.
2. `categorias`: crear índice único funcional `uq_categorias_nombre_normalizado` sobre
   `(lower(btrim(nombre)))`. La `UNIQUE (nombre)` de `0002` **se conserva** como guarda exacta
   redundante: quitarla no agrega integridad y sí churn de migración.
3. `productos`: reemplazar `UNIQUE NULLS NOT DISTINCT (codigo_barras)` por `UNIQUE (codigo_barras)`.
4. `productos`: crear `idx_productos_categoria` sobre `(categoria_id)`. La `0002` nunca lo creó.

Antes de aplicarla: verificar el nombre real del constraint en cada base (constitución —
Verificación de esquema). `0004` no se reutiliza (número quemado por el WIP) y `0005` ya está
ocupado por `0005_sesion_unica.sql`, por eso `0006`.

## Decisiones de implementación

- **Dinero**: `rust_decimal::Decimal` con `serde-float`; SQLx con feature `rust_decimal`. Se
  serializa como número JSON, nunca string.
- **SKU**: `normalizar_sku` (trim + mayúsculas) → `validar_sku` (3–30, `^[A-Z0-9][A-Z0-9-]{2,29}$`)
  con mensajes accionables. Único global, inactivos incluidos.
- **Código de barras**: `trim`; vacío/blanco → `NULL`; 8–64 alfanuméricos; único entre los que lo
  informan.
- **Unicidad**: mapeo de `23505` por nombre de constraint (`productos_sku_key`,
  `productos_codigo_barras_key`, `uq_categorias_nombre_normalizado` y, como respaldo,
  `categorias_nombre_key`) sin pre-chequeo.
- **Categoría destino**: dentro de la transacción del alta/edición, `SELECT activo FROM categorias
  WHERE id = $1` → 422 si no existe o está inactiva; `23503` como red de seguridad. La transacción
  evita asignar una categoría que se desactiva entre el chequeo y la escritura.
- **Baja lógica**: `UPDATE … SET activo = $2, actualizado_en = now() WHERE id = $1 RETURNING …`;
  0 filas → 404. Idempotente por naturaleza (no hay 409).
- **`PUT` parcial**: `UPDATE … SET campo = COALESCE($n, campo) … RETURNING`; cuerpo vacío → 422
  antes de tocar la base; `activo` presente → 422.
- **`parsear_activo(valor, por_defecto)`** en `handlers/mod.rs`: usuarios pasa `None` (ausente =
  todas), catálogo pasa `Some(true)` (ausente = solo activos). El comportamiento de usuarios se
  conserva y se cubre con un test de regresión.
- **405**: no se registra `DELETE` en ninguna ruta; Axum responde 405 con `Allow: GET, HEAD, PUT` y sin
  cuerpo. Cero código, semántica HTTP estándar (R15).
- **`DELETE` en el frontend**: no existe. Los componentes usan `/desactivar` y `/reactivar`.
- **`AppError`**: se agrega `CategoriaDuplicada` (409 `categoria_duplicada`); se elimina
  `ProductoConReferencias` por quedar sin uso y se ajusta `test_apperror`.
- **Frontend**: alias `#lib` (SvelteKit 3), runes, `adapter-static`; schemas Zod espejo del backend;
  errores del API mapeados por campo.
- **`stock_actual`**: no se agrega. La fuente de stock es de módulos posteriores; el catálogo solo
  persiste `stock_minimo`.

## Estrategia de pruebas

| Capa | Enfoque |
|---|---|
| Dominio | SKU (normalización + regex), código de barras, montos, nombre de categoría normalizado |
| Servicios | Unicidad, categoría inactiva/inexistente, baja/reactivación idempotente, filtros combinados |
| Integración HTTP | 201/200/204/401/403/404/405/409/422 por endpoint; `DELETE` no altera la fila |
| Invariantes | Un rechazo no persiste ni modifica filas; N productos con `codigo_barras = NULL` conviven; bajar no cambia conteos |
| Regresión | `GET /api/usuarios` sin `activo` sigue devolviendo todos tras centralizar `parsear_activo` |
| Frontend | Zod de categoría/producto; cliente API con `ApiError`; filtros y paginación |
| SQLx | `cargo sqlx prepare --check` sin diferencias |

Convenciones: claves únicas derivadas de UUID para tests en paralelo; RED→GREEN por regla de
dominio; datos de prueba sin limpieza destructiva.

## Riesgos y mitigaciones

| Riesgo | Mitigación |
|---|---|
| Drift local de `codigo_barras` (constraint aplicado a mano, sin fila en `_sqlx_migrations`) | La migración verifica el nombre real antes de reemplazar; `\d productos` como fuente de verdad |
| Índice funcional nuevo choca con nombres existentes que solo difieren en mayúsculas | La migración falla ruidosamente; se documenta como precondición (base vacía en la práctica) |
| Búsqueda por texto sin índice trigram | Aceptado para escala de gestión; el trigram es de 004 y no se adelanta |
| `ProductoConReferencias` eliminado rompe `test_apperror` | Se actualiza el test en la misma unidad de trabajo |
| `parsear_activo` compartido cambia el default de usuarios | Firma con `por_defecto` explícito + test de regresión |

## Fases

1. **T001–T002** (RED): tests de normalización/validación de SKU, código de barras, montos y nombre
   de categoría; tests de unicidad y baja lógica.
2. **T003**: migración `0006` + modelos + servicios de categorías y productos.
3. **T004**: handlers, `parsear_activo`, routers, `[[test]]` e integración HTTP.
4. **T005**: frontend (schemas, clientes, rutas, formularios, filtros).
5. Gates completos (`cargo test`, `cargo sqlx prepare --check`, `npm test`, `npm run check`,
   `npm run build`) y regeneración de `tasks.md`/`progress.md`.

## Complexity Tracking

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| _(ninguna)_ | — | Constitution Check PASS sin deudas. |

## Constitution Check post-diseño

Re-evaluado tras Fase 1: PASS. El diseño no agrega capas ni dependencias más allá de
`rust_decimal` (justificada en R9); no hay borrado físico que comprometa integridad; toda regla de
negocio tiene un test determinista asignado; los mensajes de error son específicos y accionables.
