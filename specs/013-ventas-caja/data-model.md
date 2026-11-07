# Data Model: Ventas de caja y clientes (013)

La migración `migrations/0014_ventas.sql` **ya está aplicada** (T002 del plan anterior). Esta fase
no agrega migraciones: completa validaciones, permisos, respuesta y pantallas.

## Verificación de esquema (Principio V)

Contrastado el 2026-11-07 contra `migrations/` y contra el código que compila. En esta sesión no
hubo MCP de PostgreSQL ni `psql`, así que **no se verificó contra la base en ejecución**; antes de
tocar SQL hay que repetir la verificación contra la BD real (T001).

| Objeto | Estado verificado | Fuente |
|---|---|---|
| `clientes` (`id` identity, `nombre VARCHAR(120) NOT NULL`, `documento VARCHAR(20) NULL` con `UNIQUE (documento)`, `email VARCHAR(200) NULL`, `telefono VARCHAR(60) NULL`, `direccion VARCHAR(500) NULL`, `activo BOOLEAN`, `creado_en`, `actualizado_en`) | existe | `0014` |
| `ventas` (`id` identity, `numero VARCHAR(20) UNIQUE`, `deposito_id` FK RESTRICT, `cliente_id` FK SET NULL, `medio_pago VARCHAR(20)` `CHECK` EFECTIVO/TARJETA/TRANSFERENCIA, `subtotal`/`descuento`/`total NUMERIC(18,2)`, `estado VARCHAR(12)` `CHECK` CONFIRMADA/ANULADA, `observacion TEXT` ≤ 500, `usuario_id` FK RESTRICT, `creado_en`, `actualizado_en`, `anulado_en NULL`) | existe | `0014` |
| `venta_lineas` (`id`, `venta_id` ON DELETE CASCADE, `producto_id` RESTRICT, `cantidad INT CHECK > 0`, `precio_unitario NUMERIC(18,2)`, `subtotal NUMERIC(18,2)`) | existe | `0014` |
| Secuencia `ventas_numero_seq` | existe | `0014` |
| `movimientos.venta_id INT NULL` → `ventas(id)` ON DELETE SET NULL, índice parcial | existe | `0014` |
| `stock_por_deposito.cantidad INT CHECK (>= 0)`, `UNIQUE NULLS NOT DISTINCT (producto_id, deposito_id, ubicacion_id)` | existe | `0009` |
| `movimiento_lineas.stock_resultante INT CHECK (>= 0)`, `cantidad_delta <> 0` | existe | `0009` |
| `productos.precio_venta NUMERIC(12,2) NOT NULL CHECK (>= 0)` | existe | `0002` |
| `usuarios.familia_rol` (`GESTION`/`OPERACION`) | existe | `0001`/`0004` |

**Sin cambios de esquema.** La unicidad de `documento` es global (también entre inactivos); no se
crea índice parcial (R4).

## Entidades

### Cliente

| Campo | Regla |
|---|---|
| `nombre` | obligatorio, recortado, ≤ 120 → 422 `validacion` |
| `documento` | opcional, recortado, ≤ 20; único → **409 `documento_duplicado`** con `"Ya existe un cliente con ese documento."` |
| `email`, `telefono`, `direccion` | opcionales, ≤ 200 / 60 / 500 |
| `activo` | baja lógica; las ventas históricas siguen mostrando el cliente |
| Permisos | lectura: cualquier sesión; alta/edición/baja/reactivación: `GESTION` |

### Venta

