# Progreso de implementación — Búsqueda y caja (004)

**Spec**: [spec.md](./spec.md) | **Plan**: [plan.md](./plan.md) | **Actualizado**: 2026-10-26 |
**Estado**: implementada y verificada; incluida en el commit único de cierre del Módulo 1 (2026-10-26).

## Estado por fase

| Fase | Tareas | Estado | Evidencia |
|---|---|---|---|
| 1. Setup y datos | T001–T003 | Completa | migración `0007` aplicada en `stockmante` y `stockmante_test`; `ListaBusqueda`; `test_busqueda` 4/4 |
| 2. Servicio | T004–T006 | Completa | `CampoBusqueda`/`buscar()`; `test_busqueda_http` 6/6 |
| 3. Handler y contrato | T007–T008 | Completa | ruta registrada; `test_openapi` 6/6 |
| 4. Frontend | T009–T013 | Completa | schemas/cliente + 6 tests nuevos; `/caja`; navegación por rol |
| 5. Cierre | T014–T016 | Completa | gates abajo; `EXPLAIN ANALYZE`; estado de specs |

## Evidencia observada (2026-10-26)

### Gates

| Comando | Resultado |
|---|---|
| `cd backend && cargo test` | **100 tests, 0 fallos** (exit 0) |
| `cd backend && cargo sqlx prepare --check` | pendiente de repetir tras 004 (sin macros nuevas) |
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

### Comportamiento cubierto por `test_busqueda_http`

- SKU exacto con minúsculas y código de barras exacto.
- Orden por similitud y `limite=1`.
- Filtro por `categoria_id`, listado sin `q` y búsqueda `campo=categoria`.
- Inactivos fuera por defecto; `incluir_inactivos=true` funciona con GESTION y da 403 con
  OPERACION.
- 422 con mensaje para término vacío, longitud mínima, campo inválido, límite y categoría.
- 401 sin sesión.

### Contrato y documentación

- `score`/`todos`: con 1 carácter solo se evalúan igualdades exactas; nombre/categoría exigen 2.
- `docs.rs` publica `/api/productos/buscar` y `ListaBusqueda`.
- `/caja` tiene debounce de 250 ms, descarta respuestas viejas, muestra tiempo y total, y oculta
  el checkbox de inactivos a OPERACION.

## Próximo paso

Spec 005 (depósitos y ubicaciones). El commit queda para el cierre del Módulo 1.
