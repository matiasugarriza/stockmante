# Research: Consulta y tablero de stock (010)

Fecha: 2026-11-07. Fuente: clarificaciones del spec y hallazgos de `/speckit analyze`.

## R1 — Depósito de la caja

- **Decision**: selector de depósitos activos en `/caja` (`listarDepositos('true')`). El valor
  elegido se recuerda en `localStorage` como preferencia de pantalla; si el depósito ya no está
  activo, se descarta y se toma el primero de la lista.
- **Rationale**: no existe un "depósito de venta" en el esquema; las ventas (0014) ya llevan
  `deposito_id`, así que el mismo selector servirá a 013 sin migración.
- **Alternatives considered**: depósito fijo por configuración (rígido con varias cajas); mostrar
  solo el total (no responde "¿hay acá?").

## R2 — Valorizado reservado a GESTION

- **Decision**: el handler pasa `ver_costo = usuario.familia_rol == "GESTION"` al servicio. La
  consulta siempre lee `p.costo`; la conversión `StockItem::desde_fila(fila, ver_costo)` deja
  `costo` y `valorizado` en `None` (JSON `null`) cuando `ver_costo` es falso.
- **Rationale**: la regla queda en una función pura testeable y en el backend; la UI no decide
  quién ve el costo.
- **Alternatives considered**: endpoint aparte para el valorizado (duplica filtros y paginación);
  ocultar solo en la UI (filtra el costo a cualquier `OPERACION` que llame a la API).

## R3 — Alcance de cada fila del listado

- **Decision**: el listado no lleva desglose. Con `deposito_id` lleva `stock_deposito`; el desglose
  por depósito y ubicación sale del detalle (FR-039), que `/admin/stock` abre desde la fila.
- **Rationale**: es lo que ya hace la API; agregar un arreglo por fila multiplica el payload.

## R4 — Stock por depósito para los resultados de la caja

- **Decision**: `GET /api/stock` acepta `producto_ids` (lista separada por comas, máximo 50 ids,
  422 si no son enteros o superan el máximo). La caja busca con `/api/productos/buscar` (hasta 20
  resultados) y luego llama `listarStock({ productoIds, depositoId, porPagina: 50 })`.
- **Rationale**: una sola llamada; reusa el mismo cálculo de `stock_deposito`. La búsqueda difusa
  de 004 (trigramas) no se puede reemplazar por el `ILIKE` de `/api/stock` sin perder resultados.
- **Alternatives considered**: una llamada de detalle por producto (hasta 20 requests); agregar
  `deposito_id` a `/api/productos/buscar` (mezcla 004 con stock).

## R5 — Faltante

- **Decision**: `faltante = max(stock_minimo − stock_actual, 0)` como `i64`, calculado en
  `StockItem::desde_fila` y en el detalle. Con `stock_minimo = 0` vale 0.
- **Rationale**: lo usarán también 011 (alertas) y el asistente; queda en un solo lugar.

## R6 — Medición de SC-018

- **Decision**: test de integración ignorado por defecto (`#[ignore]`, se corre con
  `cargo test -- --ignored sc018`) que siembra 10.000 productos y 3 depósitos, mide la página con y
  sin `q` (nombre y SKU) por HTTP y guarda el `EXPLAIN ANALYZE` en la salida. Si alguna variante
  supera 1 s, se crea `migrations/0015_productos_sku_trgm.sql` con
  `CREATE INDEX idx_productos_sku_trgm ON productos USING gin (sku gin_trgm_ops)` y se repite.
- **Rationale**: el `OR p.sku ILIKE '%…%'` no tiene índice; medir antes de agregarlo evita un
  índice innecesario (Principio I).
