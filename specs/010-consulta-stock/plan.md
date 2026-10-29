# Plan de implementación: Consulta y tablero de stock (010)

**Spec**: `spec.md` · **Migración**: ninguna (usa 0009 y sus índices).

## Contexto técnico

- Backend de stock ya implementado: `handlers/stock.rs`, `services/stock.rs`, `models/stock.rs` (listado paginado con filtros q/deposito/categoria/bajo_minimo/activo, detalle con desglose).
- Falta frontend: `/admin/stock` (gestión, valorizado a costo) y stock disponible en `/caja` (operación).
- `productos.costo` se usa para valorizado.

## Decisiones de diseño

1. Reusar endpoints existentes; no cambios de backend.
2. `/admin/stock`: tabla con filtros, paginación, mínimo, valorizado a costo (solo GESTION).
3. `/caja`: ya existe búsqueda; agregar panel de stock disponible por depósito de venta.

## Archivos

| Archivo | Cambio |
|---|---|
| `frontend/src/routes/admin/stock/+page.svelte` | nuevo |
| `frontend/src/routes/caja/+page.svelte` | agregar stock disponible |
| `frontend/src/lib/api/stock.ts` | ya existe |
| `docs/api.md` | (ya documentado) |

## Estrategia de pruebas

- Frontend: tests de schemas (stock ya tiene tests en movimientos.test.ts? verificar) y componente.
- Backend: ya cubierto por movimientos_http + stock tests.

## Orden de ejecución

1. Crear `/admin/stock`.
2. Ajustar `/caja`.
3. `npm test`, `npm run check`, `npm run build`.