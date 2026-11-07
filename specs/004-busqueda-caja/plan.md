# Implementation Plan: Búsqueda y filtrado rápido para caja (004)

**Spec**: [spec.md](./spec.md) | **Caso de uso**: US4 | **Creado**: 2026-10-26 |
**Actualizado**: 2026-11-07 (ampliación por `/speckit analyze` + `/speckit clarify`)

## Summary

Agregar `GET /api/productos/buscar` para que caja y almacén encuentren productos por SKU,
código de barras, nombre o categoría sin pasar por la pantalla de administración. La búsqueda
excluye inactivos por defecto (FR-013), ordena de forma predecible y se apoya en `pg_trgm` para
nombre/categoría. En el frontend se agrega `/caja` con búsqueda inmediata (debounce), filtro por
categoría y acceso para ambas familias de rol.

**Ampliación 2026-11-07** (la base ya está implementada; esto es lo que falta):

1. **Aviso de resultados truncados** en `/caja` cuando `items.length == limite` (clarificación 3).
2. **Extraer la lógica de debounce y descarte de respuestas viejas** de `/caja/+page.svelte` a
   `#lib/utils/busquedaCaja.ts` con tests (hallazgo D3: Principio IV, componente con lógica sin
   test).
3. **Medir SC-002 como lo define la clarificación**: tiempo de respuesta de la API vista desde el
   cliente (request → JSON parseado, sin debounce), p95 por tipo de búsqueda, sobre un catálogo de
   referencia de **10.000 productos**.
4. **Waiver W-006** para la verificación de esquema retroactiva (T000) y el RED no registrado de
   T003/T006/T009 (hallazgos D1 y D2). Se tramita con `/speckit constitution`; no se reconstruye
   evidencia artificial.
5. Ajustes de contrato: operador de `campo=categoria` y normalización del SKU en `todos`.

## Technical Context

- Backend: Rust + Axum 0.8 + SQLx 0.8 + PostgreSQL 15+ (local 18).
- Frontend: SvelteKit 5 (SPA) + Tailwind + Zod; acceso HTTP solo por `#lib/api/*.ts`.
- Migración: `0007_busqueda_trgm.sql` (extensión + índice GIN). La numeración del ledger es
  global; la reserva vieja de `0005` quedó inválida cuando 001 la usó. **La ampliación no agrega
  migraciones ni cambia SQL del backend.**
- Pruebas: TDD en validación pura, integración HTTP contra el router real y tests de
  schemas/cliente en el frontend. La ampliación agrega tests de vitest con timers falsos
  (`vi.useFakeTimers`) para el debounce: el reloj queda inyectado vía mock, no oculto.
- Medición SC-002: script Node sin dependencias (`fetch` y `performance` nativos de Node 18+),
  contra el backend real apuntando a una base con el seed de 10.000 productos.

## Constitution Check

| Principio | Cumplimiento |
|---|---|
| I. Simplicidad y anti-sobreingeniería | spec 004 + este plan + tasks; SQL estático sin QueryBuilder; sin dependencias nuevas. La ampliación agrega un solo módulo utilitario y un script de medición sin dependencias. |
| II. Mantenibilidad | `BusquedaProductos`/`CampoBusqueda` en el servicio; `fetch` solo en `#lib/api` (el script de medición es una herramienta de desarrollo, fuera de `frontend/src`); la lógica de tiempo sale del componente de 280 líneas. |
| III. Claridad en mensajes | 422/403 con mensajes accionables del contrato; el aviso de truncados dice qué pasó y qué hacer («Refiná la búsqueda…»). |
| IV. Testabilidad primero | **W-006** (aprobado) para T003/T006/T009 (RED no registrado). Las tareas nuevas de la ampliación DEBEN observar RED. El debounce pasa a ser lógica testeable con reloj mockeado. |
| V. Integridad de datos | **W-006** (aprobado) para T000 (verificación de esquema registrada el 2026-11-07, después de implementar). 004 es solo lectura, no mueve stock. La ampliación no escribe SQL de producción; el seed se verifica contra el esquema real antes de ejecutarse. |

**Gate**: los dos incumplimientos históricos (P-IV y P-V) quedan cubiertos por **W-006**
(constitución 1.3.3, aprobado 2026-11-07). Su condición de cierre se verifica en T026.

### Re-evaluación post-diseño

