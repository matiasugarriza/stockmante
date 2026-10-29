# Plan de implementación: Ventas de caja y clientes (013)

**Spec**: `spec.md` · **Migración**: `0014_ventas_caja.sql`

## Contexto técnico

- Reusa `movimientos` (SALIDA, motivo VENTA) y `stock_por_deposito` (007).
- `productos.precio_venta` existe (003).
- Pantalla `/caja` existe (004, búsqueda).

## Decisiones de diseño

1. Tablas: `clientes`, `ventas`, `venta_lineas`.
2. Una venta genera un movimiento SALIDA (motivo VENTA) en el depósito de venta.
3. Validar stock suficiente antes de confirmar.
4. Endpoints: `POST /api/ventas`, `GET /api/ventas`, `GET /api/ventas/{id}`, CRUD `/api/clientes`.

## Archivos

- `migrations/0014_ventas_caja.sql`
- `backend/src/models/venta.rs`, `cliente.rs`
- `backend/src/services/ventas.rs`, `clientes.rs`
- `backend/src/handlers/ventas.rs`, `clientes.rs`
- frontend: `/caja` (venta), `/admin/ventas`, `/admin/clientes`

## Riesgos

- Alto: salida de stock; transacción obligatoria; validación de stock.