# Contratos de API: Proveedores y compras (012)

Base: `/api`. Todas las rutas exigen cookie de sesión (`stockmante_sesion`) y familia `GESTION`;
`OPERACION` recibe **403** `sin_permisos` (escenario 12.7). Formato de error existente:
`{ "error": { "codigo": "<codigo>", "mensaje": "<texto en español>" } }`.

Las marcas **[fase 6]** señalan lo que cambia respecto de lo ya implementado (T001–T023).

## Proveedores

| Método | Ruta | Éxito | Errores |
|---|---|---|---|
| GET | `/proveedores?activo=true\|false\|all&pagina=&por_pagina=` | 200 página `{ items, total, pagina, por_pagina }` | 422 paginación |
| POST | `/proveedores` | 201 `Proveedor` | 409 `proveedor_duplicado`, 409 `cuit_duplicado`, 422 `validacion` |
| GET | `/proveedores/{id}` | 200 `Proveedor` | 404 |
| PUT | `/proveedores/{id}` | 200 `Proveedor` (DTO completo) | 404, 409 `proveedor_duplicado`, 409 `cuit_duplicado`, 422 |
| POST | `/proveedores/{id}/desactivar` | 200 `Proveedor` con `activo=false` | 404 |
| POST | `/proveedores/{id}/activar` | 200 `Proveedor` con `activo=true` | 404 |

Cuerpo de alta/edición: `{ nombre, cuit?, email?, telefono?, direccion? }`.

## Compras

| Método | Ruta | Éxito | Errores |
|---|---|---|---|
| GET | `/compras?proveedor_id=&estado=&desde=&hasta=&pagina=&por_pagina=` | 200 `ListaCompras` | **[fase 6]** 422 `validacion` si `estado` ∉ {BORRADOR, CONFIRMADA, ANULADA} o `desde`/`hasta` no son `AAAA-MM-DD` ni RFC 3339 |
| POST | `/compras` | 201 `CompraDetalle` en `BORRADOR` | 404 proveedor/depósito/producto, **[fase 6]** 409 `proveedor_inactivo`, 409 `producto_inactivo`, 422 `validacion` |
| GET | `/compras/{id}` | 200 `CompraDetalle` | 404 |
| PUT | `/compras/{id}` | 200 `CompraDetalle` | 404, 409 `compra_no_editable`, **[fase 6]** 409 `proveedor_inactivo`, 422 |
| DELETE | `/compras/{id}` | 204 | 404, 409 `compra_no_editable` |
| POST | `/compras/{id}/confirmar` | **[fase 6]** 200 `ResultadoCompra` en `CONFIRMADA` | 404, 409 `compra_ya_confirmada`, 409 `compra_no_editable` (anulada), **[fase 6]** 409 `proveedor_inactivo`, 409 `producto_inactivo`, 422 depósito inactivo |
| POST | `/compras/{id}/anular` | **[fase 6]** 200 `ResultadoCompra` en `ANULADA` | 404, 409 `stock_insuficiente`, **[fase 6]** 409 `compra_no_anulable` |

Cuerpo de alta/edición:

```json
{
  "proveedor_id": 3,
  "deposito_id": 1,
  "numero": "A-0001-00012345",
  "observacion": "Entrega parcial del pedido de marzo",
  "lineas": [{ "producto_id": 12, "cantidad": 10, "costo_unitario": 1500.00 }]
}
```

Validaciones **[fase 6]**: `numero` ≤ 40 caracteres; `costo_unitario` entre 0 y 9.999.999.999,99.
Fuera de rango → 422 `validacion` nombrando el campo.

### `ResultadoCompra` **[fase 6]**

```json
{
  "compra": { "id": 7, "estado": "CONFIRMADA", "numero": "A-0001-00012345", "deposito_nombre": "Depósito Central", "total_costo": 15000.0, "...": "..." },
  "lineas": [{ "producto_id": 12, "cantidad": 10, "costo_unitario": 1500.0, "subtotal": 15000.0, "...": "..." }],
  "movimiento": {
    "id": 98,
    "tipo": "ENTRADA",
    "motivo": "COMPRA",
    "deposito": { "id": 1, "nombre": "Depósito Central" },
    "observacion": "Compra #7",
    "usuario": { "id": 2, "nombre": "Gerente" },
    "creado_en": "2026-11-07T14:00:00Z",
    "lineas": [
      { "producto_id": 12, "producto_nombre": "Cemento 50 kg", "sku": "CEM-50", "ubicacion_id": null, "ubicacion_nombre": null, "cantidad_delta": 10, "stock_resultante": 24 }
    ]
  }
}
```

`movimiento` es el `MovimientoDetalle` de 007 sin cambios. En la anulación `tipo` es `SALIDA`,
`motivo` es `DEVOLUCION` y `cantidad_delta` es negativo.

### Mensajes de error nuevos **[fase 6]**

| Código | HTTP | Mensaje |
|---|---|---|
| `compra_no_anulable` | 409 | `La compra está en borrador; no hay stock que revertir. Si no la vas a usar, eliminala.` |
| `compra_no_anulable` | 409 | `La compra ya fue anulada; no se puede anular dos veces.` |
| `proveedor_inactivo` | 409 | `El proveedor «<nombre>» está dado de baja; reactivalo o elegí otro proveedor.` |
| `validacion` | 422 | `El parámetro 'desde' debe tener formato AAAA-MM-DD.` (ídem `hasta`) |
| `validacion` | 422 | `El estado debe ser BORRADOR, CONFIRMADA o ANULADA.` |
| `validacion` | 422 | `El número de factura o remito no puede superar los 40 caracteres.` |
| `validacion` | 422 | `El costo unitario no puede superar 9.999.999.999,99.` |
