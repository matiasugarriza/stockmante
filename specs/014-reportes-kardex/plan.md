# Plan de implementación: Reportes, kardex y valorización (014)

**Spec**: `spec.md` · **Migración**: ninguna (solo lectura).

## Contexto técnico

- Fuentes: `movimientos`, `movimiento_lineas`, `stock_por_deposito`, `productos`, `ventas`, `compras`.
- Consultas de agregación; sin escritura.

## Decisiones de diseño

1. Endpoints read-only bajo `/api/reportes`: kardex por producto, valorización por depósito, resumen de ventas/compras por período.
2. Paginación estándar.
3. Permisos: GESTION para reportes valorizados; OPERACION puede ver kardex de productos.

## Archivos

- `backend/src/services/reportes.rs`, `handlers/reportes.rs`
- `frontend/src/routes/admin/reportes/+page.svelte`

## Riesgos

- Performance: índices existentes; usar EXPLAIN ANALYZE.