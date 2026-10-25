# API Contracts — Catálogo de productos y categorías

**Base**: `/api` | **Formato**: JSON (UTF-8) | **Auth**: cookie de sesión `HttpOnly` | **Creado**: 2026-10-12 | **Dividido**: 2026-10-25

El sobre de error, la cookie de sesión y los guards son el **contrato base** de
[001/contracts/api.md](../../001-autenticacion-sesion/contracts/api.md).

## Categorías (GESTION para escritura; lectura autenticada)

| Method | Path | Auth | Body | Éxito | Errores clave |
|--------|------|------|------|-------|----------------|
| GET | `/categorias` | sesión | — | 200 lista plana | 401 |
| POST | `/categorias` | gestión | `{ nombre }` | 201 categoría | 409 categoria_duplicada; 422 |
| PUT | `/categorias/{id}` | gestión | `{ nombre?, activo? }` | 200 categoría | 409; 422; 404 |
| DELETE | `/categorias/{id}` | gestión | — | 204 si no tiene productos | 409 categoria_con_productos; 404 |

**Item categoría**: `{ id, nombre, activo }`.

## Productos

| Method | Path | Auth | Body / Query | Éxito | Errores clave |
|--------|------|------|--------------|-------|----------------|
| GET | `/productos` | gestión | `?activo=true\|false\|all`, `categoria_id`, `pagina`, `por_pagina` | 200 `{ items, total, pagina, por_pagina }` | 403; 422 |
| GET | `/productos/{id}` | sesión | — | 200 producto | 404; 401 |
| POST | `/productos` | gestión | `{ nombre, sku, codigo_barras?, categoria_id, unidad_medida, costo, precio_venta, stock_minimo }` | 201 producto | 409 sku_duplicado / codigo_barras_duplicado; 422 |
| PUT | `/productos/{id}` | gestión | mismos campos, todos opcionales; `codigo_barras: ""` limpia el código | 200 producto | 409; 422; 404 |
| DELETE | `/productos/{id}` | gestión | — | 204 si no hay referencias | 409 producto_con_referencias; 404 |
| POST | `/productos/{id}/desactivar` | gestión | — | 204 | 404 |

**Item producto**:

```json
{
  "id": 1,
  "nombre": "Yerba 1 kg",
  "sku": "YERBA-1K",
  "codigo_barras": "7791234567890",
  "categoria": { "id": 2, "nombre": "Almacén" },
  "unidad_medida": "UNIDAD",
  "costo": 1200.5,
  "precio_venta": 1800,
  "stock_minimo": 6,
  "activo": true
}
```

Los montos viajan como **número JSON** (no string) con hasta 2 decimales.

## Errores de dominio

| Situación | Status | `codigo` | Mensaje |
|-----------|--------|----------|---------|
| SKU repetido | 409 | `sku_duplicado` | Ya existe un producto con SKU {sku}. |
| Código repetido | 409 | `codigo_barras_duplicado` | Ya existe un producto con ese código de barras. |
| Borrado con asignaciones | 409 | `producto_con_referencias` | El producto tiene referencias; desactivalo en lugar de eliminarlo. |
| Categoría repetida | 409 | `categoria_duplicada` | Ya existe una categoría con ese nombre. |
| Borrado de categoría con productos | 409 | `categoria_con_productos` | La categoría tiene productos asociados. |
| Categoría inexistente en alta/edición | 422 | `validacion` | La categoría indicada no existe. |
| Payload o query inválidos | 422 | `validacion` | Mensaje accionable por campo. |
| Sin sesión | 401 | `no_autenticado` | — |
| Familia OPERACION en escritura o `GET /productos` | 403 | `sin_permisos` | — |
| Temporal pendiente | 403 | `cambio_password_requerido` | — |

## Fuera de este contrato

- `GET /productos/buscar` → [004](../../004-busqueda-caja/spec.md).
- `PUT /productos/{id}/ubicaciones` → [005](../../005-depositos-ubicaciones/spec.md).
- Verificación transversal de la matriz RBAC → [006](../../006-rbac-auditoria/spec.md).
