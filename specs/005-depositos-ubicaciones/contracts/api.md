# API Contracts — Depósitos y ubicaciones (005)

**Base**: `/api` | **Auth**: cookie `stockmante_sesion` | **Errores**: `{ error: { codigo, mensaje } }`

Todas las rutas exigen familia `GESTION`; sin sesión → 401, con familia `OPERACION` → 403
`sin_permisos` (verificado por HTTP).

## Depósitos

### GET `/api/depositos?activo=`

`activo` acepta `true`, `false` o `all`; ausente = solo activos. Devuelve `200 [Deposito]`
ordenado por `nombre ASC, id ASC`.

```json
[
  {
    "id": 1,
    "nombre": "Depósito Central",
    "identificador": "DEP-CENTRAL",
    "direccion": "Av. Siempre Viva 742",
    "activo": true
  }
]
```

### POST `/api/depositos` → 201

```json
{ "nombre": "Depósito Central", "identificador": "DEP-CENTRAL", "direccion": "Av. Siempre Viva 742" }
```

`nombre` 1–120 y `identificador` 1–40 tras `trim`; `direccion` ≤ 500 y opcional. Errores:
`409 deposito_nombre_duplicado`, `409 deposito_identificador_duplicado`, `422 validacion`.

### GET `/api/depositos/{id}` → 200 / 404

### PUT `/api/depositos/{id}` → 200

Parcial: los campos ausentes se conservan; `"direccion": ""` la limpia. `"activo"` presente es
`422` (se usa `/desactivar` o `/reactivar`). Sin campos → `422`.

### DELETE `/api/depositos/{id}` → 204

Solo sin ubicaciones ni asignaciones; si tiene, `409 deposito_con_dependencias`. La alternativa es
`POST /desactivar`.

### POST `/api/depositos/{id}/desactivar` · `/reactivar` → 204

Idempotentes; 404 si el depósito no existe. No tocan sus ubicaciones.

### GET `/api/depositos/{id}/ubicaciones` → 200 `{ items }`

Lista plana ordenada por nombre; el cliente arma el árbol con `padre_id`.

## Ubicaciones

### POST `/api/ubicaciones` → 201

```json
{ "deposito_id": 1, "padre_id": 7, "nombre": "Estantería A", "tipo": "ESTANTERIA" }
```

- `deposito_id` obligatorio, > 0, existente y activo.
- `padre_id` opcional; si viene, debe existir, estar activo y pertenecer al mismo depósito.
- `tipo` ∈ `PASILLO | ESTANTERIA | NIVEL | POSICION` (acepta minúsculas y normaliza).
- `nombre` 1–80 tras `trim`.
- Errores: `409 ubicacion_duplicada` (mismo nivel), `422 ubicacion_huerfana`, `422 validacion`.

### GET `/api/ubicaciones/{id}` → 200 / 404

### PUT `/api/ubicaciones/{id}` → 200

Parcial. `"padre_id": null` mueve a la raíz. Cambiar `deposito_id` solo sin hijos ni
asignaciones (`409 ubicacion_con_dependencias`); si no se indica `padre_id`, pasa a raíz. Un ciclo
o un padre de otro depósito es `422`.

### DELETE `/api/ubicaciones/{id}` → 204

Solo sin hijos ni productos asignados; si no, `409 ubicacion_con_dependencias`.

### POST `/api/ubicaciones/{id}/desactivar` · `/reactivar` → 204

Idempotentes; 404 si no existe.

## Asignación producto–ubicación

### GET `/api/productos/{id}/ubicaciones` → 200 `{ items }`

Ubicaciones permitidas del producto, ordenadas por depósito y nombre. 404 si el producto no existe.

### PUT `/api/productos/{id}/ubicaciones` → 204

```json
{ "ubicacion_ids": [7, 8] }
```

Reemplaza el conjunto completo; `[]` lo limpia. Ids duplicados se deduplican. Cada ubicación debe
existir con depósito activo, si no `404`. Un id ≤ 0 es `422`. Si el producto no existe, `404`.

## Matriz de acceso

| Ruta | Anónimo | OPERACION | GESTION |
|---|---|---|---|
| Cualquier `/api/depositos…` | 401 | 403 | 200/201/204 |
| Cualquier `/api/ubicaciones…` | 401 | 403 | 200/201/204 |
| `/api/productos/{id}/ubicaciones` | 401 | 403 | 200/204 |

## Fuera de este contrato

- Movimientos y transferencias de stock.
- Movimiento de subárboles completos entre depósitos.
- Auditoría de cambios → 006.
