# API Contracts — Catálogo de productos y categorías

**Base**: `/api` | **Formato**: JSON (UTF-8) | **Auth**: cookie de sesión `HttpOnly` |
**Creado**: 2026-10-12 | **Revisado**: 2026-10-26 (plan v2)

El sobre de error, la cookie de sesión y los guards son el **contrato base** de
[001/contracts/api.md](../../001-autenticacion-sesion/contracts/api.md).

> **Cambios del replanteo**: no existe `DELETE` (405), `activo` no se acepta en `PUT`, se agregan
> `GET /categorias/{id}`, `descripcion`, `reactivar`, y los filtros `q` / `categoria_id`.
> El estado por defecto de los listados es **solo activos**.

## Categorías

Lectura: cualquier sesión válida. Escritura: familia `GESTION`.

| Method | Path | Auth | Body / Query | Éxito | Errores clave |
|--------|------|------|--------------|-------|----------------|
| GET | `/categorias` | sesión | `?activo=true\|false\|all` (default `true`) | 200 `[Categoria]` | 401; 422 |
| POST | `/categorias` | gestión | `{ nombre, descripcion? }` | 201 `Categoria` | 409 `categoria_duplicada`; 422 |
| GET | `/categorias/{id}` | sesión | — | 200 `Categoria` | 401; 404 |
| PUT | `/categorias/{id}` | gestión | `{ nombre?, descripcion? }` | 200 `Categoria` | 409; 422; 404 |
| POST | `/categorias/{id}/desactivar` | gestión | — | 204 | 404 |
| POST | `/categorias/{id}/reactivar` | gestión | — | 204 | 404 |
| DELETE | `/categorias/{id}` | — | — | **405** | — |

**Item categoría**:

```json
{
  "id": 2,
  "nombre": "Almacén",
  "descripcion": "Productos secos",
  "activo": true
}
```

Reglas del contrato:

- Lista **plana**, ordenada por `nombre` ascendente. Sin paginación: el volumen de categorías es
  acotado y el selector de productos las necesita completas.
- `descripcion` es opcional; `""` o solo espacios se normaliza a `null`.
- `PUT` con `activo` → `422 validacion` (mensaje: el estado se cambia con `/desactivar` o
  `/reactivar`). `PUT` con cuerpo `{}` → `422 validacion`.
- `desactivar` y `reactivar` son **idempotentes**: repetir la operación devuelve 204.
- `DELETE` no está registrado: Axum responde 405 con `Allow: GET, HEAD, PUT` y **sin cuerpo JSON**.

## Productos

| Method | Path | Auth | Body / Query | Éxito | Errores clave |
|--------|------|------|--------------|-------|----------------|
| GET | `/productos` | gestión | `?q=&categoria_id=&activo=&pagina=&por_pagina=` | 200 `ListaProductos` | 403; 422 |
| POST | `/productos` | gestión | `{ nombre, sku, codigo_barras?, categoria_id, unidad_medida, costo, precio_venta, stock_minimo }` | 201 `Producto` | 409 `sku_duplicado` / `codigo_barras_duplicado`; 422 |
| GET | `/productos/{id}` | sesión | — | 200 `Producto` | 401; 404 |
| PUT | `/productos/{id}` | gestión | mismos campos, todos opcionales; `codigo_barras: ""` limpia | 200 `Producto` | 409; 422; 404 |
| POST | `/productos/{id}/desactivar` | gestión | — | 204 | 404 |
| POST | `/productos/{id}/reactivar` | gestión | — | 204 | 404 |
| DELETE | `/productos/{id}` | — | — | **405** | — |

**ListaProductos**:

```json
{ "items": [], "total": 0, "pagina": 1, "por_pagina": 20 }
```

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

Reglas del contrato:

- `q` busca subcadena en `nombre` **o** `sku`, sin distinguir mayúsculas, con `trim`. Se combina
  con `categoria_id` y `activo` por AND. Se recorta a 160 caracteres; más → 422.
- `activo` ausente → solo activos; `true` / `false` / `all` explícitos. Valor inválido → 422.
- `categoria_id` ∈ entero > 0; si el tipo es inválido o el número no es positivo → 422. Un
  `categoria_id` inexistente en el filtro devuelve lista vacía (no 404).
- Orden estable: `nombre` ascendente, desempate por `id`.
- `pagina` ≥ 1; `por_pagina` 1–100 (default 20).
- Los montos viajan como **número JSON** (no string) con hasta 2 decimales.
- `codigo_barras` es `null` cuando no se informó; varios productos pueden tenerlo en `null`.
- `PUT` con `activo` → `422 validacion`. `PUT` con cuerpo `{}` → `422 validacion`.
- `stock_minimo` omitido en el alta se interpreta como `0`; el frontend siempre lo envía.
- `desactivar` y `reactivar` son idempotentes (204) y no tocan ningún otro campo.
- `DELETE` no está registrado: 405 con `Allow: GET, HEAD, PUT`, sin cuerpo.

## Errores de dominio

| Situación | Status | `codigo` | Mensaje |
|-----------|--------|----------|---------|
| SKU repetido | 409 | `sku_duplicado` | Ya existe un producto con el SKU {sku}. Elegí otro. |
| Código de barras repetido | 409 | `codigo_barras_duplicado` | Ya existe un producto con ese código de barras. |
| Nombre de categoría repetido | 409 | `categoria_duplicada` | Ya existe una categoría con ese nombre. |
| Categoría inexistente o inactiva al asignar | 422 | `validacion` | La categoría indicada no existe o está dada de baja. |
| Unidad de medida fuera de la lista | 422 | `validacion` | La unidad de medida debe ser UNIDAD, KG, METRO o CAJA. |
| Monto negativo, con más de 2 decimales o ≥ 10¹⁰ | 422 | `validacion` | Mensaje por campo. |
| `stock_minimo` negativo o no entero | 422 | `validacion` | El stock mínimo debe ser un entero mayor o igual a 0. |
| `activo` en `PUT` | 422 | `validacion` | El estado se cambia con `/desactivar` o `/reactivar`. |
| Cuerpo `PUT` vacío | 422 | `validacion` | Indicá al menos un campo para actualizar. |
| Payload o query inválidos | 422 | `validacion` | Mensaje accionable por campo. |
| Método `DELETE` | 405 | — | Sin cuerpo; `Allow: GET, HEAD, PUT`. |
| Recurso inexistente | 404 | `recurso_no_encontrado` | — |
| Sin sesión | 401 | `no_autenticado` | Necesitás iniciar sesión para continuar. |
| Familia OPERACION en escritura o en `GET /productos` | 403 | `sin_permisos` | No tenés permiso para realizar esta acción. |
| Temporal pendiente | 403 | `cambio_password_requerido` | Debés cambiar tu contraseña temporal antes de continuar. |

## Matriz de acceso

| Acción | OPERACION | GESTION |
|---|---|---|
| `GET /categorias`, `GET /categorias/{id}` | ✅ | ✅ |
| `GET /productos/{id}` | ✅ | ✅ |
| `GET /productos` (listado de gestión) | 403 | ✅ |
| Escrituras de categorías y productos | 403 | ✅ |

## Fuera de este contrato

- `GET /productos/buscar` y la exclusión de inactivos en caja → [004](../../004-busqueda-caja/spec.md).
- `PUT /productos/{id}/ubicaciones` → [005](../../005-depositos-ubicaciones/spec.md).
- Verificación transversal de la matriz RBAC → [006](../../006-rbac-auditoria/spec.md).
- El rechazo de movimientos sobre productos inactivos lo implementa el módulo de stock (futuro);
  esta spec solo fija el contrato (FR-022).
