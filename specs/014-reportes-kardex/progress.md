# Progreso de implementación — Reportes y kardex (014)

**Spec**: [spec.md](./spec.md) | **Plan**: [plan.md](./plan.md) | **Actualizado**: 2026-11-07 |
**Estado**: implementado (T001–T026); T027/T028 (gates y estado) en el cierre transversal.

## Evidencia RED (T007, 2026-11-07)

| Test | Antes de implementar |
|---|---|
| `test_reportes_calculo` | no compila: `reportes::{rango, calculo, csv}` no existían |
| `test_reportes_http` | 6/6 FAILED: último día excluido (`hasta` a las 00:00 UTC), stock valorizado desde el cache y solo activos, rotación con transferencias y mermas, ventas por día UTC, CSV sin BOM ni `Content-Disposition` y faltante en 3 de 4 reportes, 422 ausentes (defaults de 30 días) |
| `test_rbac_matriz` | FAILED: `OPERACION` recibía 200 en los reportes (salvo stock valorizado) |
| `api/reportes.test.ts` (kardex, rotación, ventas, `urlCsv`) | FAILED: funciones inexistentes |

## Cambios

- `services/reportes/` como módulo: `rango` (días locales y límites UTC semiabiertos), `calculo`
  (`acumular_saldos`, `calcular_rotacion`), `csv` (RFC 4180 con BOM) y las cuatro consultas
  reescritas sin `Utc::now()`; rotación en una sola consulta agregada con `FILTER`.
- Handlers con `RequiereGestion`, `formato`/`agrupar` cerrados (422) y CSV de los cuatro reportes.
- `StockValorizadoItem.cantidad` pasa a `i64` (ver `data-model.md`).
- Frontend: schemas y cliente (`obtenerKardex`, `obtenerRotacion`, `obtenerVentas`, `urlCsv`;
  `obtenerStockValorizado` se conserva para el tablero de 018), cuatro componentes, pantalla
  `/admin/reportes` y enlace «Reportes» en la navegación de GESTION.
- Test de 008 con desborde: ahora limpia el saldo `i32::MAX` al terminar.

## Resultado

`test_reportes_http` 6/6, `test_reportes_calculo` 6/6, `test_rbac_matriz` 11/11, `test_openapi`
10/10; `api/reportes.test.ts` 6/6, `svelte-check` 0/0.

## Nota sobre el asistente (017)

La tool `reporte_agregado` llama a `/api/reportes/*` con la cookie del usuario: con `RequiereGestion`
un usuario `OPERACION` recibe 403 y el asistente lo informa (comportamiento esperado por SC-031).
