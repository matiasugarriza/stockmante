# Implementation Plan: Catálogo de productos y categorías

**Spec**: [spec.md](./spec.md) | **Creado**: 2026-10-12 | **Dividido**: 2026-10-25

**Input**: Feature specification from `/specs/003-catalogo-productos/spec.md`

## Summary

CRUD de categorías planas y de productos con SKU normalizado y único, código de barras opcional
único, montos con dos decimales y baja lógica; el borrado físico solo se permite sin referencias.
La escritura queda bajo `RequiereGestion`; la lectura de detalle, autenticada. Backend
Rust/Axum/SQLx + frontend SvelteKit SPA/Tailwind, alineado a la constitución.

## Technical Context

**Language/Version**: Rust 1.99 (backend), TypeScript 6 / SvelteKit 3 (frontend).

**Primary Dependencies**: Axum 0.7+, SQLx 0.8 (`postgres`, `runtime-tokio`, feature
`rust_decimal`), tokio, serde, `rust_decimal` con `serde-float`, tower-http, Zod 4, Tailwind.

**Storage**: PostgreSQL 18; tablas `categorias` y `productos` (migración `0002`, propiedad de
esta spec) + `0004_productos_codigo_barras_null_distinct.sql`.

**Testing**: `cargo test` con targets unitarios e integración HTTP contra `stockmante_test`;
vitest; `cargo sqlx prepare --check`; `npm run check`; `npm run build`.

**Constraints**: sin `unwrap`/`panic`/`expect` en rutas de producción; `AppError` con
409/422/403/404; SKU y código de barras únicos resueltos por constraint, no por pre-chequeo;
sin `DELETE` físico cuando hay referencias.

**Scale/Scope**: catálogo de hasta miles de productos; paginación default 20, máximo 100.

## Constitution Check

| Gate (Principio) | Estado | Evidencia |
|---|---|---|
| I — Simplicidad / YAGNI | ✅ PASS | Categorías planas; sin árbol; sin tabla de historial propia. |
| II — Mantenibilidad | ✅ PASS | `services/productos.rs` y `services/categorias.rs`; handlers delgados; modelos públicos separados. |
| III — Claridad en Mensajes | ✅ PASS | 409 `sku_duplicado`, 409 `codigo_barras_duplicado`, 409 `producto_con_referencias`, 409 `categoria_con_productos`, 422 `validacion` en español. |
| IV — Testabilidad Primero | ✅ PASS | Tests de SKU y unicidad RED→GREEN; integración HTTP; vitest de schemas/clientes. |
| V — Integridad de Datos / Errores | ✅ PASS | Unicidad por índice; borrado bloqueado con `FOR UPDATE` + fallback `23503`; transacciones en update/delete. |

**Resultado post-diseño**: PASS — sin violaciones.

## Project Structure

```text
backend/
├── src/models/producto.rs          # Categoria, CategoriaBreve, Producto
├── src/services/productos.rs       # validación, CRUD, paginación, borrado
├── src/services/categorias.rs      # CRUD plano, borrado bloqueado
├── src/handlers/productos.rs       # pendiente (T004)
├── src/handlers/categorias.rs      # pendiente (T004)
├── src/handlers/mod.rs             # parsear_activo compartido (pendiente)
├── src/app.rs                      # routers /api/productos y /api/categorias (pendiente)
└── tests/
    ├── unit/test_sku.rs
    ├── unit/test_producto_unicidad.rs
    └── integration/test_productos_http.rs

frontend/src/
├── lib/schemas/productos.ts        # pendiente (T005)
├── lib/schemas/categorias.ts
├── lib/api/productos.ts
├── lib/api/categorias.ts
├── lib/components/productos/*
└── routes/admin/{productos,categorias}/+page.svelte

migrations/
└── 0004_productos_codigo_barras_null_distinct.sql
```

## Diseño de la API

| Método | Ruta | Guard |
|---|---|---|
| GET | `/api/categorias` | `AuthUsuario` |
| POST/PUT/DELETE | `/api/categorias`, `/api/categorias/{id}` | `RequiereGestion` |
| GET | `/api/productos` | `RequiereGestion` (incluye inactivos) |
| GET | `/api/productos/{id}` | `AuthUsuario` |
| POST/PUT/DELETE | `/api/productos`, `/api/productos/{id}` | `RequiereGestion` |
| POST | `/api/productos/{id}/desactivar` | `RequiereGestion` |

> `GET /api/productos/buscar` es de 004. `PUT /api/productos/{id}/ubicaciones` es de 005.

## Decisiones de implementación

- **Dinero**: `rust_decimal::Decimal` con `serde-float` en el modelo y feature `rust_decimal`
  de SQLx; se serializa como número JSON, no como cadena.
- **SKU**: `normalizar_sku` (trim + mayúsculas) y `validar_sku` con la misma regla del contrato,
  implementada a mano para devolver mensajes accionables.
- **Código de barras**: `trim`; vacío o blanco → `NULL`; 8–64 alfanuméricos.
- **Unicidad**: `23505` mapeado por nombre de constraint (`productos_sku_key`,
  `productos_codigo_barras_key`, `categorias_nombre_key`), sin pre-chequeo TOCTOU.
- **Borrado de producto**: se bloquea con pre-check de `producto_ubicacion_permitida` bajo
  `FOR UPDATE`; el mapeo de `23503` queda como red de seguridad.
- **Borrado de categoría**: bloqueado si existen productos (`CategoriaConProductos`).
- **RBAC**: escritura y `GET /productos` con `RequiereGestion`; `GET /productos/{id}` y
  `GET /categorias` con `AuthUsuario` (caja necesita leer catálogo).
- **`parsear_activo`**: se mueve a `handlers/mod.rs` como `pub(crate)` y lo reutilizan usuarios,
  productos y categorías.

## Estrategia de pruebas

| Capa | Enfoque |
|---|---|
| Dominio | SKU (normalización + regex), unicidad de SKU y código, campos numéricos |
| Handlers | CRUD completo, 201/200/204, 409 de unicidad y referencias, 422, 403, 401, 404 |
| Invariantes | Un rechazo no persiste la fila; varios productos con `codigo_barras = NULL` conviven |
| Frontend | Zod de producto/categoría, cliente API con `ApiError` |
| SQLx | `cargo sqlx prepare --check` sin diferencias |

## Fases

1. **T001–T002**: tests RED de SKU y unicidad.
2. **T003**: servicios + migración 0004.
3. **T004**: handlers, `parsear_activo`, routers, `[[test]]` e integración HTTP.
4. **T005**: frontend (schemas, clientes, rutas y formularios).
5. Gates completos y actualización de `tasks.md`/`progress.md`.

## Complexity Tracking

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| _(ninguna)_ | — | Constitution Check PASS sin deudas. |
