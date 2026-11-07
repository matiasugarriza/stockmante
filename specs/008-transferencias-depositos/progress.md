# Progreso de implementación — Transferencias entre depósitos (008)

**Spec**: [spec.md](./spec.md) | **Plan**: [plan.md](./plan.md) | **Actualizado**: 2026-11-07 |
**Estado**: remediación 2026-11-07 implementada (T005–T009, T016, T024–T036).

## Hallazgo previo (T025)

T006–T009 figuraban como hechas sin que existieran los archivos ni evidencia de RED. Al escribir
`test_transferencias_http` se descubrió que **la transferencia no funcionaba**: el servicio leía
`t.stock_origen_resultante`/`t.stock_destino_resultante`, columnas que `0010_transferencias.sql`
nunca creó, y usaba el alias `do` (palabra reservada de PostgreSQL). Resultado: `POST
/api/transferencias` confirmaba la transacción y respondía **500** al releer el detalle, y `GET`
(listado y detalle) respondía 500 siempre.

## Evidencia RED (2026-11-07)

| Test | Antes de la corrección |
|---|---|
| `transferencia_exitosa_mueve_stock_y_audita` (T007 a/f, T027) | FAILED: 500 `interno` |
| `ubicaciones_de_origen_y_destino` (T009) | FAILED: 500 |
| `stock_por_fila_saldos_y_desborde_en_destino` (T028) | FAILED: 500; (c) desborde sin `checked_add` |
| `listado_detalle_y_movimientos_enlazados` (T008, T030) | FAILED: 500 |
| `detalle_inexistente_responde_404_con_siguiente_paso` (T033) | FAILED: 500 (alias) y mensaje sin siguiente paso |
| `fallas_no_escriben_nada` (T007 b–e, T031) | **verde en la primera corrida** → W-004 |
| `test_transferencia_validacion` (T006, 5 tests) | **verde en la primera corrida** → W-004 |
| `test_openapi::documenta_transferencias` (T032) | **verde en la primera corrida** → W-004 |

## Waiver W-004

Registrado en la constitución 1.3.4 (2026-11-07). Alcance: T006, los casos de falla de T007, T031 y
T032. No se reconstruye un RED artificial.

## Correcciones (T029, T033)

- Los saldos resultantes se leen de `movimiento_lineas.stock_resultante` de cada movimiento
  enlazado (JOIN), sin migración: es el mismo dato que exige la decisión 9 y evita duplicarlo.
- Alias `do` → `dor`.
- `.expect(...)` reemplazados por `ok_or_else(...)?`; `stock_destino_resultante` con `checked_add`
  (422 `validacion` si desborda).
- 404: `No existe la transferencia {id}. Revisá el número o buscala en el listado de transferencias.`

## Resultado

`test_transferencias_http` 6/6, `test_transferencia_validacion` 5/5, `test_openapi` 7/7.
Helper `crear_transferencia_de_prueba` en `tests/unit/common/mod.rs` (T005).

## Gates (T036)

Ver la corrida única del cierre en
[specs/006-rbac-auditoria/progress.md](../006-rbac-auditoria/progress.md#gates-del-cierre-2026-11-07).
