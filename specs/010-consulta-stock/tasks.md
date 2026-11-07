# Tasks: 010-consulta-stock

**Input**: Design documents from `/specs/010-consulta-stock/`

**Prerequisites**: plan.md ✓, spec.md ✓ (clarificado 2026-11-07), research.md ✓, data-model.md ✓,
contracts/api.md ✓, quickstart.md ✓

**Tests**: obligatorios (Principio IV). Cada test se escribe antes de su implementación y se
registra el RED.

**Historial**: la versión anterior (T001–T004, marcadas `[X]`) se reemplaza. `/speckit analyze` del
2026-11-07 encontró que `/admin/stock` no tenía valorizado, filtro de categoría ni `activo`, y que
`/caja` no tenía panel de stock (hallazgos G1, G2). Lo que ya existe en esas pantallas se reutiliza.

## Format: `[ID] [P?] [Story] Description`

## Path Conventions

- Backend: `backend/src/`, tests en `backend/tests/{unit,integration}/` registrados en `backend/Cargo.toml`
- Frontend: `frontend/src/`
- Asistente: `asistente/src/`

---

## Phase 1: Setup

- [X] T001 Verificar con el MCP de PostgreSQL (o `sqlx-cli`) las columnas y nullables de `productos` (`costo`, `stock_actual`, `stock_minimo`), `stock_por_deposito`, `depositos`, `ubicaciones` y `categorias` contra `specs/010-consulta-stock/data-model.md`; anotar diferencias en el data-model antes de seguir

---

## Phase 2: Foundational (bloquea US10)

- [X] T002 [US10] Escribir `backend/tests/unit/test_stock_item.rs` (registrarlo en `backend/Cargo.toml`): `faltante` = max(mínimo − stock, 0) con mínimo 0, stock mayor y menor; `costo`/`valorizado` en `None` si `ver_costo = false`; `valorizado` usa `stock_deposito` si existe y si no `stock_actual`, redondeado a 2 decimales. Registrar el RED
- [X] T003 [US10] En `backend/src/models/stock.rs`: agregar `costo: Decimal` a `StockFila`; agregar `faltante`, `costo: Option<Decimal>`, `valorizado: Option<Decimal>` a `StockItem` y `faltante` a `StockProducto`; reemplazar `From<StockFila>` por `StockItem::desde_fila(fila, ver_costo)` (depende de T001, T002 → GREEN)

---

## Phase 3: User Story 10 - Ver cuánto hay y dónde está (Priority: P1) 🎯 MVP

**Goal**: listado y detalle completos con valorizado por perfil, `/admin/stock` completa y stock
por depósito en `/caja`.

**Independent Test**: seguir `specs/010-consulta-stock/quickstart.md` con una sesión `GESTION` y otra
`OPERACION`.

### Tests backend (escribir primero)

- [X] T004 [US10] Crear `backend/tests/integration/test_stock_http.rs` (registrarlo en `backend/Cargo.toml`): `GESTION` recibe `costo`/`valorizado` numéricos y `OPERACION` los recibe `null` (escenario 10.7); `bajo_minimo=true` trae `faltante` (10.4); `producto_ids` devuelve solo esos ids con `stock_deposito`; `producto_ids` con 51 ids o no numérico → 422 con mensaje específico; el detalle trae `faltante` y su desglose suma `stock_actual`; id inexistente → 404 `"Producto no encontrado."` (10.5). Registrar el RED

### Implementación backend

- [X] T005 [US10] En `backend/src/services/stock.rs`: agregar `producto_ids: Option<Vec<i32>>` y `ver_costo: bool` a `FiltrosStock`; sumar `p.costo` al SELECT y `($8::int[] IS NULL OR p.id = ANY($8))` al listado y al `count`; usar `StockItem::desde_fila`; calcular `faltante` en `detalle` (depende de T003)
- [X] T006 [US10] En `backend/src/handlers/stock.rs`: leer `AuthUsuario(usuario)` y pasar `ver_costo = usuario.familia_rol == "GESTION"`; parsear `producto_ids` (coma, 1–50 enteros) devolviendo `AppError::Validacion` con mensaje específico; actualizar la documentación utoipa (depende de T005 → T004 en GREEN)
- [X] T007 [US10] Correr `cargo sqlx prepare` en `backend/` y actualizar la sección de stock de `docs/api.md` (parámetro `producto_ids`, campos `faltante`/`costo`/`valorizado`, regla de perfil) (depende de T006)

### Frontend: cliente y lógica (tests primero)

- [X] T008 [P] [US10] Agregar casos a `frontend/src/lib/schemas/movimientos.test.ts`: `StockItemSchema` acepta `faltante`, `costo`/`valorizado` numéricos o `null`; `StockProductoSchema` acepta `faltante`. Registrar el RED
- [X] T009 [P] [US10] Crear `frontend/src/lib/api/stock.test.ts`: `listarStock` serializa `categoria_id`, `activo` y `producto_ids` (coma); omite `producto_ids` vacío; `obtenerStockProducto` propaga el 404. Registrar el RED
- [X] T010 [P] [US10] Crear `frontend/src/lib/utils/stock.test.ts`: `indexarPorProducto(items)` → mapa `producto_id → stock_deposito`; `elegirDeposito(guardado, activos)` descarta un id inactivo y toma el primero; `textoValorizado(item)` muestra "—" cuando `valorizado` es `null`. Registrar el RED
- [X] T011 [US10] Extender `StockItemSchema` y `StockProductoSchema` en `frontend/src/lib/schemas/movimientos.ts` (depende de T008)
- [X] T012 [US10] Agregar `productoIds?: number[]` a `ListarStockParams` en `frontend/src/lib/api/stock.ts` (depende de T009, T011)
- [X] T013 [US10] Crear `frontend/src/lib/utils/stock.ts` con `indexarPorProducto`, `elegirDeposito` y `textoValorizado` (depende de T010, T011)

