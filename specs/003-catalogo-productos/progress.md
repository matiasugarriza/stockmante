# Progreso de implementación — Catálogo de productos y categorías (003)

**Spec**: [spec.md](./spec.md) | **Actualizado**: 2026-10-26 | **Estado**: plan v2 implementado,
probado por el usuario y commiteado en la rama `feat/003-catalogo-productos`; falta el push.

> Espejo local de reanudación del caso de uso US3.
>
> La implementación anterior (WIP) **no se reutilizó**: se borró del árbol el 2026-10-25 y sigue en
> la rama `wip/003-catalogo` (commit `941b0cf`) solo como referencia histórica. Este archivo ya no
> describe ese WIP, sino el plan v2 ejecutado el 2026-10-26.

## Estado por fase

| Fase | Tareas | Estado | Evidencia |
|---|---|---|---|
| 1. Setup | T001–T004 | Completa | dependencias, migración `0006`, `Cargo.toml`, lockfile |
| 2. Foundational | T005–T009 | Completa | migración aplicada en `stockmante` y `stockmante_test`; modelos y `AppError` |
| 3. US3 · Categorías | T010–T014 | Completa | 5 tests unitarios + 4 HTTP + servicio/handler/router |
| 4. US3 · Productos | T015–T021 | Completa | 5 + 7 unitarios, 4 HTTP + 1 de baja lógica, servicio/handler/router |
| 5. US3 · Frontend | T022–T030 | Completa | 4 archivos de test nuevos; pantallas y componentes |
| 6. Polish | T031–T037 | Completa | gates, V6a/V6b/RBAC/invariantes, documentación |

## Evidencia observada (2026-10-26)

### Gates de calidad

| Comando | Resultado |
|---|---|
| `cd backend && cargo test` | **82 tests, 0 fallos** (19 targets con tests) |
| `cd backend && cargo sqlx prepare --check` | exit 0; aviso preexistente `potentially unused queries found in .sqlx` (no hay macros `query!`) |
| `cd frontend && npm test` | **11 archivos, 104 tests, 0 fallos** |
| `cd frontend && npm run check` | 0 errores, 0 warnings |
| `cd frontend && npm run build` | OK |
| grep `unwrap(|expect(|panic!` en `backend/src` | solo una mención en el comentario de `models/producto.rs:5` |

### V6a — Categorías (quickstart, 13 pasos)

Ejecutado contra el backend real (`PORT=3000`, base `stockmante`). **13/13 pasos y subchecks en
verde**. Observaciones:

- Alta `201`; duplicado con mayúsculas y espacios de extremo `409 categoria_duplicada`.
- `PUT` parcial conserva `descripcion` y `activo`; `descripcion: ""` → `null`.
- `PUT` con `activo` → `422 validacion` «El estado se cambia con /desactivar o /reactivar.».
- `PUT` con `{}` → `422 validacion` «Indicá al menos un campo para actualizar.».
- `DELETE` → **405 sin cuerpo**, con `Allow: GET, HEAD, PUT` (Axum agrega `HEAD` a la ruta `GET`;
  corregido en `contracts/api.md` y `plan.md`).
- `/desactivar` dos veces → `204` y `204`; la fila sigue consultable con `activo: false`.
- `/reactivar` → `204`; vuelve al listado por defecto.
- El acento se guardó correctamente (verificado en base: `c3a9` en UTF-8).

### V6b — Productos (quickstart, 15 pasos + extras)

**15/15 pasos y 4 extras en verde**. Observaciones:

