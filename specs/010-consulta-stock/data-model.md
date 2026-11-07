# Data Model: Consulta y tablero de stock (010)

Sin tablas nuevas. Columnas leídas (verificadas contra `migrations/`; falta confirmar contra la BD
real con el MCP de PostgreSQL):

| Tabla | Columnas | Origen |
|---|---|---|
| `productos` | `id`, `nombre`, `sku`, `categoria_id`, `unidad_medida`, `activo`, `costo NUMERIC(12,2) NOT NULL`, `stock_minimo INT NOT NULL DEFAULT 0`, `stock_actual INT NOT NULL DEFAULT 0` | 0002, 0009 |
| `stock_por_deposito` | `producto_id`, `deposito_id`, `ubicacion_id NULL`, `cantidad` | 0009 |
| `depositos` / `ubicaciones` / `categorias` | `id`, `nombre` | 0002, 0003 |

## `StockFila` (FromRow, interno)

Agrega `costo: Decimal` a los campos actuales.

## `StockItem` (respuesta del listado)

| Campo | Tipo | Regla |
|---|---|---|
| `producto_id`, `nombre`, `sku`, `categoria_id`, `categoria_nombre`, `unidad_medida`, `activo` | sin cambios | — |
| `stock_actual` | i64 | cache `productos.stock_actual` |
| `stock_minimo` | i32 | — |
| `bajo_minimo` | bool | `stock_minimo > 0 && stock_actual < stock_minimo` |
| `faltante` | i64 | **nuevo**: `max(stock_minimo − stock_actual, 0)` |
| `stock_deposito` | i64 \| null | solo con `deposito_id` |
| `costo` | number \| null | **nuevo**: `productos.costo`; `null` si la sesión no es `GESTION` |
| `valorizado` | number \| null | **nuevo**: `costo × (stock_deposito ?? stock_actual)`, redondeo a 2 decimales; `null` si no es `GESTION` |

Construcción: `StockItem::desde_fila(fila: StockFila, ver_costo: bool) -> StockItem` (reemplaza el
`From<StockFila>`).

## `StockProducto` (respuesta del detalle)

Agrega `faltante: i64` con la misma regla. No lleva costo (FR-039 no lo pide).

## `FiltrosStock` (servicio)

Agrega `producto_ids: Option<Vec<i32>>` (1–50 elementos) y `ver_costo: bool`.
