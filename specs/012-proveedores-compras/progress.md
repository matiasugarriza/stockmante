# Progreso de implementación — Proveedores y compras (012)

**Spec**: [spec.md](./spec.md) | **Plan**: [plan.md](./plan.md) | **Actualizado**: 2026-11-07 |
**Estado**: fases 1–5 implementadas; fase 6 (T024–T035) implementada; T036 en el cierre transversal.

## Waiver W-003

Registrado en la constitución 1.3.4: RED no observado en T006/T020/T023 (tests escritos junto con el
código). No se reconstruye un RED artificial.

## Esquema (T024)

Esquema real idéntico al de las migraciones (comparación completa contra una base temporal); ver
`data-model.md`.

## Evidencia RED de la fase 6 (2026-11-07)

| Tarea | Test | Antes de implementar |
|---|---|---|
| T026 | `test_compras_concurrencia` | FAILED: **500 `interno`** en la ronda 2 (la anulación leía el stock sin bloqueo y el `UPDATE` dejaba saldo negativo contra el `CHECK`) |
| T028 | `test_compras_validacion` | no compila: `validar_numero`, `validar_costo`, `validar_estado`, `parsear_fecha` no existían |
| T028 | `escenario_12_2b_*`, `escenario_12_5b_*`, `escenario_12_8_*` | FAILED (422 genérico, `compra_no_editable`, filtros ignorados) |
| T031 | `confirmar_y_anular_devuelven_el_movimiento` | FAILED: la respuesta no traía `movimiento` |
| T033 | `api/compras.test.ts`, `utils/compras.test.ts` | FAILED: schema sin `movimiento`; módulo de aviso inexistente |

`escenario_12_1_*` se actualizó: el proveedor inactivo pasa de 422 `validacion` a 409
`proveedor_inactivo` (clarificación 3).

## Resultado

`test_compras_http` 11/11, `test_compras_concurrencia` 1/1 (6 rondas; repetido 4 veces sin fallos),
`test_compras_validacion` 4/4; frontend `api/compras.test.ts`, `utils/compras.test.ts` y
`schemas/compras.test.ts` 17/17, `svelte-check` 0/0.

## Cambios

- `AppError::CompraNoAnulable(String)` y `AppError::ProveedorInactivo(String)` (409).
- `anular`: depósito y productos `FOR UPDATE` en el orden de 007 y descuento condicional
  `UPDATE … AND cantidad >= $n RETURNING cantidad`.
- `confirmar`: `checked_add` en el saldo resultante.
- `ResultadoCompra { compra, lineas, movimiento }` en confirmar y anular (OpenAPI actualizado).
- UI: aviso «Compra #7 confirmada en Depósito Central: Cemento 50 kg → 24 u.», rótulo «Nº
  factura/remito del proveedor (máx. 40)» y «Compra #id» en la tabla.
