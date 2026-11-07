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
el catálogo tenga pocas filas. SC-002 se mide como respuesta de la API (ver
[research.md](./research.md) §7) y `EXPLAIN (ANALYZE, BUFFERS)` queda como evidencia
complementaria del plan de ejecución.

## Datos de referencia para SC-002 (ampliación 2026-11-07)

No cambia el esquema. `scripts/seed_catalogo_10000.sql` inserta datos de desarrollo que respetan
las restricciones reales de `0002`/`0006`:

| Dato | Valor | Restricción que respeta |
|---|---|---|
| Categorías | 20, `SEED4-CAT-01`…`SEED4-CAT-20` | `categorias.nombre` UNIQUE y `uq_categorias_nombre_normalizado` |
| Productos | 10.000, todos activos | — |
| `sku` | `SEED4-00001`…`SEED4-10000` | UNIQUE; `^[A-Z0-9][A-Z0-9-]{2,29}$` |
| `codigo_barras` | 13 dígitos únicos (`779` + número correlativo con relleno) | `productos_codigo_barras_key` UNIQUE |
| `nombre` | combinación de una lista fija de rubros, marcas y presentaciones + número | `VARCHAR(160) NOT NULL` |
| `categoria_id` | repartido en las 20 categorías | FK `ON DELETE RESTRICT` |
| `unidad_medida`, `costo`, `precio_venta`, `stock_minimo` | valores fijos válidos | NOT NULL y `CHECK >= 0` |

Verificación del esquema real (T021, 2026-11-07): coincide con esta tabla. La única columna NOT
NULL que no aparece arriba es `productos.stock_actual` (agregada por 0009, `INT NOT NULL DEFAULT 0`,
`CHECK >= 0`); el seed no la informa y toma el default.

El script borra primero los productos `SEED4-%` y después las categorías `SEED4-CAT-%` (en ese
orden por la FK), así que es idempotente y sirve también para limpiar. No genera movimientos,
stock por depósito ni auditoría. **Antes de ejecutarlo** se verifica el esquema real de
`productos` y `categorias` (Principio V), porque migraciones posteriores (0009–0014) pueden haber
agregado columnas NOT NULL.

## Modelo de frontend (ampliación 2026-11-07)

```ts
// #lib/utils/busquedaCaja.ts
export const LIMITE_CAJA = 20;
export function crearBusquedaDiferida(esperaMs: number): {
  programar(tarea: (esVigente: () => boolean) => Promise<void> | void): void;
  cancelar(): void;
};
export function mensajeTruncados(cantidad: number, limite: number): string | null;
```

Invariantes: solo la última tarea programada llega a ejecutarse; `esVigente()` es `false` para
toda tarea que no sea la última programada; `mensajeTruncados` devuelve texto solo si
`cantidad === limite` y `limite > 0`.
