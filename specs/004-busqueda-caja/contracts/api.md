# API Contracts — Búsqueda de caja (004)

**Base**: `/api` | **Auth**: cookie `stockmante_sesion` | **Errores**: `{ error: { codigo, mensaje } }`

## GET `/api/productos/buscar`

Búsqueda de productos para caja y almacén. Cualquier sesión válida (GESTION u OPERACION).

### Query

| Parámetro | Tipo | Requerido | Default | Reglas |
|---|---|---|---|---|
| `q` | string | condicional | — | `trim`; 1–160; vacío solo si viene `categoria_id` |
| `campo` | string | no | `todos` | `sku`, `codigo`, `nombre`, `categoria`, `todos` |
| `incluir_inactivos` | bool | no | `false` | `true` solo GESTION → si no, 403 |
| `categoria_id` | int | no | — | entero > 0 |
| `limite` | int | no | 20 | 1–50 |

### Comportamiento

- `campo=sku`: igualdad exacta contra `productos.sku` (el SKU se guarda normalizado en
  mayúsculas). Admite `q` desde 1 carácter.
- `campo=codigo`: igualdad exacta contra `productos.codigo_barras`. Admite `q` desde 1 carácter.
- `campo=nombre`: `nombre % q` (trigramas) **o** `nombre ILIKE '%q%'`; exige `q` ≥ 2.
- `campo=categoria`: compara contra `categorias.nombre`; exige `q` ≥ 2.
- `campo=todos` (default): SKU exacto → código exacto → nombre por similitud. Con `q` de 1
  carácter solo evalúa las igualdades exactas.
- `q` vacío + `categoria_id`: lista los productos de esa categoría (filtro sin texto).
- `incluir_inactivos=false`: solo `activo=true`. `true`: incluye inactivos (GESTION).
- El filtro `categoria_id` se combina con el término mediante AND.

### Orden

| Campo | Orden |
|---|---|
| `todos` | `sku = q` (0), `codigo_barras = q` (1), resto (2); luego `similarity(nombre, q) DESC`, `nombre ASC`, `id ASC` |
| `sku` / `codigo` | `nombre ASC`, `id ASC` |
| `nombre` | `similarity(nombre, q) DESC`, `nombre ASC`, `id ASC` |
| `categoria` | `similarity(categoria.nombre, q) DESC`, `nombre ASC`, `id ASC` |

### Respuesta 200

```json
{
  "items": [
    {
      "id": 12,
      "nombre": "Yerba 1 kg",
      "sku": "YERBA-1K",
      "codigo_barras": "7791234567890",
      "categoria": { "id": 3, "nombre": "Almacén" },
      "unidad_medida": "UNIDAD",
      "costo": 1200.5,
      "precio_venta": 1800.0,
      "stock_minimo": 6,
      "activo": true
    }
  ]
}
```

Sin coincidencias: `200 { "items": [] }`.

### Errores

| Status | Código | Cuándo | Mensaje |
|---|---|---|---|
| 401 | `no_autenticado` / `sesion_expirada` | sin cookie o sesión vencida | contrato de 001 |
| 403 | `sin_permisos` | `incluir_inactivos=true` con familia OPERACION | «No tenés permiso para realizar esta acción» |
| 403 | `cambio_password_requerido` | contraseña temporal vigente | contrato de 001 |
| 422 | `validacion` | `q` vacío sin categoría | «Ingresá un término de búsqueda o elegí una categoría.» |
| 422 | `validacion` | `q` > 160 | «El término de búsqueda no puede superar los 160 caracteres.» |
| 422 | `validacion` | nombre/categoría con < 2 | «La búsqueda por nombre o categoría requiere al menos 2 caracteres.» |
| 422 | `validacion` | `campo` desconocido | «El campo de búsqueda debe ser sku, codigo, nombre, categoria o todos.» |
| 422 | `validacion` | `categoria_id` ≤ 0 | «El filtro categoria_id debe ser un entero mayor a 0.» |
| 422 | `validacion` | `limite` fuera de 1–50 | «El límite debe estar entre 1 y 50.» |

## Matriz de acceso

| Ruta | Anónimo | OPERACION | GESTION |
|---|---|---|---|
| `GET /api/productos/buscar` (default) | 401 | 200 | 200 |
| `GET /api/productos/buscar?incluir_inactivos=true` | 401 | 403 | 200 |

## Fuera de este contrato

- CRUD del catálogo (003).
- Movimientos de stock (módulos posteriores).
- Búsqueda difusa con corrección ortográfica o sin acentos (`unaccent`): mejora futura.
