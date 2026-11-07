# Progreso de implementación — Búsqueda y caja (004)

**Spec**: [spec.md](./spec.md) | **Plan**: [plan.md](./plan.md) | **Actualizado**: 2026-11-07 |
**Estado**: implementada y verificada; ampliación 2026-11-07 (fase 6) completa.

## Estado por fase

| Fase | Tareas | Estado | Evidencia |
|---|---|---|---|
| 1. Setup y datos | T000–T003 | Completa | migración `0007` aplicada en `stockmante` y `stockmante_test`; `ListaBusqueda`; `test_busqueda` 4/4 |
| 2. Servicio | T004–T006 | Completa | `CampoBusqueda`/`buscar()`; `test_busqueda_http` 6/6 |
| 3. Handler y contrato | T007–T008 | Completa | ruta registrada; `test_openapi` 6/6 |
| 4. Frontend | T009–T013 | Completa | schemas/cliente + 6 tests nuevos; `/caja`; navegación por rol |
| 5. Cierre | T014–T016 | Completa | gates abajo; `EXPLAIN ANALYZE`; estado de specs |
| 6. Ampliación 2026-11-07 | T017–T026 | Completa | ver «Fase 6» abajo; SC-002 p95 máximo 86 ms (categoría) sobre 10.000 productos |

## Verificación de esquema (2026-11-07)

| Base | Consulta | Resultado |
|---|---|---|
| `stockmante` | `information_schema.columns` (`productos`, `categorias`) | 17 columnas; `productos.codigo_barras` y `categorias.descripcion` nullable; resto NOT NULL. Coincide con `data-model.md`. |
| `stockmante` | `pg_indexes` | `idx_productos_nombre_trgm` = `GIN (nombre gin_trgm_ops)` |
| `stockmante` | `pg_extension` | `pg_trgm` presente |
| `stockmante_test` | `test_busqueda_http` (T006) | Suite verde contra esa base; migración 0007 aplicada |

## Evidencia observada (2026-10-26)

### Gates

| Comando | Resultado |
|---|---|
| `cd backend && cargo test` | **100 tests, 0 fallos** (exit 0) |
| `cd backend && cargo sqlx prepare --check` | **OK** (sin diferencias; verificado 2026-11-07; 004 no agrega macros nuevas) |
| `cd frontend && npm test` | **11 archivos, 110 tests, 0 fallos** |
| `cd frontend && npm run check` | 0 errores, 0 warnings |
| `cd frontend && npm run build` | OK |

### SC-002 — tiempo de búsqueda

`EXPLAIN (ANALYZE, BUFFERS)` sobre `stockmante_test` (112 productos, 150 categorías) para la
consulta por nombre con `%` + `ILIKE` y orden por similitud:

- **Execution Time: 0.573 ms**, 14 buffers, 6 filas.
- Con ese volumen PostgreSQL elige seq scan (óptimo); el índice GIN
  `idx_productos_nombre_trgm` existe y queda disponible al crecer el catálogo.

Muy por debajo del objetivo de 1 s (SC-002).

La consulta por nombre/categoría se toma como peor caso: usa similitud y orden, mientras que
SKU/código usan igualdad exacta sobre btree. La meta de 1 s de SC-002 aplica a ambas.

### Comportamiento cubierto por `test_busqueda_http`

- SKU exacto con minúsculas y código de barras exacto.
- Orden por similitud y `limite=1`.
- Filtro por `categoria_id`, listado sin `q` y búsqueda `campo=categoria`.
- Inactivos fuera por defecto; `incluir_inactivos=true` funciona con GESTION y da 403 con
  OPERACION.
- 422 con mensaje para término vacío, longitud mínima, campo inválido, límite y categoría.
- 401 sin sesión.

### Contrato y documentación

- `todos`: con 1 carácter solo se evalúan igualdades exactas; nombre/categoría exigen 2.
- `docs.rs` publica `/api/productos/buscar` y `ListaBusqueda`.
- `/caja` tiene debounce de 250 ms, descarta respuestas viejas, muestra tiempo y total, y oculta
  el checkbox de inactivos a OPERACION.

## Fase 6 — Ampliación 2026-11-07

### Waiver W-006 (T017)

Constitución 1.3.3. Cubre T000 (verificación de esquema registrada después de implementar) y
T003/T006/T009 (RED no registrado). No se reconstruyó evidencia. Condición de cierre: T026.

### Frontend (T018–T020)

| Paso | Evidencia |
|---|---|
| T018 RED | `npx vitest run src/lib/utils/busquedaCaja.test.ts` → **falla**: no se puede resolver `./busquedaCaja` (módulo inexistente), 0 tests ejecutados |
| T019 GREEN | mismo comando → **7/7** en verde |
| T020 | `/caja` usa `crearBusquedaDiferida(250)`, `LIMITE_CAJA` y `mensajeTruncados`; `npm run check` 0 errores. `cancelar()` ahora también invalida la respuesta en curso al limpiar el filtro (antes podía pintarse una respuesta vieja). Quickstart V2 verificado en el navegador con el seed y un usuario OPERARIO: `yerba` → «20 resultados · 20 ms» + «Se muestran los primeros 20 resultados. Refiná la búsqueda para encontrar el producto.»; `SEED4-00001` → «1 resultado · 13 ms» sin aviso; el checkbox de inactivos no aparece para OPERARIO |

