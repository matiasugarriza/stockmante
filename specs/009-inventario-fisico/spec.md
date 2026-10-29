# Especificación de Funcionalidad: Inventario físico por conteo

**Spec**: `009-inventario-fisico` | **Caso de uso**: US9 | **Prioridad**: P2

**Creado**: 2026-10-26 | **Estado**: Especificado (sin `plan.md` ni `tasks.md`; se generan al
implementar) | **Módulo**: 2 — Operación de stock

**Origen**: la exactitud del inventario no se puede sostener solo con movimientos; el conteo
físico es el mecanismo estándar para detectar diferencias y ajustarlas con trazabilidad.

**Depende de**: 007 (movimientos y ajustes), 005 (depósitos y ubicaciones).

## Escenarios de Usuario y Pruebas *(obligatorio)*

### User Story 9 - Contar el inventario y ajustar diferencias (Prioridad: P2)

Como **Gerente**, quiero abrir un conteo de un depósito, cargar las cantidades contadas y cerrarlo
para que las diferencias se ajusten automáticamente, para mantener el stock del sistema igual al
stock real sin planillas paralelas.

**Por qué esta prioridad**: sin conteos, cualquier error de carga o merma no registrada se acumula
y el inventario pierde confiabilidad; es el control periódico que exige un sistema de inventario.

**Escenario 9.1 — Abrir un conteo**

- **Dado** un gerente y el depósito "Principal" activo,
- **Cuando** crea un conteo,
- **Entonces** responde **201** con estado `ABIERTO` y las líneas del stock actual del depósito
  (producto, ubicación, cantidad del sistema) para contar sobre ellas.

**Escenario 9.2 — Cargar cantidades contadas**

- **Dado** un conteo abierto,
- **Cuando** se cargan las cantidades contadas por producto y ubicación,
- **Entonces** cada línea muestra la diferencia contra el stock del sistema y el conteo sigue
  abierto.

**Escenario 9.3 — Cerrar con diferencias**

- **Dado** un conteo con una diferencia de −2 en un producto y 0 en el resto,
- **Cuando** se cierra,
- **Entonces** el sistema genera **un** movimiento `AJUSTE` (motivo `INVENTARIO`) solo por las
  líneas con diferencia distinta de cero, el stock queda igual a lo contado y el conteo pasa a
  `CERRADO`.

**Escenario 9.4 — Cerrar sin diferencias**

- **Dado** un conteo donde todo coincide,
- **Cuando** se cierra,
- **Entonces** no se genera ningún movimiento y el conteo pasa a `CERRADO`.

**Escenario 9.5 — Conteo cerrado**

- **Dado** un conteo en estado `CERRADO`,
- **Cuando** se intenta cargar una línea o cerrarlo de nuevo,
- **Entonces** responde **409** `conteo_cerrado` con
  `"El conteo ya está cerrado; abrí uno nuevo para volver a contar."`.

**Escenario 9.6 — Cancelar**

- **Dado** un conteo abierto,
- **Cuando** se cancela,
- **Entonces** pasa a `CANCELADO`, no ajusta stock y no puede reabrirse.

**Escenario 9.7 — Operario**

- **Dado** una sesión `OPERACION`,
- **Cuando** intenta abrir, cargar o cerrar un conteo,
- **Entonces** responde **403** `sin_permisos`.

## Requisitos Funcionales

- **FR-034 Conteo y líneas.** El sistema DEBE permitir crear un conteo por depósito activo en
  estado `ABIERTO`, con líneas por producto y ubicación que incluyan `cantidad_sistema` y
  `cantidad_contada` (nula hasta que se cargue). La combinación (conteo, producto, ubicación) DEBE
  ser única.
- **FR-035 Cierre con ajuste.** Al cerrar, el sistema DEBE recalcular la cantidad del sistema con
  el stock vigente, generar **un** movimiento `AJUSTE` (motivo `INVENTARIO`) con una línea por cada
  diferencia distinta de cero y dejar el stock igual a lo contado. El cierre DEBE ser transaccional,
  idempotente y no reabrible.
- **FR-036 Cancelación y consulta.** El sistema DEBE permitir cancelar un conteo abierto sin tocar
  stock y consultar conteos con sus diferencias (`GET /api/conteos`, `GET /api/conteos/{id}`).
- **FR-037 Permisos.** Crear, cargar, cerrar y cancelar conteos DEBE estar reservado a `GESTION`;
  la consulta de conteos cerrados DEBE estar disponible para `GESTION`.

## Criterios de Éxito Medibles

- **SC-017** Cero cierres con diferencias sin ajuste: después de cerrar un conteo, el stock del
  depósito coincide exactamente con lo contado para el 100% de las líneas.

## Supuestos

- El conteo es total por depósito; el conteo parcial por producto o categoría queda fuera de
  alcance.
- Las cantidades del sistema se recalculan al cerrar para no ajustar contra un snapshot viejo si
  hubo movimientos durante el conteo.
- El ajuste que genera el cierre es negativo si se contó menos y positivo si se contó más.
- No se bloquean movimientos mientras el conteo está abierto.
- No hay conteos cíclicos programados ni asignación de sectores a operarios.

## Fuera de Alcance

- Conteos parciales, cíclicos o programados.
- Bloqueo de movimientos durante el conteo.
- Aprobación por un segundo usuario o doble validación.
- Ajuste automático por código de barras escaneado.

## Dependencias y orden de implementación

1. 007 implementada (movimientos y ajustes).
2. Migración reservada: `0011_inventario_fisico.sql` (`conteos`, `conteo_lineas`).
3. Alimenta la auditoría ampliada (015) y los reportes de ajustes (014).
