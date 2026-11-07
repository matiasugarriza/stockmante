# Progreso de implementación — Inventario físico (009)

**Spec**: [spec.md](./spec.md) | **Plan**: [plan.md](./plan.md) | **Actualizado**: 2026-11-07 |
**Estado**: implementado y verificado por tests (2026-11-07).

## Esquema (T002)

`_sqlx_migrations` tiene la versión 11 (`inventario fisico`) en `stockmante` y `stockmante_test`;
`conteos` y `conteo_lineas` existen con las columnas de `0011_inventario_fisico.sql` (verificado
con `psql` el 2026-11-07).

## Evidencia RED (2026-11-07)

| Test | Antes de T004/T013/T014 |
|---|---|
| `alta_y_consulta_de_conteos` (T008) | FAILED: `validacion` en vez de `deposito_inactivo`; el depósito sin stock creaba un conteo vacío |
| `carga_y_cierre_ajustan_al_contado` (T009) | FAILED: `validacion` en vez de `lineas_sin_contar` |
| `cierre_ajusta_contra_el_stock_vigente` (SC-017) | FAILED: con stock cambiado tras abrir, el cierre **dejaba el stock en lo contado sin generar movimiento** (comparaba contra el snapshot) |
| `cancelacion_y_rbac` (T010) | verde en la primera corrida (comportamiento ya implementado) |
| `test_conteo_validacion` (T007, 4 tests) | verde en la primera corrida (reglas puras ya implementadas) |

Los dos últimos no tienen waiver propio: son tests de verificación de comportamiento existente, que
la tarea pedía crear; se registran acá sin reconstruir un RED artificial.

## Correcciones

- `AppError::LineasSinContar` (422 `lineas_sin_contar`) y `AppError::DepositoInactivo` (422
  `deposito_inactivo`).
- El alta verifica las filas de stock **antes** de insertar la cabecera: sin filas → 422
  `validacion` (`El depósito no tiene stock registrado para contar.`).
- El cierre: una sola compuerta para líneas sin contar (sin `unwrap()`), diferencia `contada −
  vigente` con `checked_sub`, el stock queda igual a lo contado y el `AJUSTE` se arma con esas
  mismas diferencias (antes podía omitirse o tener deltas que no explicaban el cambio de stock).
- OpenAPI con los códigos específicos (T018) y test de rutas/schemas (T029).

## Resultado

`test_conteos_http` 4/4, `test_conteo_validacion` 4/4; frontend `schemas/conteos.test.ts` (5) y
`api/conteos.test.ts` (+3 códigos específicos).