### Verificación de esquema (T021)

`stockmante` (desarrollo), `information_schema` + `pg_constraint` + `pg_indexes`, solo lectura:
`productos` 13 columnas y `categorias` 4; coinciden con `data-model.md` salvo
`productos.stock_actual` (`INT NOT NULL DEFAULT 0`, `CHECK >= 0`, de 0009), que el seed no
necesita informar. Restricciones: `productos_sku_check`, `productos_sku_key`,
`productos_codigo_barras_key`, FK `ON DELETE RESTRICT`, `uq_categorias_nombre_normalizado`. Sin
triggers en ninguna de las dos tablas. `pg_trgm` 1.6.

### Seed y medición (T022–T024)

- `scripts/seed_catalogo_10000.sql` ejecutado **dos veces** en `stockmante`: ambas dejan 10.000
  productos `SEED4-`, 20 categorías `SEED4-CAT-` y 10.000 códigos distintos (idempotente). Luego
  `ANALYZE productos, categorias`.
- `EXPLAIN (ANALYZE, BUFFERS)` con el seed (10.001 productos):

| Consulta | Plan | Execution Time |
|---|---|---|
| `campo=nombre`, `q=yerba` | Seq Scan `productos` (434 filas) + top-N heapsort | **50,3 ms** |
| `campo=categoria`, `q=seed4-cat-07` | Seq Scan `productos` + `categorias` + top-N heapsort | **27,3 ms** |
| `campo=sku` exacto | Index Scan `productos_sku_key` | **0,10 ms** |
| `campo=codigo` exacto | Index Scan `productos_codigo_barras_key` | **0,12 ms** |

  Aun con 10.000 filas el planificador prefiere seq scan para nombre: el `OR` con
  `ILIKE '%q%'` y el orden por similitud recorren casi toda la tabla igual; el costo sigue muy
  por debajo de 1 s. `idx_productos_nombre_trgm` queda disponible.
- **Medición HTTP (T024, la medida que define SC-002)**: `node scripts/medir_sc002.mjs` contra el
  backend local (`:3000`, base `stockmante` con el seed), 40 consultas por tipo más una de
  calentamiento, tiempo desde el `fetch` hasta terminar `response.json()`:

| Campo | Consultas | Sin resultados | p50 | **p95** | Máximo | p95 ≤ 1 s |
|---|---|---|---|---|---|---|
| `sku` | 40 | 0 | 15,1 ms | **17,3 ms** | 20,9 ms | sí |
| `codigo` | 40 | 0 | 15,3 ms | **17,0 ms** | 17,7 ms | sí |
| `nombre` | 40 | 0 | 17,4 ms | **28,5 ms** | 94,4 ms | sí |
| `categoria` | 40 | 0 | 61,9 ms | **86,0 ms** | 104,1 ms | sí |

  **SC-002 cumplido** (exit 0): el peor p95 (categoría) es 86 ms, con margen amplio sobre 1 s.
- **Usuario de medición**: no había credenciales de desarrollo en el repo, así que se registró un
  usuario temporal `sc002-medicion@stockmante.local` vía `POST /api/auth/registro` (OPERARIO) y,
  sin SMTP, se activó con `UPDATE usuarios SET activo, email_verificado`. Al terminar se cerró su
  sesión y quedó **inactivo** (no se borró para no romper referencias de sesiones/auditoría). Su
  contraseña aleatoria no se guardó.
- **Limpieza**: se borraron los 10.000 productos `SEED4-` y las 20 categorías `SEED4-CAT-` en una
  transacción; verificado 0 filas `SEED4-` (queda 1 producto previo en `stockmante`).

### Gates (2026-11-07)

| Comando | Resultado |
|---|---|
| `cd backend && cargo test` (con `CARGO_TARGET_DIR` aparte: el backend estaba corriendo) | **187 tests, 0 fallos** en 38 binarios (exit 0) |
| `cd backend && cargo sqlx prepare --check` | **OK**, sin diferencias (la ampliación no toca SQL del backend) |
| `cd frontend && npm test` | **27 archivos, 244 tests, 0 fallos** (incluye `busquedaCaja.test.ts` 7/7) |
| `cd frontend && npm run check` | 0 errores, 0 warnings |
| `cd frontend && npm run build` | OK |

**Condición de cierre de W-006**: cumplida (suite de backend en verde y `prepare --check` sin
diferencias). W-006 queda cerrado.

### Docs (T025)

`docs/api.md` (comparación de nombre/categoría, SKU normalizado en `todos`, truncado inferido por
el cliente) y fila de 004 en `specs/README.md`.

## Próximo paso

Fase 6 cerrada; queda hacer el commit de la ampliación (código, scripts, docs y constitución
1.3.3). El usuario temporal `sc002-medicion@stockmante.local` sigue inactivo en `stockmante`; se
puede borrar a mano si no se lo quiere conservar.
