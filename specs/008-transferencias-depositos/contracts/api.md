# API Contract: Transferencias entre depósitos (008)

## POST /api/transferencias

Registra una transferencia inmediata de stock entre dos depósitos distintos.

**Permiso**: `OPERACION` o `GESTION` (familia de rol).

**Request body**:

```json
{
  "producto_id": 1,
  "deposito_origen_id": 1,
  "deposito_destino_id": 2,
  "cantidad": 8,
  "ubicacion_origen_id": null,
  "ubicacion_destino_id": null,
  "observacion": "Abastecer sucursal"
}
```

| Campo | Tipo | Requerido | Descripción |
|---|---|---|---|
| `producto_id` | `integer` | sí | Producto a transferir; debe estar activo |
| `deposito_origen_id` | `integer` | sí | Depósito de salida; debe estar activo y distinto del destino |
| `deposito_destino_id` | `integer` | sí | Depósito de entrada; debe estar activo y distinto del origen |
| `cantidad` | `integer` | sí | Unidades; debe ser mayor que cero |
| `ubicacion_origen_id` | `integer \| null` | no | Ubicación de salida; debe pertenecer al depósito origen y estar permitida para el producto |
| `ubicacion_destino_id` | `integer \| null` | no | Ubicación de entrada; debe pertenecer al depósito destino y estar permitida para el producto |
| `observacion` | `string \| null` | no | Máximo 500 caracteres |

**Respuesta 201**:

```json
{
  "id": 1,
  "producto_id": 1,
  "producto_nombre": "Café",
  "sku": "CAF-001",
  "deposito_origen": { "id": 1, "nombre": "Principal" },
  "deposito_destino": { "id": 2, "nombre": "Sucursal" },
  "cantidad": 8,
  "stock_origen_resultante": 12,
  "stock_destino_resultante": 8,
  "usuario": { "id": 3, "nombre": "Juan" },
  "creado_en": "2026-10-29T12:00:00Z",
  "movimiento_salida_id": 10,
  "movimiento_entrada_id": 11,
  "observacion": "Abastecer sucursal"
}
```

**Errores**:

| Status | Código | Mensaje |
|---|---|---|
| 401 | `no_autenticado` | `Necesitás iniciar sesión para continuar` |
| 403 | `sin_permisos` | `No tenés permiso para realizar esta acción` |
| 409 | `stock_insuficiente` | `Stock insuficiente en Principal: 3 disponibles, 5 solicitadas.` |
| 409 | `producto_inactivo` | `El producto está inactivo y no admite movimientos.` |
| 422 | `transferencia_mismo_deposito` | `El depósito de origen y el de destino deben ser distintos.` |
| 422 | `validacion` | `La cantidad transferida debe ser mayor que cero.` |

---

## GET /api/transferencias

Lista transferencias con filtros opcionales.

**Permiso**: sesión vigente.

**Query parameters**:

| Parámetro | Tipo | Descripción |
|---|---|---|
| `producto_id` | `integer` | Filtra por producto |
| `deposito_origen_id` | `integer` | Filtra por depósito de origen |
| `deposito_destino_id` | `integer` | Filtra por depósito de destino |
| `desde` | `string` | Fecha ISO 8601 (`YYYY-MM-DD` o `YYYY-MM-DDTHH:MM:SSZ`); inclusivo |
| `hasta` | `string` | Fecha ISO 8601; inclusivo |
| `pagina` | `integer` | Página (default: 1) |
| `por_pagina` | `integer` | Ítems por página (default: 20, max: 100) |

**Respuesta 200**:

```json
{
  "items": [ /* TransferenciaDetalle[] */ ],
  "total": 42,
  "pagina": 1,
  "por_pagina": 20
}
```

---

## GET /api/transferencias/{id}

Detalle de una transferencia con los dos movimientos generados.

**Permiso**: sesión vigente.

**Respuesta 200**: `TransferenciaDetalle` (mismo shape que el 201 de POST).

**Errores**:

| Status | Código | Mensaje |
|---|---|---|
| 404 | `recurso_no_encontrado` | `No existe la transferencia {id}.` |
