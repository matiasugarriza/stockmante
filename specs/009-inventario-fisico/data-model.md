# Data Model: Inventario físico por conteo (009)

## Entidad: `conteos`

| Campo | Tipo | Nullable | Constraints | Descripción |
|---|---|---|---|---|
| `id` | `SERIAL PRIMARY KEY` | no | | Identificador único |
| `deposito_id` | `INTEGER NOT NULL` | no | FK → `depositos(id)` | Depósito contado |
| `estado` | `VARCHAR(10) NOT NULL` | no | `CHECK (estado IN ('ABIERTO','CERRADO','CANCELADO'))` | Estado del conteo |
| `usuario_id` | `INTEGER NOT NULL` | no | FK → `usuarios(id)` | Autor |
| `creado_en` | `TIMESTAMPTZ NOT NULL` | no | `DEFAULT now()` | Apertura |
| `cerrado_en` | `TIMESTAMPTZ NULL` | sí | | Cierre |
| `cancelado_en` | `TIMESTAMPTZ NULL` | sí | | Cancelación |
| `movimiento_ajuste_id` | `INTEGER NULL` | sí | FK → `movimientos(id)` | AJUSTE generado al cerrar (null si sin diferencias) |
| `observacion` | `TEXT NULL` | sí | `CHECK (length(observacion) <= 500)` | Nota libre **opcional** (FR-034) |

## Entidad: `conteo_lineas`

| Campo | Tipo | Nullable | Constraints | Descripción |
|---|---|---|---|---|
| `id` | `SERIAL PRIMARY KEY` | no | | Identificador único |
| `conteo_id` | `INTEGER NOT NULL` | no | FK → `conteos(id)` ON DELETE CASCADE | Conteo padre |
| `producto_id` | `INTEGER NOT NULL` | no | FK → `productos(id)` | Producto contado |
| `ubicacion_id` | `INTEGER NULL` | sí | FK → `ubicaciones(id)` | Ubicación (null = depósito) |
| `cantidad_sistema` | `INTEGER NOT NULL` | no | `CHECK (cantidad_sistema >= 0)` | Snapshot al abrir; se sobrescribe con el stock vigente al cerrar |
| `cantidad_contada` | `INTEGER NULL` | sí | `CHECK (cantidad_contada IS NULL OR cantidad_contada >= 0)` | Cantidad contada |
| `diferencia` | `INTEGER NULL` | sí | | `cantidad_contada − cantidad_sistema` |

**Constraints adicionales:**
- `UNIQUE NULLS NOT DISTINCT (conteo_id, producto_id, ubicacion_id)` — una línea por
  producto+ubicación (FR-034).
- No existe un `CHECK` que exija `cantidad_contada` no nula para `CERRADO`: esa validación es de
  servicio y devuelve **422 `lineas_sin_contar`** al cerrar (FR-035).

**Semántica de `diferencia` (aclaración 3)**
- Al cargar (`PATCH /lineas`): `diferencia = cantidad_contada − cantidad_sistema` usando el
  **snapshot de apertura**; `cantidad_sistema` no se modifica.
- Al cerrar: se recalcula `cantidad_sistema` con el stock vigente y luego
  `diferencia = cantidad_contada − cantidad_sistema`; ambos valores se persisten (FR-035).

## Transiciones de estado

| Desde | Evento | Hacia | Efecto en stock |
|---|---|---|---|
| — | `POST /api/conteos` | `ABIERTO` | ninguno (crea líneas snapshot) |
| `ABIERTO` | `PATCH /lineas` | `ABIERTO` | ninguno |
| `ABIERTO` | `POST /cerrar` | `CERRADO` | un `AJUSTE` si hay diff ≠ 0 |
| `ABIERTO` | `POST /cancelar` | `CANCELADO` | ninguno |
| `CERRADO` | cualquier acción | — | rechazo **409 `conteo_cerrado`** |
| `CANCELADO` | cualquier acción | — | rechazo **409 `conteo_cancelado`** |

## Transacción de cierre

```
BEGIN;
  SELECT * FROM conteos WHERE id = $id FOR UPDATE;          -- valida estado ABIERTO
  -- validar que todas las líneas tengan cantidad_contada (si no: 422 lineas_sin_contar)
  -- por línea: cantidad_sistema = stock vigente en stock_por_deposito
  --            diferencia = cantidad_contada - cantidad_sistema
  -- si diferencia != 0: asegurar fila y aplicar delta (nunca < 0)
  -- insertar movimiento AJUSTE (motivo INVENTARIO) solo si hay diffs
  -- insertar movimiento_lineas por cada diff != 0
  -- recalcular productos.stock_actual = SUM(stock_por_deposito)
  -- actualizar conteo: estado=CERRADO, cerrado_en=now(), movimiento_ajuste_id
  -- auditar (tabla_ref='conteos', accion=CERRAR)
COMMIT;
```

## Reglas de validación (puras, `models::conteo`)

| Regla | Error |
|---|---|
| `deposito_id > 0` | 422 `validacion` |
| `observacion` ≤ 500 caracteres (opcional) | 422 `validacion` |
| `cantidad_contada >= 0` | 422 `validacion` |
| Depósito inactivo | 422 `deposito_inactivo` |
| Depósito sin filas de stock al abrir | 422 `validacion` |
| Líneas con `cantidad_contada` nula al cerrar | 422 `lineas_sin_contar` |
