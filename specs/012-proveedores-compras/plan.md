# Plan de implementación: Proveedores y compras (012)

**Spec**: `spec.md` · **Migración**: `0013_proveedores_compras.sql`

## Contexto técnico

- Reusa `movimientos` (ENTRADA, motivo COMPRA) y `stock_por_deposito` (007).
- `productos.costo` se actualiza con el costo de compra.
- Patrón de capas: models → services (transacciones) → handlers → app.rs.

## Decisiones de diseño

1. Tablas: `proveedores`, `ordenes_compra`, `orden_compra_lineas`, `recepciones`.
2. Una compra recibida genera un movimiento ENTRADA (motivo COMPRA) en el depósito destino.
3. El costo del producto se actualiza al costo de la última compra (o promedio ponderado — decisión: último costo por simplicidad, YAGNI).
4. Endpoints REST bajo `/api/proveedores` y `/api/compras`.

## Superficie de API (borrador)

| Método | Ruta | Permiso | Respuesta |
|---|---|---|---|
| CRUD | `/api/proveedores` | GESTION | 201/200 |
| POST | `/api/compras` | GESTION | 201 orden |
| POST | `/api/compras/{id}/recibir` | GESTION | 200 recepción + movimiento |
| GET | `/api/compras` | GESTION | 200 página |

## Archivos

- `migrations/0013_proveedores_compras.sql`
- `backend/src/models/proveedor.rs`, `compra.rs`
- `backend/src/services/proveedores.rs`, `compras.rs`
- `backend/src/handlers/proveedores.rs`, `compras.rs`
- frontend: `/admin/proveedores`, `/admin/compras`

## Riesgos

- Alto: la recepción genera entrada de stock; transacción obligatoria.
- Concurrencia: bloqueo de stock.