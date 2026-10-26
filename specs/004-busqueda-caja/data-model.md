# Data Model — Búsqueda de caja (004)

**Spec**: [spec.md](./spec.md) | **Plan**: [plan.md](./plan.md)

## Alcance

No se crean tablas ni columnas. Se reutilizan `productos` y `categorias` de 003 y se agrega un
índice de apoyo.

## Delta de esquema (0007)

| Objeto | Definición | Propósito |
|---|---|---|
| Extensión `pg_trgm` | `CREATE EXTENSION IF NOT EXISTS pg_trgm` | operador `%` y `similarity()` |
| `idx_productos_nombre_trgm` | `GIN (nombre gin_trgm_ops)` | búsqueda por subcadena/similitud |

No hay migración de datos. `CREATE INDEX` es transaccional en PostgreSQL; en un catálogo grande
puede requerir `CONCURRENTLY` fuera de migración, pero el Módulo 1 arranca con pocos productos.

## Entidades existentes relevantes

### `productos`

| Columna | Uso en 004 |
|---|---|
| `id` | desempate estable |
| `nombre` | `%` / `ILIKE` + ranking |
| `sku` | igualdad exacta (normalizado en mayúsculas por 003) |
| `codigo_barras` | igualdad exacta; `NULL` no matchea |
| `categoria_id` | filtro por categoría |
| `activo` | exclusión por defecto (FR-013) |

### `categorias`

| Columna | Uso en 004 |
|---|---|
| `id` | filtro `categoria_id` |
| `nombre` | búsqueda `campo=categoria` |

## Modelos de aplicación

```rust
pub enum CampoBusqueda { Sku, Codigo, Nombre, Categoria, Todos }

pub struct BusquedaProductos {
    pub q: String,                 // ya normalizado (trim)
    pub campo: CampoBusqueda,      // default Todos
    pub incluir_inactivos: bool,   // default false; el handler valida el rol
    pub categoria_id: Option<i32>, // > 0
    pub limite: i64,               // 1..=50, default 20
}

pub struct ListaBusqueda { pub items: Vec<Producto> }
```

## Invariantes

1. `q` nunca se interpola en SQL: viaja como bind; los `%`/`_` del patrón se escapan.
2. Todo producto devuelto tiene categoría existente (JOIN interno, igual que 003).
3. Con `incluir_inactivos=false`, ningún item tiene `activo=false`.
4. El orden es total: siempre termina en `p.id ASC`.
5. `limite` acota filas; no hay `OFFSET`.
6. `categoria_id` inválido (≤ 0) es 422, no una lista vacía silenciosa.

## Índices y plan esperado

- `productos_sku_key` (btree) → igualdad de SKU.
- `productos_codigo_barras_key` (btree) → igualdad de código.
- `idx_productos_categoria` (btree) → filtro por categoría.
- `idx_productos_nombre_trgm` (GIN) → `nombre % q` / `nombre ILIKE '%q%'`.

Con `limite=20`, el plan esperado es índice GIN + sort por similitud, o seq scan pequeño mientras
el catálogo tenga pocas filas. La verificación de SC-002 se hace con `EXPLAIN ANALYZE`.
