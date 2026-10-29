# Data Model: Transferencias entre depósitos (008)

## Entidad: `transferencias`

| Campo | Tipo | Nullable | Constraints | Descripción |
|---|---|---|---|---|
| `id` | `SERIAL PRIMARY KEY` | no | | Identificador único |
| `producto_id` | `INTEGER NOT NULL` | no | FK → `productos(id)` | Producto transferido |
| `deposito_origen_id` | `INTEGER NOT NULL` | no | FK → `depositos(id)` | Depósito de salida |
| `deposito_destino_id` | `INTEGER NOT NULL` | no | FK → `depositos(id)` | Depósito de entrada |
| `cantidad` | `INTEGER NOT NULL` | no | `CHECK (cantidad > 0)` | Unidades transferidas |
| `ubicacion_origen_id` | `INTEGER NULL` | sí | FK → `ubicaciones(id)` | Ubicación de salida (opcional) |
| `ubicacion_destino_id` | `INTEGER NULL` | sí | FK → `ubicaciones(id)` | Ubicación de entrada (opcional) |
| `movimiento_salida_id` | `INTEGER NOT NULL` | no | FK → `movimientos(id)` | Movimiento `SALIDA` generado |
| `movimiento_entrada_id` | `INTEGER NOT NULL` | no | FK → `movimientos(id)` | Movimiento `ENTRADA` generado |
| `usuario_id` | `INTEGER NOT NULL` | no | FK → `usuarios(id)` | Autor de la transferencia |
| `observacion` | `TEXT NULL` | sí | `CHECK (length(observacion) <= 500)` | Nota libre |
| `creado_en` | `TIMESTAMPTZ NOT NULL` | no | `DEFAULT now()` | Timestamp de creación |

**Constraints adicionales:**
- `CHECK (deposito_origen_id <> deposito_destino_id)` — refuerzo a nivel BD de la regla de negocio.
- `UNIQUE (movimiento_salida_id)` y `UNIQUE (movimiento_entrada_id)` — cada movimiento pertenece a una sola transferencia.

## Cambio en `movimientos`

| Campo | Tipo | Nullable | Constraints | Descripción |
|---|---|---|---|---|
| `transferencia_id` | `INTEGER NULL` | sí | FK → `transferencias(id)` | Transferencia que generó este movimiento (NULL si no aplica) |

## Relaciones

```
transferencias ──1──N── movimientos (salida)
transferencias ──1──N── movimientos (entrada)
transferencias ──N──1── productos
transferencias ──N──1── depositos (origen)
transferencias ──N──1── depositos (destino)
transferencias ──N──1── usuarios
```

## Reglas de validación (modelos)

| Regla | Error | Status |
|---|---|---|
| `deposito_origen_id ≠ deposito_destino_id` | `transferencia_mismo_deposito` | 422 |
| `cantidad > 0` | `validacion` | 422 |
| Producto existe y está activo | `producto_inactivo` | 409 |
| Depósito origen activo | `validacion` | 422 |
| Depósito destino activo | `validacion` | 422 |
| Stock suficiente en origen | `stock_insuficiente` | 409 |
| Ubicación origen pertenece al depósito origen y está permitida | `validacion` | 422 |
| Ubicación destino pertenece al depósito destino y está permitida | `validacion` | 422 |

## Estados

La transferencia no tiene estados intermedios: es inmediata (supuesto de la spec). No hay
"en tránsito" ni recepción diferida.

## Transacción

```
BEGIN;
  -- 1. Bloquear depósitos (origen y destino) por id ASC
  SELECT * FROM depositos WHERE id IN ($origen, $destino) ORDER BY id FOR UPDATE;

  -- 2. Bloquear productos por id ASC
  SELECT * FROM productos WHERE id = $producto FOR UPDATE;

  -- 3. Bloquear stock origen y destino
  SELECT * FROM stock_por_deposito
   WHERE producto_id = $producto AND deposito_id IN ($origen, $destino)
   ORDER BY deposito_id FOR UPDATE;

  -- 4. Validar stock suficiente en origen
  -- 5. Actualizar stock origen (descuento)
  -- 6. Actualizar stock destino (incremento)

  -- 7. Insertar movimiento SALIDA (deposito_id = origen, motivo = TRANSFERENCIA)
  -- 8. Insertar movimiento ENTRADA (deposito_id = destino, motivo = TRANSFERENCIA)
  -- 9. Insertar línea de cada movimiento
  -- 10. Insertar transferencia (enlaza los dos movimientos)
  -- 11. Recalcular productos.stock_actual = SUM(stock_por_deposito.cantidad)
  -- 12. Auditar (tabla_ref = 'transferencias', accion = CREAR)
COMMIT;
```