| Campo | Regla |
|---|---|
| `numero` | `V-000001` desde `ventas_numero_seq`, único; se asigna dentro de la transacción |
| `deposito_id` | existe (404) y activo (422) |
| `cliente_id` | opcional; si viene, existe (404) y activo (422) |
| `medio_pago` | `EFECTIVO`/`TARJETA`/`TRANSFERENCIA`, normalizado a mayúsculas → 422 |
| `subtotal` | Σ `cantidad × precio_unitario` de las líneas ya agregadas, calculado en el servidor |
| `descuento` | `0 ≤ descuento ≤ subtotal` → 422 si no |
| `total` | `subtotal - descuento`, calculado en el servidor |
| `estado` | `CONFIRMADA` al crear; `ANULADA` solo por anulación |
| `observacion` | opcional, ≤ 500 |
| `usuario_id` | autor tomado de la sesión, nunca del cuerpo |
| Permisos | crear: `OPERACION`/`GESTION`; listar y ver: cualquier sesión; anular: `GESTION` |

### VentaLinea

| Campo | Regla |
|---|---|
| `producto_id` | existe (404) y activo → 422 `"El producto no está disponible para la venta."` |
| `cantidad` | entero > 0 → 422; líneas del mismo producto se **suman** (R2) |
| `precio_unitario` | **siempre** `productos.precio_venta`; el request no puede enviarlo → 422 (R1) |
| `subtotal` | `cantidad × precio_unitario`, calculado en el servidor |
| `stock_resultante` | **nuevo** campo de respuesta: saldo del producto en el depósito tras la última operación de la venta (`movimiento_lineas.stock_resultante` del movimiento `SALIDA` al crear y del `ENTRADA` si está anulada), según Escenario 13.1 |

## Máquina de estados

```text
             crear (POST /api/ventas)
   (nada) ─────────────────────────▶ CONFIRMADA ──── anular (GESTION) ────▶ ANULADA
                                        │                                     │
                                        └── anular dos veces ──▶ 409 venta_ya_anulada (sigue ANULADA)
```

| Acción \ estado | CONFIRMADA | ANULADA |
|---|---|---|
| consultar | permitido | permitido |
| anular | permitido (`GESTION`) | 409 `venta_ya_anulada` |
| editar / eliminar | no existe (venta inmutable) | no existe |

## Efectos sobre el stock

| Transición | Movimiento | Stock |
|---|---|---|
| crear | 1 `SALIDA`, motivo `VENTA`, `venta_id` | `-cantidad` **agregada** por producto en el depósito; recalcula `productos.stock_actual` |
| anular | 1 `ENTRADA`, motivo `DEVOLUCION`, `venta_id` | `+cantidad` por línea persistida (una por producto); recalcula `stock_actual` |

Ambas transiciones ocurren en una única transacción con el orden de bloqueo de 007
(`depositos` → `productos ORDER BY id` → `stock_por_deposito`) y auditan
`movimientos/CREAR` y `ventas/CREAR|ANULAR`.

## DTOs

- **Entrada** `VentaDto { deposito_id, cliente_id?, medio_pago, descuento?, observacion?, lineas: [{ producto_id, cantidad, precio_unitario? }] }`:
  `precio_unitario` se acepta en el tipo solo para poder rechazarlo con 422 (R1).
- **Salida** `VentaDetalle { venta: Venta, lineas: VentaLinea[] }`; `VentaLinea` agrega
  `stock_resultante: Option<i32>`.
- `ListaVentas { items, total, pagina, por_pagina }`, `ListaClientes` con la misma forma y
  paginación estándar (`normalizar_paginacion`: 50 por defecto, máximo 100).

## Catálogo de errores de la feature

| Código | HTTP | Cuándo |
|---|---|---|
| `stock_insuficiente` | 409 | cantidad agregada por producto > saldo del depósito |
| `venta_ya_anulada` | 409 | anular una venta `ANULADA` |
| `documento_duplicado` | 409 | alta/edición de cliente con documento existente |
| `validacion` | 422 | precio enviado, producto inactivo, medio de pago, descuento, cantidades, longitudes |
| `recurso_no_encontrado` | 404 | venta, cliente, depósito o producto inexistente |
| `sin_permisos` | 403 | anular o escribir clientes sin familia `GESTION` |