### Frontend: pantallas

- [X] T014 [US10] Completar `frontend/src/routes/admin/stock/+page.svelte`: filtros de categoría (`listarCategorias`) y activo (activos/inactivos/todos); columnas stock global, stock en depósito (si hay depósito elegido), mínimo, faltante, costo y valorizado; fila clickeable que abre el desglose con `obtenerStockProducto`; reiniciar a página 1 al cambiar filtros; errores con qué falló, qué hacer y botón Reintentar, sin descartar en silencio el fallo de depósitos (FR-040, K1; depende de T012, T013)
- [X] T015 [US10] En `frontend/src/routes/caja/+page.svelte`: selector de depósitos activos recordado en `localStorage` vía `elegirDeposito`; tras cada búsqueda, `listarStock({ productoIds, depositoId, porPagina: 50 })` y columna "Stock en <depósito>"; respetar la secuencia de turnos existente para descartar respuestas viejas; mensaje cuando no hay depósitos activos; sin costo ni valorizado (escenario 10.6, R1, R4; depende de T012, T013)
- [X] T016 [US10] Verificar que el enlace "Stock" de `frontend/src/routes/+layout.svelte` sigue visible solo para `GESTION` y que el aviso de bajo mínimo sigue funcionando con el schema extendido (depende de T011)

### Asistente

- [X] T017 [P] [US10] Verificar en `asistente/src/tools.ts` y `asistente/src/pruebas/api-falsa.ts` que la herramienta de stock tolera `faltante`, `costo` y `valorizado` (null o numéricos); ajustar el fixture y correr `npm test` en `asistente/` (depende de T006)

---

## Phase 4: Polish & cross-cutting

- [X] T018 [US10] Agregar a `backend/tests/integration/test_stock_http.rs` el test `sc018` con `#[ignore]`: sembrar 10.000 productos y 3 depósitos, medir por HTTP una página sin `q`, con `q` por nombre y con `q` por SKU, e imprimir `EXPLAIN ANALYZE`; registrar los tiempos en `specs/010-consulta-stock/research.md` (R6) (depende de T006)
- [X] T019 *(no aplica: SC-018 ≈ 37 ms, ver research R6)* [US10] Solo si T018 supera 1 s en alguna variante: crear `migrations/0015_productos_sku_trgm.sql` con el índice GIN trigram sobre `productos.sku`, repetir T018 y actualizar el spec (sección Dependencias) y el ledger de migraciones de `specs/README.md` (depende de T018)
- [X] T020 Quality gates: `cargo test` y `cargo sqlx prepare --check` en `backend/`; `npm test`, `npm run check` y `npm run build` en `frontend/`; `npm test` en `asistente/`; sin `unwrap`/`expect`/`panic!` en el código nuevo de producción
- [ ] T021 Recorrer `specs/010-consulta-stock/quickstart.md` (pasos manuales 1–5) y actualizar el estado del spec a Implementado

---

## Dependencies & Execution Order

- T001 → T002 → T003 → T004 → T005 → T006 → T007
- T008, T009, T010 en paralelo desde el inicio (solo dependen del contrato)
- T011 → T012, T013 → T014, T015 (T014 y T015 tocan archivos distintos: paralelizables)
- T016 después de T011; T017 después de T006
- T018 después de T006; T019 solo si T018 falla
- T020 y T021 al final

## Parallel Example

```text
Mientras el backend avanza (T002–T006):
  T008 movimientos.test.ts  |  T009 stock.test.ts  |  T010 utils/stock.test.ts
Con T012 y T013 listos:
  T014 admin/stock  |  T015 caja
```

## Implementation Strategy

1. **MVP**: T001–T007 (API con valorizado por perfil y `faltante`). La seguridad de FR-040 queda
   cerrada aunque la UI todavía no muestre el valorizado.
2. **Incremento 1**: T008–T014 (`/admin/stock` completa).
3. **Incremento 2**: T015–T017 (caja y asistente).
4. **Cierre**: T018–T021 (SC-018, gates, quickstart).

## Trazabilidad

| Requisito / hallazgo | Tareas |
|---|---|
| FR-038 | T004, T005, T006, T009, T012, T014 |
| FR-039 | T003, T004, T005, T014 |
| FR-040 / S1 / I1 | T002–T006, T014, T015 |
| SC-018 / G3 / I2 | T018, T019 |
| U1 (depósito de caja) | T010, T013, T015 |
| U2 (faltante) | T002, T003, T004 |
| K1 (mensajes) | T014, T015 |
| K2 (tests frontend) | T008, T009, T010 |
