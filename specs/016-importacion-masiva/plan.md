# Plan de implementación: Importación masiva de productos y stock (016)

**Spec**: `spec.md` · **Migración**: ninguna (reusa 0002, 0009).

## Contexto técnico

- CSV/Excel con productos y stock inicial.
- Reusa productos (003) y movimientos (007): el stock inicial se registra como movimiento ENTRADA.

## Decisiones de diseño

1. Endpoint `POST /api/importaciones/productos` (multipart/form-data) con CSV.
2. Validación por fila; reporte de errores por fila.
3. Transacción por fila o global? Decisión: procesar fila por fila, registrar errores, continuar (no todo-o-nada) para permitir carga parcial con reporte.
4. Frontend: `/admin/importacion` con upload y reporte de resultados.

## Archivos

- `backend/src/services/importacion.rs`, `handlers/importacion.rs`
- `frontend/src/routes/admin/importacion/+page.svelte`