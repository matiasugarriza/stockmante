# Especificación de Funcionalidad: Transferencias entre depósitos

**Spec**: `008-transferencias-depositos` | **Caso de uso**: US8 | **Prioridad**: P1

**Creado**: 2026-10-26 | **Estado**: Especificado (sin `plan.md` ni `tasks.md`; se generan al
implementar) | **Módulo**: 2 — Operación de stock

**Origen**: "transferencias entre depósitos" figura como fuera de alcance del Módulo 1 en el
README y en `specs/README.md`.

**Depende de**: 007 (movimientos de stock), 005 (depósitos y ubicaciones).

## Escenarios de Usuario y Pruebas *(obligatorio)*

### User Story 8 - Mover stock entre depósitos sin perder unidades (Prioridad: P1)

Como **Operario o Gerente**, quiero transferir unidades de un depósito a otro en una sola
operación, para abastecer sucursales o cámaras sin que el stock se pierda ni se duplique por el
camino.

**Por qué esta prioridad**: es la operación diaria de un negocio con más de un depósito y la única
forma segura de mover stock sin dos movimientos manuales que pueden quedar a medias.

**Escenario 8.1 — Transferencia completa**

- **Dado** 20 unidades de un producto en "Principal" y el depósito "Sucursal" activo,
- **Cuando** se transfieren 8 unidades,
- **Entonces** responde **201** con el stock resultante en origen (12) y destino (8), en una sola
  transacción.

**Escenario 8.2 — Stock insuficiente en origen**

- **Dado** 3 unidades en origen,
- **Cuando** se transfieren 5,
- **Entonces** responde **409** `stock_insuficiente` con el mensaje
  `"Stock insuficiente en Principal: 3 disponibles, 5 solicitadas."` y ningún depósito cambia.

**Escenario 8.3 — Mismo depósito**

- **Dado** origen y destino iguales,
- **Cuando** se envía la transferencia,
- **Entonces** responde **422** `transferencia_mismo_deposito` con
  `"El depósito de origen y el de destino deben ser distintos."`.

**Escenario 8.4 — Depósito inactivo**

- **Dado** un depósito de destino inactivo,
- **Cuando** se transfiere,
- **Entonces** responde **422** `validacion` y no se escribe nada.

**Escenario 8.5 — Consulta**

- **Dado** una transferencia confirmada,
- **Cuando** se consulta `GET /api/transferencias/{id}`,
- **Entonces** se ven origen, destino, cantidad, autor, fecha y los dos movimientos generados.

## Requisitos Funcionales

- **FR-031 Transferencia atómica.** El sistema DEBE registrar una transferencia inmediata de un
  producto entre dos depósitos activos y distintos, con cantidad entera mayor que cero y stock
  suficiente en origen. La operación DEBE ejecutarse en una única transacción que descuente origen,
  incremente destino y escriba dos movimientos (`SALIDA` y `ENTRADA`, motivo `TRANSFERENCIA`)
  enlazados a la transferencia.
- **FR-032 Consulta de transferencias.** El sistema DEBE exponer `GET /api/transferencias`
  (filtros por producto, depósito, rango de fechas, paginado) y `GET /api/transferencias/{id}` con
  los dos movimientos asociados.
- **FR-033 Permisos y resultado concreto.** `OPERACION` y `GESTION` DEBEN poder transferir; la
  respuesta DEBE informar el stock resultante en origen y destino (Principio III) y la acción DEBE
  quedar auditada (`tabla_ref = 'transferencias'`, acción `CREAR`).

## Criterios de Éxito Medibles

- **SC-016** Cero transferencias con stock inconsistente: la suma global de stock antes y después
  de cualquier transferencia (exitosa o fallida) es idéntica, y origen/destino reflejan
  exactamente la cantidad transferida.

## Supuestos

- La transferencia es inmediata: no hay estado "en tránsito" ni recepción diferida en destino.
- Se transfiere entre depósitos; mover stock entre ubicaciones del mismo depósito queda fuera de
  alcance (se resuelve con un ajuste o movimiento interno en una spec posterior).
- El costo del producto no cambia al transferirse.
- No se transfieren productos inactivos.

## Fuera de Alcance

- Transferencias programadas, en tránsito o con aprobación de un segundo usuario.
- Mover stock entre ubicaciones del mismo depósito.
- Logística, transportistas o remitos.

## Dependencias y orden de implementación

1. 007 implementada (tablas de stock y movimientos).
2. Migración reservada: `0010_transferencias.sql` (`transferencias` y `movimientos.transferencia_id`).
3. Habilita reportes por depósito (014) y auditoría ampliada (015).
