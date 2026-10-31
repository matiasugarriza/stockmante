# API Contract: Inventario físico por conteo (009)

## POST /api/conteos

Abre un conteo por depósito.

**Permiso**: `GESTION`.

**Request body**:

```json
{ "deposito_id": 1, "observacion": "Conteo mensual" }
```

**Respuesta 201**: `ConteoDetalle` con líneas del stock actual (`cantidad_sistema`, `cantidad_contada: null`).

**Errores**: 401, 403, 422 `deposito_inactivo` / `validacion`.

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

**Respuesta 200**: `ConteoDetalle` con diferencias recalculadas.

**Errores**: 404, 409 `conteo_cerrado` / `conteo_cancelado`, 422 `validacion`.

---

## POST /api/conteos/{id}/cerrar

Cierra el conteo y genera el AJUSTE si hay diferencias.

**Permiso**: `GESTION`.

**Respuesta 200**: `ConteoDetalle` (estado `CERRADO`, `movimiento_ajuste_id`).

**Errores**: 404, 409 `conteo_cerrado` / `conteo_cancelado`, 422 `lineas_sin_contar`.

---

## POST /api/conteos/{id}/cancelar

Cancela un conteo abierto sin tocar stock.

**Permiso**: `GESTION`.

**Respuesta 200**: `ConteoDetalle` (estado `CANCELADO`).

**Errores**: 404, 409 `conteo_cerrado` / `conteo_cancelado`.