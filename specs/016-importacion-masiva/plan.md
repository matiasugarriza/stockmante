# Plan de implementación: Importación masiva de productos y stock (016)

**Branch**: `016-importacion-masiva` | **Date**: 2026-11-07 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/016-importacion-masiva/spec.md` (clarificada 2026-11-07).

## Summary

Endpoint `POST /api/importaciones/productos` que recibe un CSV por `multipart/form-data` y opera en
dos modos: `validar` (dry-run, 200 con resumen y errores) y `aplicar` (201, atómico). En `aplicar`
crea productos y, si `stock_inicial` > 0, un movimiento `ENTRADA` con motivo `IMPORTACION`, todo en
una única transacción SQLx. Cualquier error de fila en `aplicar` responde 422
`importacion_invalida` con `detalles` por fila/columna y no escribe nada. Pantalla
`/admin/importacion` para subir el archivo, validar y aplicar, mostrando el reporte.

La implementación existente (`services/importaciones.rs`, `handlers/importaciones.rs`) es un punto de
partida parcial: aplica en transacción, pero no cumple dialecto CSV, contrato de error, validación de
depósito condicionada al stock, mensajes de duplicados con filas, validadores de 003, OpenAPI,
documentación ni frontend. Este plan corrige y completa esas brechas.

## Technical Context

**Language/Version**: Rust edition 2021 (backend); Svelte 5 + TypeScript 6 (frontend).

**Primary Dependencies**: Axum 0.8 (`multipart`), SQLx 0.8, `csv` 1 (ya declarada y justificada por
spec), `rust_decimal`, `utoipa` 6; SvelteKit 3, Tailwind 4, Zod 4, Vitest 4.

**Storage**: PostgreSQL. Tablas existentes: `productos`, `categorias`, `depositos`,
`stock_por_deposito`, `movimientos`, `movimiento_lineas`. Sin migración nueva (0002 y 0009 cubren el
esquema).

**Testing**: `cargo test` (targets unitarios e integración declarados en `backend/Cargo.toml`),
`cargo sqlx prepare --check`, `npm test`, `npm run check`, `npm run build`.

**Target Platform**: API server + SPA web.

**Project Type**: Web application (`backend/` + `frontend/`).

**Performance Goals**: 5.000 filas válidas aplicadas en < 60 s (SC-024); archivo ≤ 5 MB.

**Constraints**: Aplicación atómica; CSV UTF-8 con dialecto autodetectado; sin Excel; sin migración;
sin dependencias nuevas; errores de fila sin abortar el dry-run; `GESTION` obligatorio.

**Scale/Scope**: Hasta 5.000 filas por archivo; catálogo de decenas de miles de productos.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principio | Evaluación | Evidencia / mitigación |
|-----------|------------|------------------------|
| I. Simplicidad | PASS | Se reutiliza `csv` ya justificado en spec/Cargo; no se agregan capas ni patrones nuevos. El parser y el validador puro se separan de la I/O solo para testear (P-IV). |
| II. Mantenibilidad | PASS | Lógica en `services/importaciones.rs`; router delgado en `handlers/importaciones.rs`; frontend con `$lib/api/importaciones.ts` tipado y página pequeña; sin `panic!/unwrap/expect`. |
| III. Claridad de mensajes | PASS | Mensajes exactos de la spec; sobre `{ error: { codigo, mensaje, detalles } }`; 200 con resumen en validar y 422 con detalle en aplicar. |
| IV. Testabilidad | PASS | Parser/validación sin I/O, testeable con strings; tests unitarios + integración HTTP; sin reloj/red ocultos. RED antes de GREEN. |
| V. Integridad de datos | PASS | Una transacción (`pool.begin()`); `stock_por_deposito` fuente de verdad; `productos.stock_actual` actualizado en la misma transacción; `CHECK >= 0`; esquema verificado contra migraciones 0002/0009. |
| Flujo SDD | PASS | Spec clarificada, esquema verificado, plan y tasks antes de código. |

Sin violaciones que justifiquen `Complexity Tracking`.

## Project Structure

### Documentation (this feature)

```text
specs/016-importacion-masiva/
├── spec.md              # Clarificada 2026-11-07
├── plan.md              # Este archivo
├── research.md          # Fase 0: decisiones técnicas
├── data-model.md        # Fase 1: entidades, validaciones y DTOs
├── quickstart.md        # Fase 1: validación end-to-end
├── contracts/api.md     # Fase 1: contrato HTTP y formato CSV
└── tasks.md             # Fase 2 (/speckit-tasks)
```

### Source Code (repository root)

```text
backend/
├── src/
│   ├── error.rs                     # Variante ImportacionInvalida + detalles
│   ├── docs.rs                      # Registrar path y schemas en OpenAPI
│   ├── handlers/
│   │   └── importaciones.rs         # Multipart, query modo, mapeo 200/201/422
│   ├── models/
│   │   └── producto.rs              # Reutiliza validar_sku/unidad/nombre/monto/codigo_barras
│   └── services/
│       └── importaciones.rs         # Parser CSV + validación por fila + transacción
└── tests/
    ├── unit/test_importacion_validacion.rs
    └── integration/test_importacion_http.rs
    (registrar ambos en backend/Cargo.toml)

