# Data Model: Proveedores y compras (012)

La migración `migrations/0013_proveedores_compras.sql` ya está aplicada. **La fase 6 no agrega
migraciones**: los cambios son de validación, de manejo de errores y de forma de respuesta.

## Verificación de esquema (Principio V)

Contrastado el 2026-11-07 contra `migrations/`. En esta sesión no hubo MCP de PostgreSQL ni `psql`,
así que **no se verificó contra la BD en ejecución**; antes de tocar SQL en la fase 6 (T024) hay que
repetirlo contra la BD real.

| Objeto | Estado verificado | Fuente |
|---|---|---|
| `proveedores` (`id`, `nombre VARCHAR(120)` único, `cuit VARCHAR(11) NULL` único, `email`, `telefono`, `direccion`, `activo`, `creado_en`, `actualizado_en`) | existe | `0013` |
| `compras` (`id` identity, `proveedor_id`, `deposito_id`, `numero VARCHAR(40) NULL`, `estado VARCHAR(12)` con `CHECK` BORRADOR/CONFIRMADA/ANULADA, `observacion TEXT` ≤ 500, `total_costo NUMERIC(18,2)`, `usuario_id`, `creado_en`, `actualizado_en`, `confirmado_en NULL`, `anulado_en NULL`) | existe | `0013` |
| `compra_lineas` (`compra_id` ON DELETE CASCADE, `producto_id`, `cantidad INT CHECK > 0`, `costo_unitario NUMERIC(18,2) CHECK >= 0`, `subtotal NUMERIC(18,2)`, `orden`) | existe | `0013` |
| `movimientos.compra_id INT NULL` → `compras(id)` ON DELETE SET NULL, índice parcial | existe | `0013` |
| `movimientos.motivo` admite `COMPRA` y `DEVOLUCION` | existe | `0009` |
| `stock_por_deposito.cantidad INT CHECK (>= 0)`, `UNIQUE NULLS NOT DISTINCT (producto_id, deposito_id, ubicacion_id)` | existe | `0009` |
| `movimiento_lineas.stock_resultante INT CHECK (>= 0)`, `cantidad_delta <> 0` | existe | `0009` |
| `productos.costo NUMERIC(12,2) CHECK (>= 0)` | existe: **más chico que `costo_unitario`** (R5) | `0002` |
| `productos.activo`, `depositos.activo` | existen | `0002`, `0003` |
| Tablas `ordenes_compra`, `orden_compra_lineas`, `recepciones` (del plan anterior) | **no existen** | — |

## Entidades

### Proveedor

| Campo | Regla |
|---|---|
| `nombre` | obligatorio, recortado, ≤ 120, único → 409 `proveedor_duplicado` |
| `cuit` | opcional, exactamente 11 dígitos → 422; único en alta y edición → 409 `cuit_duplicado` |
| `email`, `telefono`, `direccion` | opcionales |
| `activo` | baja lógica; inactivo → no se usa en compras (409 `proveedor_inactivo`) |

### Compra

| Campo | Regla |
|---|---|
| `id` | número interno y secuencial de la compra (clarificación 1) |
| `numero` | nº de factura o remito del proveedor; opcional, ≤ 40 caracteres → 422 (R5); sin unicidad |
| `proveedor_id` | debe existir (404) y estar activo (409 `proveedor_inactivo`) al crear, editar y confirmar |
| `deposito_id` | debe existir (404) y estar activo (422) |
| `total_costo` | Σ `subtotal`, calculado en el servidor |
| `usuario_id` | autor tomado de la sesión, nunca del cuerpo |

### CompraLinea

| Campo | Regla |
|---|---|
| `producto_id` | existe, activo (409 `producto_inactivo`), no repetido en la compra (422) |
| `cantidad` | entero > 0 (422) |
| `costo_unitario` | 0 ≤ c ≤ 9.999.999.999,99 (422, R5) |
| `subtotal` | `cantidad × costo_unitario`, calculado en el servidor |

## Máquina de estados

```text
            crear              confirmar                 anular
  (nada) ─────────▶ BORRADOR ─────────────▶ CONFIRMADA ─────────▶ ANULADA
                     │   ▲ editar                │
                     │   └───────┘               └─ anular sin stock → 409 stock_insuficiente (sigue CONFIRMADA)
                     └─ eliminar → (borrada)
```

| Acción \ estado | BORRADOR | CONFIRMADA | ANULADA |
|---|---|---|---|
| editar / eliminar | permitido | 409 `compra_no_editable` | 409 `compra_no_editable` |
| confirmar | permitido | 409 `compra_ya_confirmada` | 409 `compra_no_editable` |
| anular | 409 `compra_no_anulable` (borrador) | permitido | 409 `compra_no_anulable` (ya anulada) |

## Efectos sobre el stock

| Transición | Movimiento | Stock | Costo |
|---|---|---|---|
| confirmar | 1 `ENTRADA`, motivo `COMPRA`, `compra_id`, sin ubicación | `+cantidad` por línea en el depósito destino; recalcula `productos.stock_actual` | `productos.costo = costo_unitario` |
| anular | 1 `SALIDA`, motivo `DEVOLUCION`, `compra_id`, sin ubicación | `-cantidad` con guarda `cantidad >= n` (R1); recalcula `stock_actual` | no se revierte |

Ambas transiciones ocurren en una única transacción, con el orden de bloqueo de 007 (R1), y
auditan `movimientos/CREAR` y `compras/CONFIRMAR|ANULAR`.

## DTOs de respuesta

- `CompraDetalle { compra: Compra, lineas: CompraLinea[] }`: `GET /{id}`, `POST /`, `PUT /{id}`. Sin
  cambios.
- **Nuevo** `ResultadoCompra { compra: Compra, lineas: CompraLinea[], movimiento: MovimientoDetalle }`:
  `POST /{id}/confirmar` y `POST /{id}/anular`. `MovimientoDetalle` es el de
  `backend/src/models/stock.rs` (007), sin cambios (R2).

## Verificación de esquema (T024, 2026-11-07)

Se aplicaron todas las migraciones a una base temporal y se comparó el esquema completo (columnas,
tipos, nulabilidad, defaults, constraints e índices) con `stockmante` y `stockmante_test`:
**idéntico**, incluidos `proveedores`, `compras`, `compra_lineas`, `movimientos.compra_id`,
`stock_por_deposito` y `productos.costo NUMERIC(12,2)`. `0013_proveedores_compras.sql` estaba
aplicada a mano (sin fila en `_sqlx_migrations`); se registró con su checksum real (ver
`specs/011-alertas-stock-minimo/progress.md`). Sin diferencias con este data-model.
