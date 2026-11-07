# Especificación de Funcionalidad: Transferencias entre depósitos

**Spec**: `008-transferencias-depositos` | **Caso de uso**: US8 | **Prioridad**: P1

**Creado**: 2026-10-26 | **Estado**: Implementado (`928aa8a`); remediación 2026-11-07 en curso
(tasks.md, Fase 5) | **Módulo**: 2 — Operación de stock

**Origen**: "transferencias entre depósitos" figura como fuera de alcance del Módulo 1 en el
README y en `specs/README.md`.

**Depende de**: 007 (movimientos de stock), 005 (depósitos y ubicaciones).

## Clarifications

### Session 2026-11-07

- Q: Cuando una transferencia no indica ubicación de origen, ¿contra qué stock se valida y se
  informa? → A: Contra la fila de stock del producto en ese depósito y esa ubicación; sin
  ubicación, la fila sin ubicación del depósito (no el total del depósito), igual que en 007.
- Q: ¿Qué debe mostrar el detalle de una transferencia sobre los dos movimientos que generó?
  → A: Solo sus ids (`movimiento_salida_id`, `movimiento_entrada_id`); el movimiento completo se
  consulta con `GET /api/movimientos/{id}`.

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
  `"Stock insuficiente en Principal: 3 disponibles, 5 solicitadas. Reducí la cantidad o registrá antes una entrada en ese depósito."` y ningún depósito cambia.

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
- **Entonces** se ven origen, destino, cantidad, autor, fecha y los ids de los dos movimientos
  generados (`movimiento_salida_id`, `movimiento_entrada_id`),
- **Y** `GET /api/movimientos/{id}` de cada uno devuelve la `SALIDA` en origen y la `ENTRADA` en
  destino, ambas con motivo `TRANSFERENCIA`.
- **Dado** un id inexistente, **Cuando** se consulta, **Entonces** responde **404**
  `recurso_no_encontrado` con `"No existe la transferencia {id}. Revisá el número o buscala en el
  listado de transferencias."`.

## Requisitos Funcionales

- **FR-031 Transferencia atómica.** El sistema DEBE registrar una transferencia inmediata de un
  producto entre dos depósitos activos y distintos, con cantidad entera mayor que cero y stock
  suficiente en origen. La operación DEBE ejecutarse en una única transacción que descuente origen,
  incremente destino y escriba dos movimientos (`SALIDA` y `ENTRADA`, motivo `TRANSFERENCIA`)
  enlazados a la transferencia.
  El stock que se valida y se informa es el de la fila (producto, depósito, ubicación): la
  ubicación de origen o destino indicada o, si se omite, la fila sin ubicación de ese depósito.
  Con 20 unidades solo en la ubicación A-01 del origen, transferir sin ubicación responde 409
  `stock_insuficiente` con "0 disponibles".
- **FR-032 Consulta de transferencias.** El sistema DEBE exponer `GET /api/transferencias`
  (filtros por producto, depósito, rango de fechas, paginado) y `GET /api/transferencias/{id}` con
  los ids de los dos movimientos asociados.
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
- Sin idempotencia: un reintento manual registra una segunda transferencia.
- `OPERACION` y `GESTION` son hoy las únicas familias de rol, así que transferir exige solo sesión
  vigente; si aparece una familia nueva, esta regla se revisa.

## Fuera de Alcance

- Transferencias programadas, en tránsito o con aprobación de un segundo usuario.
- Mover stock entre ubicaciones del mismo depósito.
- Logística, transportistas o remitos.

## Dependencias y orden de implementación

1. 007 implementada (tablas de stock y movimientos).
2. Migración reservada: `0010_transferencias.sql` (`transferencias` y `movimientos.transferencia_id`).
3. Habilita reportes por depósito (014) y auditoría ampliada (015).
