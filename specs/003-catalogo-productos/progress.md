# Progreso de implementación — Catálogo de productos y categorías (003)

**Spec**: [spec.md](./spec.md) | **Actualizado**: 2026-10-25 | **Estado**: En curso (backend de servicios; handlers y frontend pendientes)

> Espejo local de reanudación del caso de uso US3.

## Último estado verificado (2026-10-25)

- **tasks.md**: T001–T003 `[X]`; T004 `[ ]` (tests RED escritos); T005 `[ ]`.
- **Tests backend**:
  - `test_sku` 3/3 en verde (normalización y regex).
  - `test_producto_unicidad` 4/4 en verde (SKU duplicado, código duplicado, múltiples `NULL`,
    código vacío normalizado).
  - `test_productos_http` escrito en rojo: no compila hasta que existan los handlers y el target
    en `Cargo.toml`.
- **BD**: migración `0004_productos_codigo_barras_null_distinct.sql` aplicada en `stockmante` y
  `stockmante_test`. Constraint verificada como `UNIQUE (codigo_barras)` (`NULLS DISTINCT`).
- **Working tree**: cambios de 003 sin commitear (migración, servicios, modelos, tests) + el
  split de specs.

## Arquitectura implementada

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

## Siguiente paso — T004

1. Crear `backend/src/handlers/categorias.rs`: `GET` con `AuthUsuario`; `POST`/`PUT`/`DELETE` con
   `RequiereGestion`; body validado y errores de dominio.
2. Crear `backend/src/handlers/productos.rs`: `GET /productos` (gestión, `parsear_activo` +
   `categoria_id`), `GET /productos/{id}` (sesión), `POST`/`PUT`/`DELETE`/`desactivar` (gestión).
3. Mover `parsear_activo` a `handlers/mod.rs` (`pub(crate)`) y reutilizarlo en usuarios.
4. Registrar `nest("/api/productos", …)` y `nest("/api/categorias", …)` en `app.rs`.
5. Agregar `[[test]] name = "test_productos_http"` en `Cargo.toml` y hacer pasar la integración.

Luego T005 (frontend) y los gates de cierre.

## Gotchas

- `psql` no está en PATH: usar `C:\Program Files\PostgreSQL\18\bin\psql.exe`.
- `cargo sqlx prepare --check` pasa sin `.sqlx/` porque no se usan macros `query!`.
- PowerShell con `cargo test 2>&1 | …` puede devolver exit 1 espurio; la fuente de verdad son las
  líneas `test result:`.
- SvelteKit 3 usa `#lib`, no `$lib`.
- La suite backend previa al trabajo de 003 era 45/45; los targets nuevos suman 7 tests
  unitarios verdes.
