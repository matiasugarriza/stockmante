# Progreso de implementación — Movimientos de stock (007)

**Spec**: [spec.md](./spec.md) | **Plan**: [plan.md](./plan.md) | **Actualizado**: 2026-11-07 |
**Estado**: base implementada y verificada (2026-10-28); ampliación 2026-11-07 (T020–T034)
implementada.

## Waiver W-005 (constitución 1.3.3)

Aprobado y registrado en `.specify/memory/constitution.md`. Estado por tarea cubierta:

| Tarea | Estado RED | Registro |
|---|---|---|
| T011 `test_movimiento_validacion` (histórico) | no observado | se escribió junto con el modelo; no se reconstruye un RED artificial |
| T012 `test_movimientos_http` (histórico) | no observado | ídem |
| T025 concurrencia (SC-014) | **verde en la primera corrida** (2026-11-07) | verifica el bloqueo `SELECT … FOR UPDATE` por producto ya implementado; 5 rondas de dos altas simultáneas en ubicaciones distintas, cache = suma del desglose |
| T033 filtro `categoria_id` de `GET /api/stock` | **verde en la primera corrida** (2026-11-07) | verifica el filtro ya implementado |

## Evidencia RED de la ampliación (2026-11-07)

Corrida previa a T026 (`cargo test --test test_movimientos_http` y `--test test_movimiento_validacion`):

| Tarea | Test | Resultado antes de T026 |
|---|---|---|
| T021 | `matriz_tipo_motivo_*`, `motivos_reservados_nombran_su_flujo`, `cantidad_entera_*` | **no compila**: `validar_motivo_para_tipo` y `cantidad_entera` no existían (E0432) |
| T022 | `motivo_incoherente_o_reservado_responde_422_sin_escritura` | FAILED (201 en vez de 422) |
| T023 | `ajuste_negativo_mayor_que_el_stock_responde_409` | FAILED (literal viejo de `stock_insuficiente`) |
| T024 | `mensajes_de_validacion_indican_el_siguiente_paso`, `entidades_inactivas_rechazan_el_movimiento`, `salida_sin_stock_responde_409_y_no_cambia_nada` | FAILED (mensajes sin siguiente paso) |
| T031 | `precedencia_de_errores_del_alta` | FAILED (422 de motivo antes del 403) |
| T032 | `cantidad_no_entera_responde_422_especifico` | FAILED (mensaje genérico del extractor) |

Después de T026: `test_movimientos_http` 16/16, `test_movimiento_validacion` 14/14,
`test_asistente_http` 12/12.

**Ajuste de tests existentes**: `ajuste_reservado_a_gestion` usaba motivo `INVENTARIO` para un
`AJUSTE` manual; con la matriz es un motivo reservado, así que pasa a `AJUSTE`.

## Frontend y asistente

- `MOTIVOS_POR_TIPO` y `motivoPorDefecto` en `frontend/src/lib/schemas/movimientos.ts` con test; el
  selector de `routes/movimientos/+page.svelte` solo ofrece los motivos del tipo elegido.
- Fixtures de `movimientos.test.ts`, `transferencias.test.ts` y `asistente.test.ts` alineados al
  literal nuevo de `stock_insuficiente`.
- `asistente/src/tools.ts`: `proponer_movimiento` rechaza un motivo fuera de la matriz (2 tests
  nuevos en `tools.test.ts`; se escribieron junto con el cambio, sin RED registrado).

## Gates (T030)

Ver la sección de gates del cierre transversal en
[specs/006-rbac-auditoria/progress.md](../006-rbac-auditoria/progress.md#gates-del-cierre-2026-11-07):
la corrida es única para 006–017.