- SKU normalizado a mayúsculas y `codigo_barras: ""` → `null`.
- SKU repetido en minúsculas → `409 sku_duplicado` («Ya existe un producto con el SKU … Elegí otro.»).
- Varios productos con `codigo_barras` ausente conviven (`NULL` distintos).
- Código de barras repetido → `409 codigo_barras_duplicado`.
- SKU `bad!`, `AB`, 31 caracteres y `-ABC` → `422 validacion` con el formato esperado.
- `costo: -1`, `precio_venta: 10.999`, `stock_minimo: -3` → `422` con mensaje por campo.
- `unidad_medida: "LITRO"` → `422` enumerando `UNIDAD`, `KG`, `METRO`, `CAJA`.
- `categoria_id` inactiva o inexistente → `422` «La categoría indicada no existe o está dada de baja.».
- `q=yerba` (nombre) y `q=YERBA-1K-…` (SKU) encuentran el producto; `q` sin coincidencias → lista vacía.
- `categoria_id` + `activo=all` filtra e incluye inactivos; `activo=otro` → `422`.
- `DELETE` → **405 sin cuerpo**; el detalle sigue devolviendo el mismo `id` y `activo`.
- `/desactivar` → `204`; el default ya no lo muestra, `activo=all` sí, el detalle sigue en `200`.
- `/reactivar` → `204` con SKU, precios y categoría intactos.
- `PUT { "activo": false }` → `422` y el producto sigue activo.
- Extras: `PUT` parcial conserva el resto; `PUT` con `codigo_barras: ""` limpia a `null`; un `PUT`
  con un campo inválido no aplica ningún cambio (atomicidad).

### RBAC (quickstart)

Sesión real `OPERARIO` (familia `OPERACION`) creada por API. **22/22 checks en verde**:

| Acción | Observado |
|---|---|
| `GET /categorias`, `GET /categorias/{id}` | 200 |
| `GET /productos/{id}` | 200 |
| `GET /productos` | 403 `sin_permisos` |
| `POST` / `PUT` / `desactivar` / `reactivar` de ambas entidades | 403 `sin_permisos` |
| Cualquier ruta sin sesión | 401 `no_autenticado` |

Las escrituras rechazadas no dejaron filas ni cambios (verificado con el listado de gestión).

### Invariantes

- Conteos e ids de `categorias` y `productos` **idénticos antes y después** de un ciclo
  desactivar→reactivar; solo cambió el flag `activo`.
- Los datos de las filas de la corrida quedaron intactos (SKU, montos, categoría, nombre).
- `GET /api/usuarios` sin filtro sigue devolviendo activos e inactivos (regresión de
  `parsear_activo`), y `activo=true|false` filtra como corresponde.
- **Huecos de id**: los ids saltados (`categorias` 1, 2, 5, 8; `productos` 2, 6) provienen de
  altas rechazadas que consumieron la secuencia, no de borrados. La secuencia de `productos` quedó
  en `last_value=6` con `max(id)=5`; ninguna fila desapareció.

## Desvíos y correcciones registrados

### TDD (desvío honesto)

- T006 implementó las funciones puras de normalización/validación antes de escribir los tests
  unitarios de T010/T015/T016. Por eso esos tests quedaron **verdes en su primera corrida**: no hubo
  RED observable en ellos.
- El **RED sí se observó** en los tests HTTP de T011/T017/T018: fallaron por router ausente/compilación
  antes de implementar servicios y handlers, y luego pasaron a verde.
- No se inventó evidencia RED para los tests unitarios; queda registrado como desvío de secuencia.

### Fragilidades de test corregidas

- `test_productos_http.rs` armaba a mano un SKU de 36 caracteres (el contrato permite 30) → se
  reemplazó por el helper `sku_unico`.
- `test_usuarios_http.rs` contaba `COUNT(*)` global de `usuarios`; la suite de ese binario corre en
  paralelo y el conteo era flaky → ahora cuenta una marca única por corrida.
- Los listados de `test_usuarios_http.rs` asumían que la población cabía en una página de 100 y
  fallaban con la base de test acumulada → ahora recorren todas las páginas, y el test nuevo cubre
  el default de `activo` sin filtro (regresión de 002).

### Otros ajustes

- `AppError::SkuDuplicado` pasó a `SkuDuplicado(String)` para devolver el SKU en el mensaje.
- Se eliminó `AppError::ProductoConReferencias` (era del borrado físico) y se agregó
  `CategoriaDuplicada`.
- `parsear_activo(valor, por_defecto)` quedó centralizado en `handlers/mod.rs` (usuarios `None`,
  catálogo `Some(true)`).
- `categoria_id` igual a la actual se permite aunque la categoría esté inactiva; asignar otra
  inactiva es `422`.

## Qué no se reutilizó del WIP deshecho

