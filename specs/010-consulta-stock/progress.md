# Progreso de implementación — Consulta de stock (010)

**Spec**: [spec.md](./spec.md) | **Plan**: [plan.md](./plan.md) | **Actualizado**: 2026-11-07 |
**Estado**: implementado (T001–T019); gates en el cierre transversal.

## Esquema (T001)

Verificado con `psql` sobre `stockmante_test`: `productos.costo numeric NOT NULL`,
`stock_actual integer NOT NULL`, `stock_minimo integer NOT NULL`; `stock_por_deposito` con
`ubicacion_id` nullable. Coincide con `data-model.md`; sin diferencias.

## Evidencia RED (2026-11-07)

| Tarea | Resultado antes de implementar |
|---|---|
| T002 `test_stock_item` | no compila: `StockFila.costo` y `StockItem::desde_fila` no existían |
| T004 `test_stock_http` | 3/3 FAILED (`faltante` ausente, `producto_ids` ignorado: 805 resultados en vez de 2) |
| R1 `depositos_activos_para_cualquier_sesion` | FAILED: 403 |
| T008 `schemas/movimientos.test.ts` | FAILED: el schema aceptaba items sin `faltante` |
| T009 `api/stock.test.ts` | FAILED: `producto_ids` no se serializaba |
| T010 `utils/stock.test.ts` | FAILED: el módulo no existía |
| `api/depositos.test.ts` (activos) | FAILED: `listarDepositosActivos` no existía |

## Resultado

`test_stock_http` 4/4 (+ `sc018` ignorado), `test_stock_item` 4/4; frontend 296+ tests, `svelte-check`
0/0; asistente 53/53 con `faltante` en el fixture (T017).

## Pantallas (T014–T016)

- `/admin/stock`: filtros de categoría y estado, columnas de stock en depósito, faltante, costo y
  valorizado, desglose al hacer clic en el producto, errores con Reintentar y aviso si fallan los
  filtros (ya no se descartan en silencio).
- `/caja`: selector de depósito activo recordado en `localStorage` (`elegirDeposito`), columna
  "Stock en <depósito>" con `producto_ids` y descarte de respuestas viejas por turno.
- `/movimientos` y `/transferencias` usan `listarDepositosActivos` (antes 403 para OPERACION).
- El enlace "Stock" sigue restringido a `GESTION` (`navegacion.ts`, `roles: ['GESTION']`).

## Pendiente

- T020 (gates) en el cierre transversal; T021 (recorrido manual del quickstart) requiere el backend
  reiniciado con este código.