El diseño de la ampliación no introduce violaciones nuevas: no hay capas, dependencias ni SQL
de producción nuevos, y cada pieza nueva tiene un test o una evidencia medible.

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
scripts/
  seed_catalogo_10000.sql          # NUEVO: catálogo de referencia de SC-002 (prefijo SEED4-)
  medir_sc002.mjs                  # NUEVO: p95 de la API vista desde el cliente
frontend/
  src/lib/schemas/productos.ts     # ListaBusquedaSchema
  src/lib/api/productos.ts         # buscarProductos()
  src/lib/utils/busquedaCaja.ts    # NUEVO: búsqueda diferida + aviso de truncados
  src/lib/utils/busquedaCaja.test.ts  # NUEVO
  src/routes/caja/+page.svelte     # pantalla de caja (usa busquedaCaja.ts)
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

Comparación de texto (clarificación 4): `nombre` y `categoria` usan el mismo criterio,
`% q` (trigramas) **o** `ILIKE '%q%'` escapado, sobre `productos.nombre` y `categorias.nombre`
respectivamente. SKU se compara normalizado (`normalizar_sku`: trim + mayúsculas, regla de 003)
tanto en `campo=sku` como en `todos`.

Reglas de longitud: `sku`/`codigo` aceptan desde 1 carácter (son igualdad exacta); `nombre` y
`categoria` exigen ≥ 2. Con `campo=todos` y 1 carácter solo se devuelven coincidencias exactas de
SKU/código (sin 422). Con `q` vacío y `categoria_id` presente se listan los productos de esa
categoría (escenario 4: filtrar sin escribir).

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
  quien decide si el rol puede pedirlo, para no filtrar el permiso dentro del SQL. Alcance de
  FR-013 en 004: solo búsqueda (clarificación 5); movimientos/ventas/reportes quedan en
  007, 013, 003 y 014.
- **Categoría**: el filtro por `categoria_id` se combina con el término; no fuerza que la
  categoría esté activa (los productos activos de una categoría dada de baja siguen siendo
  vendibles hasta que se los dé de baja).
- **Sin paginación**: el contrato devuelve `items`; el límite acota el costo. El frontend muestra
  la cantidad de resultados devueltos (`items.length`) y el tiempo de la consulta.

### Ampliación 2026-11-07

- **`#lib/utils/busquedaCaja.ts`** expone dos piezas puras y chicas:
  - `crearBusquedaDiferida(esperaMs)` → `{ programar(tarea), cancelar() }`. `programar` cancela
    el temporizador pendiente, incrementa un turno y, al vencer la espera, ejecuta
    `tarea(esVigente)`; `esVigente()` devuelve `false` si después se programó otra búsqueda. Es
    exactamente la lógica de `secuencia`/`temporizador` que hoy vive en el componente, sin
    cambiar el comportamiento (250 ms, descarte de respuestas viejas).
  - `mensajeTruncados(cantidad, limite)` → `string | null`: devuelve
    «Se muestran los primeros N resultados. Refiná la búsqueda para encontrar el producto.» solo
    cuando `cantidad === limite`.
  - `LIMITE_CAJA = 20` reemplaza el literal del componente para que el aviso y el request usen el
    mismo valor.
- **`/caja/+page.svelte`** usa esas piezas: el `$effect` llama `programar`/`cancelar` y la vista
  muestra el aviso debajo del contador. No se crean componentes nuevos (YAGNI).
- **Seed de referencia** `scripts/seed_catalogo_10000.sql`: mismo patrón que
  `seed_catalogo_5000.sql` de 003 (transacción, idempotente por prefijo `SEED4-`, sin movimientos
  ni auditoría), pero con lo que SC-002 necesita medir: 10.000 productos con nombres variados
  (para que la similitud trabaje sobre texto realista), `codigo_barras` único de 13 dígitos y 20
  categorías `SEED4-CAT-NN`. Se ejecuta en una base de desarrollo, nunca en `stockmante_test`
  (los tests asumen un catálogo chico) y se limpia al terminar con el bloque `DELETE` del propio
  script.
