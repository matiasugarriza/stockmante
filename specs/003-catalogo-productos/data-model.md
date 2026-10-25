# Data Model — Catálogo de productos y categorías

**Spec**: [spec.md](./spec.md) | **Plan**: [plan.md](./plan.md) | **Creado**: 2026-10-12 | **Dividido**: 2026-10-25

> Tipos PostgreSQL; nombres en snake_case; fechas en `TIMESTAMPTZ`. Toda migración vive en
> `/migrations` (raíz). ANTES de cada migración/SQL: verificar esquema real (constitución —
> Verificación de esquema).
>
> Las tablas se crearon en la fase foundational de 001 (migración `0002`) para no bloquear el
> módulo; **la propiedad del esquema es de esta spec**.

## Diagrama de relaciones (texto)

```text
categorias 1─* productos
productos *─* ubicaciones   (005, producto_ubicacion_permitida)
```

## categorias

| Campo | Tipo | Restricciones |
|-------|------|----------------|
| id | SERIAL | PK |
| nombre | VARCHAR(80) | NOT NULL, UNIQUE |
| activo | BOOLEAN | NOT NULL DEFAULT TRUE |

Lista plana (spec Supuestos). El borrado físico solo se permite sin productos asociados.

## productos

| Campo | Tipo | Restricciones |
|-------|------|----------------|
| id | SERIAL | PK |
| nombre | VARCHAR(160) | NOT NULL, TRIM |
| sku | VARCHAR(30) | NOT NULL, UNIQUE, CHECK `^[A-Z0-9][A-Z0-9-]{2,29}$` |
| codigo_barras | VARCHAR(64) | NULL, UNIQUE (`NULLS DISTINCT`, ver 0004), alfanumérico |
| categoria_id | INT | FK → categorias(id) ON DELETE RESTRICT, NOT NULL |
| unidad_medida | VARCHAR(20) | NOT NULL (`UNIDAD`, `KG`, `CAJA`) |
| costo | NUMERIC(12,2) | NOT NULL, CHECK (costo >= 0) |
| precio_venta | NUMERIC(12,2) | NOT NULL, CHECK (precio_venta >= 0) |
| stock_minimo | INT | NOT NULL DEFAULT 0, CHECK (stock_minimo >= 0) |
| activo | BOOLEAN | NOT NULL DEFAULT TRUE |
| creado_en / actualizado_en | TIMESTAMPTZ | NOT NULL DEFAULT now() |

### Reglas

- **SKU**: se normaliza (trim + mayúsculas) antes de validar e insertar; unicidad global, incluso
  frente a productos inactivos, para no reutilizar el SKU de un producto histórico.
- **Código de barras**: opcional; vacío/blanco → `NULL`; si se informa, único entre todos los
  productos. Se puede limpiar con `PUT` enviando `""`.
- **Montos**: `NUMERIC(12,2)`; máximo 2 decimales y menos de `10^10` (10 dígitos enteros).
- **Borrado físico**: solo sin asignaciones en `producto_ubicacion_permitida`; con referencias →
  409 y se ofrece baja lógica.
- **Baja lógica**: `activo = FALSE`; el producto sale del catálogo activo y conserva historial.
- **Categorías**: FK `ON DELETE RESTRICT`; el servicio además pre-chequea y devuelve 409
  `categoria_con_productos`.
- **Precio y baja**: solo familia GESTION (FR-014).

## Migración 0004

`0004_productos_codigo_barras_null_distinct.sql` corrige el constraint original:

```sql
ALTER TABLE productos DROP CONSTRAINT productos_codigo_barras_key;
ALTER TABLE productos ADD CONSTRAINT productos_codigo_barras_key UNIQUE (codigo_barras);
```

Con `NULLS DISTINCT` (default de PostgreSQL) varios productos pueden no tener código.

## Índices

- `productos(sku)` UNIQUE
- `productos(codigo_barras)` UNIQUE
- `productos(categoria_id)` (para el JOIN y el borrado de categorías)
- `categorias(nombre)` UNIQUE
- El índice trigram de nombre es de 004 (migración `0005`).

## Validaciones de integridad (app + BD)

| Regla | Nivel |
|-------|-------|
| SKU normalizado + único + regex | BD CHECK + UNIQUE + app |
| Código de barras único | BD UNIQUE + app (mapeo 23505) |
| Código de barras opcional | `NULL` permitido; normalización app |
| Categoría existente | FK + mapeo `23503` → 422 |
| Producto con referencias | Servicio (`FOR UPDATE` + conteo) + fallback `23503` |
| Categoría con productos | Servicio + FK `RESTRICT` |
| Montos no negativos y 2 decimales | BD CHECK + app |
| Stock mínimo entero ≥ 0 | BD CHECK + app |
