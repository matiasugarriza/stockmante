# API Contract: Inventario físico por conteo (009)

Fuente de verdad de los códigos de error: [`spec.md`](../spec.md) §Errores. Toda la superficie
requiere `GESTION`; `OPERACION` recibe **403** `sin_permisos`.

## POST /api/conteos

Abre un conteo por depósito.

**Permiso**: `GESTION`.

**Request body**:

```json
{ "deposito_id": 1, "observacion": "Conteo mensual" }
```

`observacion` es opcional y admite hasta 500 caracteres.

**Respuesta 201**: `ConteoDetalle` con líneas del stock actual (`cantidad_sistema`,
`cantidad_contada: null`).

**Errores**: 401, 403, 422 `deposito_inactivo` (depósito inactivo), 422 `validacion`
(`deposito_id` inexistente, depósito sin filas de `stock_por_deposito`, `observacion` > 500,
`deposito_id` ≤ 0).

---

## GET /api/conteos

Lista conteos con filtros.

**Permiso**: `GESTION`.

**Query**: `deposito_id`, `estado`, `desde`, `hasta`, `pagina`, `por_pagina`.

**Respuesta 200**: `ListaConteos`.

---

## GET /api/conteos/{id}

Detalle de un conteo con líneas y diferencias.

**Permiso**: `GESTION`.

**Respuesta 200**: `ConteoDetalle`. **404** `conteo_no_encontrado`.

---

## PATCH /api/conteos/{id}/lineas

Carga cantidades contadas.

**Permiso**: `GESTION`.

**Request body**:

```json
{ "lineas": [ { "linea_id": 1, "cantidad_contada": 8 } ] }
```

**Cálculo de `diferencia`**: `cantidad_contada − cantidad_sistema` usando el **snapshot de
apertura**; `cantidad_sistema` no se modifica en este endpoint. El recálculo contra el stock
vigente ocurre solo al cerrar.

**Respuesta 200**: `ConteoDetalle` con diferencias actualizadas.

**Errores**: 404 `conteo_no_encontrado`, 409 `conteo_cerrado` (conteo `CERRADO`), 409
`conteo_cancelado` (conteo `CANCELADO`), 422 `validacion` (cantidad negativa o línea ajena).

---

## POST /api/conteos/{id}/cerrar

Cierra el conteo y genera el `AJUSTE` si hay diferencias.

**Permiso**: `GESTION`.

**Respuesta 200**: `ConteoDetalle` (estado `CERRADO`, `movimiento_ajuste_id` null si no hubo
diferencias).

**Errores**: 404 `conteo_no_encontrado`, 409 `conteo_cerrado`, 409 `conteo_cancelado`, 422
`lineas_sin_contar` (alguna línea sin `cantidad_contada`).

---

## POST /api/conteos/{id}/cancelar

Cancela un conteo abierto sin tocar stock.

**Permiso**: `GESTION`.

**Respuesta 200**: `ConteoDetalle` (estado `CANCELADO`).

**Errores**: 404 `conteo_no_encontrado`, 409 `conteo_cerrado`, 409 `conteo_cancelado`.