- **Medición** `scripts/medir_sc002.mjs`: hace login con credenciales tomadas de variables de
  entorno (`STOCKMANTE_EMAIL`, `STOCKMANTE_PASSWORD`; nunca escritas en el script), luego 40
  búsquedas por tipo (`sku`, `codigo`, `nombre`, `categoria`) con términos del seed, y mide
  `performance.now()` desde el `fetch` hasta terminar `response.json()`: el mismo tramo que mide
  `/caja` en `ms`, sin debounce. Imprime p50/p95/máximo por tipo. **Meta**: p95 ≤ 1000 ms en los
  cuatro. Se complementa con `EXPLAIN (ANALYZE, BUFFERS)` de la consulta más lenta para registrar
  si usa el índice GIN.

## Estrategia de pruebas

- Unitarias: `CampoBusqueda::parsear` (todos los valores + inválido), longitud mínima, límite
  fuera de rango, categoría inválida, término vacío con/sin categoría.
- Integración: crear categoría y productos; buscar por SKU exacto, código exacto, fragmento de
  nombre con orden por relevancia, categoría, exclusión de inactivos, `incluir_inactivos` como
  GESTION/OPERACION, límite y 401.
- Frontend: schema de respuesta y serialización de query params; test del cliente con `fetch`
  mockeado. Verificación manual de `/caja`.
- **Ampliación** (RED obligatorio antes de implementar):
  - `busquedaCaja.test.ts` con `vi.useFakeTimers()`: la tarea no corre antes de 250 ms; dos
    `programar` seguidos ejecutan solo la última; `esVigente()` pasa a `false` cuando se programa
    otra búsqueda mientras la primera espera su respuesta; `cancelar()` evita la ejecución.
  - `mensajeTruncados`: `null` con 0, con menos que el límite y con límite inválido; mensaje con
    `cantidad === limite`.
- SC-002: `medir_sc002.mjs` sobre el seed de 10.000 productos; p95 por tipo registrado en
  `progress.md`, más `EXPLAIN (ANALYZE, BUFFERS)` de la consulta más lenta.

## Riesgos y mitigaciones

| Riesgo | Mitigación |
|---|---|
| `pg_trgm` no disponible en el entorno | extensión trusted; el gate falla temprano con mensaje claro |
| Ranking inestable con pocos trigramas | desempate por nombre e id |
| Búsqueda por 1 carácter satura | solo exactos en `todos`; nombre exige 2 |
| Fuga de inactivos a caja | filtro por defecto + test HTTP explícito |
| UI que consulta en cada tecla | debounce 250 ms + cancelación por turno, ahora con test |
| `campo=categoria` lento a 10.000 productos (no hay GIN sobre `categorias.nombre`) | `categorias` es chica (20 filas en el seed); si el p95 supera 1 s, registrar el desvío y evaluar un índice en una spec aparte |
| El seed contamina una base usada por tests | se ejecuta solo en la base de desarrollo y se limpia con el `DELETE` por prefijo |
| Binario del backend bloqueado al medir (pasó en 003) | usar `CARGO_TARGET_DIR` aparte; si igual no se puede medir HTTP, registrar el desvío como en 003 en vez de dar SC-002 por cumplido |

## Fases

1. Migración 0007 y modelo `ListaBusqueda`. *(hecho)*
2. Servicio `buscar` + validaciones (TDD). *(hecho)*
3. Handler, ruta y OpenAPI. *(hecho)*
4. Cliente/schema frontend + pantalla `/caja` + navegación. *(hecho)*
5. Gates, quickstart y cierre. *(hecho el 2026-10-26)*
6. **Ampliación**: waiver W-006, `busquedaCaja.ts` (RED → GREEN), aviso de truncados en `/caja`,
   seed de 10.000 + medición SC-002, contrato/docs y gates.

## Complexity Tracking

Sin complejidad adicional: una consulta de lectura, un índice y una pantalla. La ampliación suma
un módulo utilitario con tests y dos herramientas de desarrollo (seed y medición) fuera del código
de producción.

| Violación | Por qué | Alternativa descartada |
|---|---|---|
| P-V: T000 registrado después de implementar | la implementación de 2026-10-26 no dejó evidencia previa; la verificación del 2026-11-07 confirma que el esquema coincide | reconstruir una verificación "previa" ficticia (prohibido: los waivers no reconstruyen evidencia) |
| P-IV: RED no registrado en T003/T006/T009 | los tests existen y están verdes, pero `progress.md` no guardó el fallo previo | borrar e implementar de nuevo para provocar un RED artificial |
