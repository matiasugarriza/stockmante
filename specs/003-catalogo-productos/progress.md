# Progreso de implementación — Catálogo de productos y categorías (003)

**Spec**: [spec.md](./spec.md) | **Actualizado**: 2026-10-25 | **Estado**: Replanteo pendiente — WIP de backend deshecho

> Espejo local de reanudación del caso de uso US3.
>
> **El código, los tests y la migración descritos abajo ya no están en `main`**: se quitaron del
> árbol el 2026-10-25 y se conservan en la rama `wip/003-catalogo` (commit `941b0cf`). Este
> archivo queda como referencia del diseño anterior hasta el replanteo.

## Último estado verificado (2026-10-25)

- **`main`**: sin código de 003. `cargo check --all-targets` con `SQLX_OFFLINE=true` en verde.
- **Rama de respaldo**: `wip/003-catalogo` en `941b0cf` (modelo, servicios, tests, migración y
  lockfile con `rust_decimal`).
- **tasks.md**: T001–T003 vueltas a `[ ]`; T004–T005 sin empezar.
- **BD local**: la constraint de `productos.codigo_barras` quedó como `UNIQUE (codigo_barras)`
  (`NULLS DISTINCT`) en `stockmante` y `stockmante_test`, aplicada a mano y **sin registro en
  `_sqlx_migrations`**. **Decisión 2026-10-25: se deja aplicada** (no se revierte); el replanteo
  decide cómo formalizarla en una migración.

## Arquitectura del WIP deshecho (referencia)

### Servicios

- `backend/src/services/productos.rs`:
  - `normalizar_sku` (trim + mayúsculas) y `validar_sku` (3–30; primer char `[A-Z0-9]`; resto
    `[A-Z0-9-]`) con mensajes accionables.
  - `validar_codigo_barras`: vacío/blanco → `None`; 8–64 alfanuméricos.
  - `validar_monto`: ≥ 0, `scale() <= 2`, `< 10^10` (NUMERIC(12,2)).
  - `listar` paginado con filtros `activo` y `categoria_id` (JOIN a categorías).
  - `crear` / `actualizar` (parcial, `FOR UPDATE`, transacción) / `desactivar` / `eliminar`.
  - `mapear_error`: `23505` por constraint (`productos_sku_key`,
    `productos_codigo_barras_key`), `23503` de categoría → 422, `23503` de asignaciones →
    `ProductoConReferencias`.
- `backend/src/services/categorias.rs`: `listar` (filtro `activo`), `obtener`, `crear`,
  `actualizar` (COALESCE), `eliminar` con pre-check de productos (`FOR UPDATE`).
- `backend/src/models/producto.rs`: `Categoria`, `CategoriaBreve`, `Producto` con `Decimal`.
- `backend/src/error.rs`: `SkuDuplicado`, `CodigoBarrasDuplicado`, `ProductoConReferencias`,
  `CategoriaDuplicada`, `CategoriaConProductos` (409) con sus códigos.

### Migración 0004

El `0002` original declaraba `codigo_barras UNIQUE NULLS NOT DISTINCT`, lo que permitía un solo
producto sin código. La `0004` lo reemplaza por `UNIQUE (codigo_barras)`.

## Decisiones registradas

- Dinero con `rust_decimal` (`serde-float` + feature `rust_decimal` de SQLx): los montos viajan
  como número JSON y `10.1` conserva escala 1.
- Unicidad por mapeo de `23505` según constraint, sin pre-chequeo TOCTOU.
- Borrado físico de producto bloqueado si hay filas en `producto_ubicacion_permitida`; la FK de
  esa tabla es `ON DELETE CASCADE`, por eso el pre-check es obligatorio.
- Categoría bloqueada si tiene productos.
- `GET /productos/{id}` y `GET /categorias` con `AuthUsuario`; escritura y `GET /productos` con
  `RequiereGestion`.
- `parsear_activo` se moverá a `handlers/mod.rs` como `pub(crate)`.
- `bad` no es SKU inválido (`BAD` cumple el regex); el test usa `bad!`.
- Filename de la ruta frontend: `admin/categorias` (sin tilde).

## Siguiente paso — replantear 003

1. Revisar `spec.md` y `plan.md` con el alcance definitivo.
2. Regenerar `tasks.md` y decidir el destino de la migración de `codigo_barras` (formalizarla,
   renumerarla o revertirla al baseline de 0002).
3. Recién entonces volver a implementar; el código anterior queda en `wip/003-catalogo` por si
   sirve de base.

## Gotchas

- `psql` no está en PATH: usar `C:\Program Files\PostgreSQL\18\bin\psql.exe`.
- `cargo sqlx prepare --check` pasa sin `.sqlx/` porque no se usan macros `query!`.
- PowerShell con `cargo test 2>&1 | …` puede devolver exit 1 espurio; la fuente de verdad son las
  líneas `test result:`.
- SvelteKit 3 usa `#lib`, no `$lib`.
- La suite backend al deshacer el WIP es 55/55; los 7 tests unitarios de 003 (3 de SKU y 4 de
  unicidad) quedan en la rama de respaldo.
