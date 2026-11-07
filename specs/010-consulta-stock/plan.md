# Implementation Plan: Consulta y tablero de stock (010)

**Branch**: `010-consulta-stock` | **Date**: 2026-11-07 | **Spec**: [spec.md](spec.md)

**Input**: `specs/010-consulta-stock/spec.md` (clarificado el 2026-11-07) y hallazgos de
`/speckit analyze` del mismo día (G1, G2, I1, U1, S1, C1, G3, I2, K1, K2, U2, D1).

## Summary

Completar la consulta de stock: el backend de 007 (FR-029) ya lista y detalla, pero le faltan
`faltante`, el valorizado a costo **filtrado por perfil en el backend** y un filtro por ids para
la caja. El frontend necesita una `/admin/stock` completa (filtros, faltante, valorizado, desglose)
y un selector de depósito con stock disponible en `/caja`. SC-018 se mide antes de decidir un índice
nuevo. El plan anterior ("sin cambios de backend") queda reemplazado: era incompatible con FR-040.

## Technical Context

**Language/Version**: Rust (edición del workspace) + TypeScript (SvelteKit 2 / Svelte 5 runes)

**Primary Dependencies**: Axum, SQLx 0.8 (`rust_decimal` con `serde-float`), utoipa; Zod, Vitest,
Tailwind. Sin dependencias nuevas.

**Storage**: PostgreSQL. Tablas: `productos` (`costo NUMERIC(12,2)`, `stock_actual INT`,
`stock_minimo INT`), `stock_por_deposito`, `depositos`, `ubicaciones`, `categorias`. Verificado
contra `migrations/0002`, `0007` y `0009`. **Pendiente**: verificar contra la BD real con el MCP de
PostgreSQL antes de T003 (Principio V).

**Testing**: `cargo test` (unit + integración HTTP en `backend/tests/`), `cargo sqlx prepare --check`;
`npm test` (Vitest), `npm run check`, `npm run build` en `frontend/`; `npm test` en `asistente/`.

**Target Platform**: servidor Linux/Windows + navegador (SPA).

**Project Type**: web (backend + frontend + asistente).

**Performance Goals**: SC-018, página de `GET /api/stock` < 1 s con 10.000 productos y 3 depósitos.

**Constraints**: sin migración salvo que la medición de SC-018 falle; `costo`/`valorizado` nunca
llegan a una sesión `OPERACION`.

**Scale/Scope**: 2 endpoints extendidos, 2 pantallas, 1 helper de frontend.

## Constitution Check

*GATE: antes de Phase 0 y otra vez después de Phase 1.*

| Principio | Estado | Cómo se cumple |
|---|---|---|
| I. Simplicidad | ✅ | Se extienden los endpoints existentes; `producto_ids` evita N llamadas de detalle desde la caja sin crear un endpoint nuevo. Índice solo si la medición lo exige. |
| II. Mantenibilidad | ✅ | Acceso HTTP solo por `$lib/api/stock.ts`; cálculo de `faltante`/valorizado en funciones puras del modelo; las pantallas solo presentan. |
| III. Mensajes | ✅ | Errores de carga con qué falló + qué hacer + Reintentar (casos borde del spec). Hallazgo K1 corregido. |
| IV. Testabilidad | ✅ | Test unitario de `StockItem::desde_fila` (faltante y costo por perfil); test HTTP por perfil; tests Vitest de `stock.ts`, del schema y del helper de pantalla; RED antes de GREEN. |
| V. Integridad | ✅ | Solo lectura, sin transacciones de escritura. Esquema verificado antes de tocar SQL/`FromRow`. |
| Seguridad chatbot | ✅ | El asistente consulta con la sesión del usuario: hereda el filtrado de costo. Se verifica que su parser tolere los campos nuevos. |

Sin violaciones: no hace falta Complexity Tracking.

## Project Structure

### Documentation (this feature)

```text
specs/010-consulta-stock/
├── spec.md
├── plan.md            # este archivo
├── research.md        # decisiones R1–R6
├── data-model.md      # forma de StockItem / StockProducto
├── quickstart.md      # validación de punta a punta
├── contracts/api.md   # contrato extendido de /api/stock
└── tasks.md
```

### Source Code

```text
backend/
├── src/models/stock.rs          # StockFila + costo; StockItem + faltante/costo/valorizado
├── src/services/stock.rs        # listar(ver_costo, producto_ids); detalle con faltante
├── src/handlers/stock.rs        # lee familia_rol del AuthUsuario; query producto_ids
└── tests/
    ├── unit/test_stock_item.rs              # nuevo
    └── integration/test_stock_http.rs       # nuevo: perfiles, producto_ids, faltante, SC-018
frontend/src/
├── lib/schemas/movimientos.ts   # StockItemSchema/StockProductoSchema extendidos
├── lib/api/stock.ts             # productoIds en ListarStockParams
├── lib/api/stock.test.ts        # nuevo
├── lib/utils/stock.ts           # nuevo: indexar stock por producto, formato de valorizado
├── lib/utils/stock.test.ts      # nuevo
├── routes/admin/stock/+page.svelte
└── routes/caja/+page.svelte
asistente/src/tools.ts           # solo verificación de tolerancia a campos nuevos
docs/api.md                      # sección de stock
```

**Structure Decision**: web application existente (`backend/`, `frontend/`, `asistente/`); no se crean
carpetas nuevas.

## Fases

- **Phase 0 — research.md**: R1 depósito de caja, R2 valorizado por perfil, R3 alcance del listado,
  R4 `producto_ids` para la caja, R5 `faltante`, R6 medición de SC-018.
- **Phase 1 — diseño**: `data-model.md`, `contracts/api.md`, `quickstart.md`.
- **Re-check post-diseño**: sin cambios en la tabla de la constitución.

## Orden de ejecución

1. Verificar el esquema real (MCP PostgreSQL).
2. Backend con TDD: unit de `StockItem` → modelo/servicio/handler → tests HTTP por perfil.
3. `cargo sqlx prepare` y docs.
4. Frontend: schema y cliente con tests → helper → `/admin/stock` → `/caja`.
5. Asistente: confirmar que el parser tolera los campos nuevos.
6. SC-018: sembrar 10.000 productos, `EXPLAIN ANALYZE` + prueba HTTP; decidir índice.
7. Quality gates.