frontend/
├── src/
│   ├── lib/
│   │   ├── api/client.ts            # FormData + ApiError.detalles
│   │   ├── api/importaciones.ts     # Tipado y test
│   │   ├── api/importaciones.test.ts
│   │   └── schemas/importaciones.ts # Zod del resumen/error
│   └── routes/
│       ├── admin/importacion/+page.svelte
│       └── +layout.svelte           # Enlace "Importar" para GESTION
docs/api.md                          # Sección de importación
```

**Structure Decision**: Monorepo web ya existente; backend Rust/Axum con separación
`handlers`/`services`/`models` y frontend SvelteKit con clientes API centralizados en
`frontend/src/lib/api/`.

## Phase 0 — Investigación

Salida: [research.md](./research.md). Decisiones que resuelven los `NEEDS CLARIFICATION`:

1. **Dialecto CSV**: autodetección `,`/`;` por la primera línea; decimal `,` si el delimitador es `;`,
   `.` si es `,`; comillas dobles RFC 4180; BOM UTF-8 tolerado; sin separador de miles.
2. **Atomicidad**: una sola transacción para todo el archivo en `aplicar`.
3. **Contrato de error**: nueva variante `AppError::ImportacionInvalida(Vec<DetalleImportacion>)`
   con 422 y código `importacion_invalida`; el sobre agrega `detalles`.
4. **Validación**: reutilizar `validar_sku`, `validar_nombre_producto`, `validar_unidad`,
   `validar_monto`, `validar_codigo_barras` de 003, traduciendo cada error a `(fila, columna)`.
5. **Duplicados**: prefetch de SKU/códigos existentes; duplicados intra-archivo con número de fila
   de la primera aparición.
6. **Depósito**: solo se valida si `stock_inicial` > 0; default "Principal" por nombre.
7. **Performance**: prefetch de categorías/depósitos, un `INSERT ... RETURNING id` por fila y
   actualización de stock en la misma transacción; sin consultas por fila fuera de la tx.
8. **Frontend**: subida multipart sin `Content-Type` manual, dry-run primero, bloqueo de "Aplicar"
   si hay errores, render de `detalles`.

## Phase 1 — Diseño y contratos

Salidas:

- [data-model.md](./data-model.md): entidades existentes, DTOs, reglas de validación y mapeo a la BD.
- [contracts/api.md](./contracts/api.md): endpoint, permisos, request/response, errores, formato CSV.
- [quickstart.md](./quickstart.md): comandos y escenarios ejecutables (dry-run, aplicar, duplicados,
  límites, 403, 5.000 filas).

**Post-design Constitution Check**: se re-evalúa tras generar los artefactos; sin cambios en los
gates (PASS).

## Complexity Tracking

Sin violaciones; no aplica.
