# Implementation Plan: Búsqueda y filtrado rápido para caja (004)

**Spec**: [spec.md](./spec.md) | **Caso de uso**: US4 | **Creado**: 2026-10-26

## Summary

Agregar `GET /api/productos/buscar` para que caja y almacén encuentren productos por SKU,
código de barras, nombre o categoría sin pasar por la pantalla de administración. La búsqueda
excluye inactivos por defecto (FR-013), ordena de forma predecible y se apoya en `pg_trgm` para
nombre/categoría. En el frontend se agrega `/caja` con búsqueda inmediata (debounce), filtro por
categoría y acceso para ambas familias de rol.

## Technical Context

- Backend: Rust + Axum 0.8 + SQLx 0.8 + PostgreSQL 15+ (local 18).
- Frontend: SvelteKit 5 (SPA) + Tailwind + Zod; acceso HTTP solo por `#lib/api/*.ts`.
- Migración: `0007_busqueda_trgm.sql` (extensión + índice GIN). La numeración del ledger es
  global; la reserva vieja de `0005` quedó inválida cuando 001 la usó.
- Pruebas: TDD en validación pura, integración HTTP contra el router real y tests de
  schemas/cliente en el frontend.

## Constitution Check

| Principio | Cumplimiento |
|---|---|
| I. Especificación primero | spec 004 + este plan + tasks antes de escribir código |
| II. Tipado y centralización | `BusquedaProductos`/`CampoBusqueda` en el servicio; `fetch` solo en `#lib/api` |
| III. Mensajes accionables en español | 422/403 con mensajes del contrato |
| IV. Seguridad y RBAC | sesión válida; `incluir_inactivos=true` solo GESTION (403) |
| V. Pruebas | TDD para parsing/validación y casos HTTP; sin `unwrap`/`panic!`/`expect` en producción |

## Project Structure

```text
backend/
  src/services/productos.rs        # buscar(), CampoBusqueda, validaciones
  src/handlers/productos.rs        # GET /buscar + utoipa
  src/models/producto.rs           # ListaBusqueda
  src/docs.rs                      # path/schema en OpenAPI
  tests/unit/test_busqueda.rs      # parsing de campo, límites, errores
  tests/integration/test_busqueda_http.rs
migrations/0007_busqueda_trgm.sql
frontend/
  src/lib/schemas/productos.ts     # ListaBusquedaSchema
  src/lib/api/productos.ts         # buscarProductos()
  src/routes/caja/+page.svelte     # pantalla de caja
  src/lib/components/caja/...      # si hace falta separar la tabla
  src/lib/api/productos.test.ts
  src/lib/schemas/productos.test.ts
```

## Diseño de la API

`GET /api/productos/buscar?q=&campo=&incluir_inactivos=&categoria_id=&limite=`

| Parámetro | Tipo | Default | Reglas |
|---|---|---|---|
| `q` | string | — | `trim`; 1–160; vacío solo si hay `categoria_id` |
| `campo` | enum | `todos` | `sku`, `codigo`, `nombre`, `categoria`, `todos` |
| `incluir_inactivos` | bool | `false` | `true` exige familia GESTION |
| `categoria_id` | int | — | > 0; filtra por categoría (activa o no) |
| `limite` | int | 20 | 1–50 |

Respuesta `200 { "items": [Producto…] }`, sin paginación (la caja necesita la lista corta y
completa de coincidencias). Orden:

1. `todos`: SKU exacto, luego código exacto, luego similitud de nombre descendente, nombre ASC, id ASC.
2. `sku`/`codigo`: igualdad exacta; nombre ASC, id ASC (máximo 1 esperado por unicidad).
3. `nombre`: `similarity(nombre, q)` DESC, nombre ASC, id ASC.
4. `categoria`: `similarity(categoria.nombre, q)` DESC, nombre ASC, id ASC.

Reglas de longitud: `sku`/`codigo` aceptan desde 1 carácter (son igualdad exacta); `nombre`,
`categoria` y la parte por nombre de `todos` exigen ≥ 2. Con `campo=todos` y 1 carácter solo se
devuelven coincidencias exactas de SKU/código. Con `q` vacío y `categoria_id` presente se listan
los productos de esa categoría (escenario 4: filtrar sin escribir).

Errores: `401 no_autenticado`, `403 sin_permisos`, `422 validacion` con mensaje accionable.

## Migración y esquema (0007)

```sql
CREATE EXTENSION IF NOT EXISTS pg_trgm;
CREATE INDEX idx_productos_nombre_trgm ON productos USING gin (nombre gin_trgm_ops);
```

Sin cambios de columnas ni de datos. Reversible con `DROP INDEX`; la extensión puede quedar.

## Decisiones de implementación

- **SQL estático por campo** en lugar de `QueryBuilder`: cada rama tiene WHERE/ORDER fijos y
  binds posicionales; no se concatena entrada del usuario.
- **`%` + `ILIKE`**: `%` aporta ranking por similitud; `ILIKE '%q%'` cubre subcadenas cortas que
  caen debajo del umbral de trigramas. El patrón se escapa (`%`, `_`, `\`) como en 003.
- **Inactivos**: el filtro `p.activo` se aplica salvo `incluir_inactivos=true`; el handler es
  quien decide si el rol puede pedirlo, para no filtrar el permiso dentro del SQL.
- **Categoría**: el filtro por `categoria_id` se combina con el término; no fuerza que la
  categoría esté activa (los productos activos de una categoría dada de baja siguen siendo
  vendibles hasta que se los dé de baja).
- **Sin paginación**: el contrato devuelve `items`; el límite acota el costo. El frontend muestra
  el total encontrado y el tiempo de la consulta.

## Estrategia de pruebas

- Unitarias: `CampoBusqueda::parsear` (todos los valores + inválido), longitud mínima, límite
  fuera de rango, categoría inválida, término vacío con/sin categoría.
- Integración: crear categoría y productos; buscar por SKU exacto, código exacto, fragmento de
  nombre con orden por relevancia, categoría, exclusión de inactivos, `incluir_inactivos` como
  GESTION/OPERACION, límite y 401.
- Frontend: schema de respuesta y serialización de query params; test del cliente con `fetch`
  mockeado. Verificación manual de `/caja`.
- SC-002: medir con `EXPLAIN ANALYZE` sobre datos de prueba y registrar el tiempo observado.

## Riesgos y mitigaciones

| Riesgo | Mitigación |
|---|---|
| `pg_trgm` no disponible en el entorno | extensión trusted; el gate falla temprano con mensaje claro |
| Ranking inestable con pocos trigramas | desempate por nombre e id |
| Búsqueda por 1 carácter satura | solo exactos en `todos`; nombre exige 2 |
| Fuga de inactivos a caja | filtro por defecto + test HTTP explícito |
| UI que consulta en cada tecla | debounce 250 ms + cancelación por secuencia |

## Fases

1. Migración 0007 y modelo `ListaBusqueda`.
2. Servicio `buscar` + validaciones (TDD).
3. Handler, ruta y OpenAPI.
4. Cliente/schema frontend + pantalla `/caja` + navegación.
5. Gates, quickstart y cierre.

## Complexity Tracking

Sin complejidad adicional: una consulta de lectura, un índice y una pantalla.
