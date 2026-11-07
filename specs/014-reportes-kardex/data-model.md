# Data Model: Reportes, kardex y valorización (014)

**Cambios de esquema: ninguno.** La feature solo lee. Las tablas y columnas de abajo se
contrastaron con `migrations/`; T001 las confirma contra la BD real antes de tocar SQL.

## Tablas leídas

| Tabla | Columnas usadas | Origen |
|---|---|---|
| `movimientos` | `id`, `tipo` (`ENTRADA`/`SALIDA`/`MERMA`/`AJUSTE`), `motivo` (`COMPRA`, `VENTA`, `MERMA`, `AJUSTE`, `DEVOLUCION`, `TRANSFERENCIA`, `INVENTARIO`, `IMPORTACION`, `OTRO`), `deposito_id`, `creado_en TIMESTAMPTZ` | 0009 (+`transferencia_id` 0010, `venta_id` 0014) |
| `movimiento_lineas` | `id`, `movimiento_id`, `producto_id`, `cantidad_delta INT ≠ 0` (con signo) | 0009 |
| `stock_por_deposito` | `producto_id`, `deposito_id`, `cantidad ≥ 0` | 0009 |
| `productos` | `id`, `nombre`, `sku`, `costo NUMERIC(12,2) ≥ 0`, `stock_actual`, `activo` | 0002 |
| `depositos` | `id`, `nombre` | 0003 |
| `ventas` | `id`, `estado` (`CONFIRMADA`/`ANULADA`), `total NUMERIC(18,2)`, `creado_en` | 0014 |
| `venta_lineas` | `venta_id`, `producto_id`, `cantidad > 0`, `subtotal NUMERIC(18,2)` | 0014 |

Índices que usan las consultas: `idx_lineas_producto`, `idx_lineas_movimiento`,
`idx_movimientos_creado_en`, `idx_movimientos_deposito`, `idx_movimientos_tipo` (0009),
`idx_ventas_creado_en`, `idx_ventas_estado`, `idx_venta_lineas_venta` (0014).

## Valores de entrada (tipos puros, sin I/O)

| Tipo | Campos / variantes | Reglas |
|---|---|---|
| `RangoDias` | `desde: Option<NaiveDate>`, `hasta: Option<NaiveDate>` | `desde ≤ hasta`; se convierte a `[desde 00:00 -03:00, hasta+1 00:00 -03:00)` en UTC |
| `Formato` | `Json` (default), `Csv` | otro valor → 422 |
| `Agrupar` | `Dia` (default), `Producto` | solo ventas; otro valor → 422 |

## Respuestas

### `KardexReporte`

| Campo | Tipo | Nota |
|---|---|---|
| `producto_id` | `i32` | |
| `producto_nombre` | `String` | |
| `deposito_id` | `Option<i32>` | filtro aplicado, `null` = todos |
| `desde`, `hasta` | `Option<NaiveDate>` | tal como se interpretaron |
| `saldo_inicial` | `i32` | Σ deltas antes de `desde` (0 sin `desde`) |
| `lineas` | `Vec<KardexLinea>` | orden `creado_en, movimiento_id, linea_id` |
| `saldo_final` | `i32` | `saldo_inicial + Σ cantidad` (SC-022) |

`KardexLinea`: `movimiento_id i32`, `creado_en DateTime<Utc>`, `tipo String`, `motivo String`,
`deposito_id i32`, `deposito_nombre String`, `cantidad i32` (con signo), `saldo i32`.

### `StockValorizadoReporte`

`items: Vec<StockValorizadoItem>` (`producto_id`, `nombre`, `sku`, `cantidad i32` = Σ
`stock_por_deposito`, `costo_unitario Decimal`, `valor Decimal` = costo × cantidad) y
`total Decimal`. Solo productos con cantidad > 0.

### `RotacionReporte`

`desde`, `hasta` (`NaiveDate`, obligatorios) e `items: Vec<RotacionItem>`:
`producto_id`, `nombre`, `unidades_salidas i32` (SALIDA sin TRANSFERENCIA), `stock_inicial i32`,
`stock_final i32`, `stock_promedio f64`, `indice_rotacion f64` (0 si promedio = 0).

### `VentasReporte`

`desde`, `hasta` (`NaiveDate`), `cantidad_ventas i64`, `total Decimal` (Σ `ventas.total`),
`por_dia: Vec<{ dia: "YYYY-MM-DD", cantidad_ventas i64, total Decimal }>` (día local),
`por_producto: Vec<{ producto_id, nombre, unidades i32, total Decimal }>` (Σ subtotales de línea,
orden por total descendente). Solo ventas `CONFIRMADA`.

Los `Decimal` se serializan como en el resto de la API (`#[schema(value_type = f64)]`).

## Errores

| Situación | `AppError` | HTTP / código | Mensaje |
|---|---|---|---|
| Sesión `OPERACION` | `SinPermisos` | 403 `sin_permisos` | estándar de `RequiereGestion` |
| `desde > hasta` | `Validacion` | 422 `validacion` | `La fecha desde no puede ser posterior a la fecha hasta.` |
| Falta `desde`/`hasta` en rotación o ventas | `Validacion` | 422 | `Indicá las fechas desde y hasta del período (formato AAAA-MM-DD).` |
| Fecha mal formada | `Validacion` | 422 | `El parámetro desde debe estar en formato ISO 8601 (AAAA-MM-DD).` (o `hasta`) |
| `formato` / `agrupar` inválido | `Validacion` | 422 | `El parámetro formato acepta json o csv.` / `El parámetro agrupar acepta dia o producto.` |
| Producto inexistente (kardex) | `RecursoNoEncontrado` | 404 `recurso_no_encontrado` | `El producto no existe.` |
| Falla de BD | genérico | 500 | sin detalles internos (P-V) |
