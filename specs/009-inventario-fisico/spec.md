# Especificación de Funcionalidad: Inventario físico por conteo

**Spec**: `009-inventario-fisico` | **Caso de uso**: US9 | **Prioridad**: P2

**Creado**: 2026-10-26 | **Estado**: Especificado (plan y tasks en generación) | **Módulo**: 2 — Operación de stock

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
  estado `ABIERTO`, con líneas por producto y ubicación que incluyan `cantidad_sistema` (snapshot
  al abrir, no negativa) y `cantidad_contada` (nula hasta que se cargue). La combinación
  (conteo, producto, ubicación) DEBE ser única. `cantidad_contada`, cuando se cargue, DEBE ser
  un entero mayor o igual a cero. El conteo DEBE capturar todas las filas de `stock_por_deposito`
  del depósito al momento de abrir (una línea por producto+ubicación).
- **FR-035 Cierre con ajuste.** Al cerrar, el sistema DEBE recalcular `cantidad_sistema` con el
  stock vigente (para no ajustar contra un snapshot viejo), generar **un** movimiento `AJUSTE`
  (motivo `INVENTARIO`, que requiere familia `GESTION` según FR-027 de 007) con una línea por
  cada diferencia distinta de cero (delta = `cantidad_contada − cantidad_sistema`), y dejar el
  stock igual a lo contado. El cierre DEBE ser transaccional y no reabrible. Si se intenta
  cerrar un conteo ya `CERRADO` o `CANCELADO`, responde **409** `conteo_cerrado` sin re-aplicar
  ningún ajuste (el primer cierre es el único que muta). El cierre DEBE rechazarse con **422**
  `validacion` si alguna línea tiene `cantidad_contada` nula. Al cerrar se recalcula
  `productos.stock_actual` como suma de `stock_por_deposito`. Si todas las diferencias son cero,
  no se genera movimiento.
- **FR-036 Cancelación y consulta.** El sistema DEBE permitir cancelar un conteo abierto sin
  tocar stock (pasa a `CANCELADO`, no reabrible) y consultar conteos con sus diferencias
  (`GET /api/conteos`, `GET /api/conteos/{id}`). El listado y el detalle DEBEN estar disponibles
  para `GESTION`; `OPERACION` recibe **403** `sin_permisos` en toda la superficie de conteos.
- **FR-037 Permisos.** Crear, cargar, cerrar y cancelar conteos DEBE estar reservado a
  `GESTION`; `OPERACION` recibe **403** `sin_permisos`.

## Superficie de API (borrador)

| Método | Ruta | Permiso | Respuesta |
|---|---|---|---|
| POST | `/api/conteos` | `GESTION` | 201 conteo abierto con líneas |
| GET | `/api/conteos?deposito_id=&estado=&desde=&hasta=&pagina=&por_pagina=` | `GESTION` | 200 página |
| GET | `/api/conteos/{id}` | `GESTION` | 200 detalle con diferencias |
| PATCH | `/api/conteos/{id}/lineas` | `GESTION` | 200 conteo con diferencias actualizadas |
| POST | `/api/conteos/{id}/cerrar` | `GESTION` | 200 conteo cerrado (+ movimiento generado) |
| POST | `/api/conteos/{id}/cancelar` | `GESTION` | 200 conteo cancelado |

**Cuerpo de `PATCH /api/conteos/{id}/lineas`**:

```json
{ "lineas": [ { "linea_id": 1, "cantidad_contada": 8 } ] }
```

**Cuerpo de `POST /api/conteos`**:

```json
{ "deposito_id": 1 }
```

**Estados**: `ABIERTO`, `CERRADO`, `CANCELADO`. Transiciones: `ABIERTO → CERRADO` (cierra y ajusta),
`ABIERTO → CANCELADO` (sin ajuste). No se puede reabrir.

**Errores nuevos**:

| Código | Status | Cuándo |
|---|---|---|
| `conteo_cerrado` | 409 | El conteo ya está cerrado o cancelado. |
| `conteo_cancelado` | 409 | El conteo está cancelado. |
| `conteo_no_encontrado` | 404 | No existe el conteo. |
| `lineas_sin_contar` | 422 | Hay líneas con `cantidad_contada` nula al cerrar. |
| `deposito_inactivo` | 422 | El depósito está inactivo. |

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
- La auditoría registra creación, cierre y cancelación (`tabla_ref = 'conteos'`).
- Concurrencia: el cierre y la cancelación bloquean la fila del conteo (`FOR UPDATE`); el segundo
  intento recibe 409.
- El alcance de esta spec incluye backend y una pantalla mínima de conteo en el frontend.
- `cantidad_contada` es un entero >= 0.

## Fuera de Alcance

- Conteos parciales, cíclicos o programados.
- Bloqueo de movimientos durante el conteo.
- Aprobación por un segundo usuario o doble validación.
- Ajuste automático por código de barras escaneado.

## Dependencias y orden de implementación

1. 007 implementada (movimientos y ajustes).
2. Migración reservada: `0011_inventario_fisico.sql` (`conteos`, `conteo_lineas`).
3. Alimenta la auditoría ampliada (015) y los reportes de ajustes (014).
