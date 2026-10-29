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
| `observacion` | `TEXT NULL` | sí | `CHECK (length(observacion) <= 500)` | Nota libre |

## Entidad: `conteo_lineas`

| Campo | Tipo | Nullable | Constraints | Descripción |
|---|---|---|---|---|
| `id` | `SERIAL PRIMARY KEY` | no | | Identificador único |
| `conteo_id` | `INTEGER NOT NULL` | no | FK → `conteos(id)` ON DELETE CASCADE | Conteo padre |
| `producto_id` | `INTEGER NOT NULL` | no | FK → `productos(id)` | Producto contado |
| `ubicacion_id` | `INTEGER NULL` | sí | FK → `ubicaciones(id)` | Ubicación (null = depósito) |
| `cantidad_sistema` | `INTEGER NOT NULL` | no | `CHECK (cantidad_sistema >= 0)` | Snapshot al abrir / vigente al cerrar |
| `cantidad_contada` | `INTEGER NULL` | sí | `CHECK (cantidad_contada IS NULL OR cantidad_contada >= 0)` | Cantidad contada |
| `diferencia` | `INTEGER NULL` | sí | | `cantidad_contada − cantidad_sistema` (calculada al cerrar) |

**Constraints adicionales:**
- `UNIQUE (conteo_id, producto_id, ubicacion_id)` (NULLS NOT DISTINCT) — combinación única.
- `CHECK (estado <> 'CERRADO' OR cantidad_contada IS NOT NULL)` a nivel de línea no aplica; la validación se hace en el servicio.

## Transacción de cierre

```
BEGIN;
  SELECT * FROM conteos WHERE id = $id FOR UPDATE;
  -- validar estado ABIERTO
  -- validar que todas las líneas tengan cantidad_contada NOT NULL
  -- por línea: recalcular cantidad_sistema = stock vigente en stock_por_deposito
  --            diferencia = cantidad_contada - cantidad_sistema
  -- si diferencia != 0: aplicar delta a stock_por_deposito (asegurar fila)
  -- insertar movimiento AJUSTE (motivo INVENTARIO) si hay diferencias
  -- insertar movimiento_lineas por cada diff != 0
  -- recalcular productos.stock_actual
  -- actualizar conteo: estado=CERRADO, cerrado_en=now(), movimiento_ajuste_id
  -- auditar (tabla_ref='conteos', accion='CERRAR' o ACTUALIZAR)
COMMIT;
```