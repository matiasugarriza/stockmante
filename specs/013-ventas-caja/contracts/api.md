# Contratos de API: Ventas de caja y clientes (013)

Base: `/api`. Todas las rutas exigen cookie de sesión (`stockmante_sesion`). La matriz de
permisos es la de FR-055:

| Operación | `OPERACION` (CAJERO/OPERARIO) | `GESTION` (ADMIN/GERENTE) |
|---|---|---|
| Vender (`POST /api/ventas`) | ✅ | ✅ |
| Anular venta | ❌ 403 `sin_permisos` | ✅ |
| Listar/ver ventas | ✅ (sesión) | ✅ (sesión) |
| Listar/ver clientes | ✅ (sesión) | ✅ (sesión) |
| Crear/editar/baja/reactivar clientes | ❌ 403 `sin_permisos` | ✅ |

Formato de error (`ErrorRespuesta`): `{ "error": { "codigo": "<codigo>", "mensaje": "<texto>" } }`.
Códigos de la feature: `stock_insuficiente` (409), `venta_ya_anulada` (409),
`documento_duplicado` (409), `validacion` (422), `recurso_no_encontrado` (404), `sin_permisos`
(403), `no_autenticado`/`sesion_expirada` (401).

## POST `/api/ventas`

Registra una venta y descuenta stock en una única transacción (FR-050, FR-051). Permiso:
`OPERACION` o `GESTION`.

**Request**

```json
{
  "deposito_id": 1,
  "cliente_id": 7,
  "medio_pago": "EFECTIVO",
  "descuento": 1000,
  "observacion": "Mostrador",
  "lineas": [
    { "producto_id": 12, "cantidad": 2 },
    { "producto_id": 12, "cantidad": 1 },
    { "producto_id": 15, "cantidad": 1 }
  ]
}
```

**Reglas**

- `lineas` no vacía; `cantidad` entera > 0.
- Las líneas del mismo `producto_id` se **agregan** (2 + 1 = 3) y se persiste una sola línea con
  cantidad 3 (R2, Escenario 13.9). El precio es el de `productos.precio_venta` congelado al
  confirmar.
- Si el request incluye `precio_unitario` (en cualquier línea), responde 422 `validacion` con
  `"El precio unitario se toma del catálogo; no se puede enviar en la venta."` (R1).
- `descuento` ≥ 0 y ≤ subtotal; `total = subtotal - descuento`.
- `medio_pago` ∈ {`EFECTIVO`, `TARJETA`, `TRANSFERENCIA`}; depósito y cliente (si viene) deben
  existir y estar activos.

**201**

```json
{
  "venta": {
    "id": 31,
    "numero": "V-000031",
    "deposito_id": 1,
    "deposito_nombre": "Principal",
    "cliente_id": 7,
    "cliente_nombre": "Ana Pérez",
    "medio_pago": "EFECTIVO",
    "subtotal": 15000,
    "descuento": 1000,
    "total": 14000,
    "estado": "CONFIRMADA",
    "observacion": "Mostrador",
    "usuario_id": 4,
    "usuario_nombre": "Caja 1",
    "creado_en": "2026-11-07T12:30:00Z",
    "anulado_en": null
  },
  "lineas": [
    {
      "id": 91,
      "producto_id": 12,
      "producto_nombre": "Cemento 50 kg",
      "sku": "CEM-50",
      "cantidad": 3,
      "precio_unitario": 3000,
      "subtotal": 9000,
      "stock_resultante": 7
    },
    {
      "id": 92,
      "producto_id": 15,
      "producto_nombre": "Arena 1 m³",
      "sku": "ARE-1",
      "cantidad": 1,
      "precio_unitario": 6000,
      "subtotal": 6000,
      "stock_resultante": 4
    }
  ]
}
```

**Errores**

- 409 `stock_insuficiente`: `"Stock insuficiente en Principal: 2 disponibles, 3 solicitadas."`
  (cantidad **agregada** por producto; no se registra nada).
- 422 `validacion`: precio enviado, producto inactivo
  (`"El producto no está disponible para la venta."`), medio de pago inválido, descuento mayor al
  subtotal, cantidad ≤ 0, depósito/cliente inactivo, longitudes.
- 404 `recurso_no_encontrado`: depósito, cliente o producto inexistente.

## GET `/api/ventas`

Listado paginado (FR-054). Permiso: cualquier sesión vigente.

**Query params**

| Param | Tipo | Regla |
|---|---|---|
| `deposito_id` | int | opcional |
| `cliente_id` | int | opcional |
| `estado` | `CONFIRMADA`/`ANULADA` | opcional |
| `desde`, `hasta` | fecha `AAAA-MM-DD` o RFC 3339 | opcionales; orden `creado_en DESC` |
| `pagina` | int ≥ 1 | default 1 |
| `por_pagina` | int 1..100 | default 50 |

**200**: `{ "items": [Venta], "total", "pagina", "por_pagina" }`.

## GET `/api/ventas/{id}`

Detalle con líneas y totales (FR-054). Permiso: cualquier sesión vigente. 200 `VentaDetalle`
(misma forma que el 201, incluido `stock_resultante`); 404 si no existe.

## POST `/api/ventas/{id}/anular`

Anula la venta completa y repone stock (FR-052). Permiso: **`GESTION`**.

**200**: `VentaDetalle` con `estado: "ANULADA"`, `anulado_en` no nulo y las líneas con el
`stock_resultante` posterior a la devolución. Se genera un movimiento `ENTRADA` motivo
`DEVOLUCION`.

**Errores**

- 409 `venta_ya_anulada`: `"La venta ya está anulada; no se puede anular dos veces."`
- 404 `recurso_no_encontrado` si la venta no existe.
- 403 `sin_permisos` si el usuario no es `GESTION`.

## GET `/api/clientes`

Listado paginado (FR-049). Permiso: cualquier sesión vigente. Query: `activo` (bool),
`pagina`, `por_pagina`.

**200**

```json
{
  "items": [
    {
      "id": 7,
      "nombre": "Ana Pérez",
      "documento": "30111222",
      "email": "ana@example.com",
      "telefono": "11 5555-5555",
      "direccion": "Av. Siempre Viva 742",
      "activo": true
    }
  ],
  "total": 1,
  "pagina": 1,
  "por_pagina": 50
}
```

## GET `/api/clientes/{id}`

Permiso: cualquier sesión vigente. 200 `Cliente`; 404 si no existe.

## POST `/api/clientes`

Alta de cliente (FR-049). Permiso: **`GESTION`**. Cuerpo: `nombre` obligatorio ≤ 120;
`documento` opcional ≤ 20; `email` ≤ 200; `telefono` ≤ 60; `direccion` ≤ 500.

**201** `Cliente`.

**Errores**: 409 `documento_duplicado` con
`"Ya existe un cliente con ese documento."`; 422 `validacion` por longitudes o nombre vacío;
403 `sin_permisos`.

## PUT `/api/clientes/{id}`

Edición completa (FR-049). Permiso: **`GESTION`**. Mismos campos y validaciones que el alta;
409 `documento_duplicado` si el documento pertenece a otro cliente. 200 `Cliente`; 404.

## POST `/api/clientes/{id}/desactivar` · `/activar`

Baja lógica y reactivación (FR-049). Permiso: **`GESTION`**. 200 `Cliente`; 404. La baja no borra
ventas históricas: `cliente_id`/`cliente_nombre` siguen visibles en `GET /api/ventas/{id}`.