- `DELETE` físico y bloqueo por referencias (`ProductoConReferencias`, `CategoriaConProductos`).
- `activo` dentro del `PUT`.
- La migración `0004` (número quemado; su contenido se absorbió en `0006`).
- El pre-chequeo de productos para bloquear la baja de una categoría: la baja es lógica y no toca
  productos.

## Entorno local dejado para la prueba manual

- Backend detenido al terminar la corrida. Para levantarlo:
  `cd backend`; `$env:DATABASE_URL="postgres://stockmante:stockmante@localhost:5432/stockmante"`;
  `cargo run --bin stockmante-backend`.
- Frontend: `cd frontend && npm run dev` (usa `PUBLIC_API_URL` o el default `http://localhost:3000`).
- Usuarios locales (solo DEV, no forman parte del repositorio):
  - `admin@stockmante.local` / `Prueba1234` (ADMIN, GESTION) — la temporal se regeneró con
    `cargo run --bin bootstrap -- --reset-admin-password` y se cambió por API.
  - `operario@stockmante.local` / `Prueba1234` (OPERARIO, OPERACION) — creado por API para RBAC.
- Datos de ejemplo de la corrida (base `stockmante`): categorías 3, 4, 6, 7, 9 y 10 (la 10 queda
  inactiva); productos 1, 3, 4 y 5, todos activos.

## Commits (rama `feat/003-catalogo-productos`, desde `main` en `f3a9854`)

| Unidad | Commit | Contenido |
|---|---|---|
| 1 | `8fa1e79` | Migración `0006`, modelos y validaciones puras |
| 2 | `49a59d8` | CRUD de categorías con baja lógica |
| 3 | `b152b51` | CRUD de productos con filtros y paginación |
| 4 | `bc9335a` | Schemas Zod y clientes API del frontend |
| 5 | `c02c99e` | Pantallas y componentes de administración |
| 6 | `350f4fe` | Tests de regresión y estabilidad de listados |
| 7 | `388c1f4` | Spec, plan y contratos |
| 8 | este commit | Tareas, progreso y README de specs |

RDD está activo (global). El commit 1 se evaluó como `high` y su revisión nativa terminó
**aprobada** (2 lentes: riesgo y confiabilidad; 7 hallazgos no bloqueantes, 1 refutado). Los commits
2 y 3 se evaluaron juntos como `medium`, `under_budget`; la frontera revisada sigue en `8fa1e79`.

## Pendiente

1. `git push -u origin feat/003-catalogo-productos` (autorizado por el usuario).
2. Integrar la rama según la política del repositorio.
3. Los hallazgos advisory de RDD quedan como trabajo posterior: `app_de_test` devuelve un
   `Arc<CorreoDev>` distinto del que usa el router; `cuerpo_json` convierte JSON inválido en
   `Value::Null`; `test_apperror` no verifica el mensaje de `SkuDuplicado`.

## Gotchas

- `psql` no está en PATH: usar `C:\Program Files\PostgreSQL\18\bin\psql.exe`.
- PowerShell 5.1: los cuerpos JSON para `curl.exe` deben escribirse en UTF-8 **sin BOM**
  (`[IO.File]::WriteAllText` con `UTF8Encoding($false)`); `Set-Content -Encoding UTF8` agrega BOM y
  el backend responde `422` «El cuerpo de la solicitud no es un JSON válido».
- En arrays de PowerShell, `"@" + $ruta` se parte en dos elementos; hay que envolverlo:
  `("@" + $ruta)`.
- `cargo` devuelve exit 1 en PowerShell cuando escribe advertencias por stderr aunque compile; la
  fuente de verdad es `Finished` / `test result`.
- Advertencias preexistentes ajenas a 003: nombre no `snake_case` en `test_cambio_password.rs:106` y
  variable `id` sin usar en `test_usuarios_http.rs`.
- `stockmante_test` conserva usuarios y categorías residuales: usar marcas únicas por corrida y
  recorrer todas las páginas de los listados, nunca conteos globales ni la primera página.
- Si queda una versión aplicada en `_sqlx_migrations` sin archivo, `sqlx migrate run` falla; la fila
  huérfana `4` se borró de `stockmante` y `stockmante_test`.
- SvelteKit 3 usa `#lib`, no `$lib`.
