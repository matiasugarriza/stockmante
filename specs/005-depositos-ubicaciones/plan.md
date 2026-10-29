# Implementation Plan: Depósitos y ubicaciones jerárquicas (005)

**Spec**: [spec.md](./spec.md) | **Caso de uso**: US5 | **Creado**: 2026-10-26

## Summary

Exponer el CRUD de depósitos y ubicaciones jerárquicas que ya existen en la base desde la
migración `0003`, con las reglas de integridad en el servicio: el padre debe pertenecer al mismo
depósito y estar activo, no se permiten ciclos, el borrado físico se bloquea con dependencias y la
asignación producto–ubicación reemplaza el conjunto permitido. En el frontend se agrega
`/admin/depositos` con el árbol, el formulario de ubicaciones y la asignación por producto.

## Technical Context

- Backend: Rust + Axum 0.8 + SQLx 0.8 + PostgreSQL 15+ (local 18).
- Frontend: SvelteKit 5 (SPA) + Tailwind + Zod; acceso HTTP solo por `#lib/api/*.ts`.
- Migraciones: **ninguna nueva**. `depositos`, `ubicaciones` y `producto_ubicacion_permitida`
  existen desde `0003_depositos_ubicaciones.sql` (fase foundational de 001). La propiedad del
  esquema queda en esta spec, como declara la spec.
- Pruebas: validadores puros (unitarios), integración HTTP contra el router real, schemas y
  cliente en el frontend.

## Constitution Check

| Principio | Cumplimiento |
|---|---|
| I. Especificación primero | spec 005 + este plan + tasks antes del código |
| II. Tipado y centralización | modelos en `models/deposito.rs`; un solo `AppEstado`; `fetch` solo en `#lib/api` |
| III. Mensajes accionables en español | 409/422 con mensajes del contrato |
| IV. Seguridad y RBAC | todas las rutas con `RequiereGestion`; el frontend oculta, el backend decide |
| V. Pruebas | TDD de validadores y casos HTTP; sin `unwrap`/`panic!`/`expect` en producción |

## Project Structure

```text
backend/
  src/models/deposito.rs           # modelos, normalización y validadores
  src/services/depositos.rs        # CRUD, jerarquía, ciclos, asignaciones
  src/handlers/depositos.rs        # /api/depositos
  src/handlers/ubicaciones.rs      # /api/ubicaciones
  src/handlers/productos.rs        # /api/productos/{id}/ubicaciones
  src/error.rs                     # 409/422 de depósitos y ubicaciones
  src/docs.rs                      # paths, schemas y tags OpenAPI
  tests/unit/test_deposito_validacion.rs
  tests/integration/test_depositos_http.rs
frontend/
  src/lib/schemas/depositos.ts
  src/lib/api/depositos.ts
  src/routes/admin/depositos/+page.svelte
  src/lib/schemas/depositos.test.ts
  src/lib/api/depositos.test.ts
```

## Diseño de la API

Todas las rutas exigen `RequiereGestion` (401 sin sesión, 403 fuera de GESTION).

| Método | Ruta | Éxito | Errores propios |
|---|---|---|---|
| GET | `/api/depositos?activo=` | 200 `[Deposito]` | 422 filtro inválido |
| POST | `/api/depositos` | 201 `Deposito` | 409 nombre/identificador, 422 |
| GET | `/api/depositos/{id}` | 200 `Deposito` | 404 |
| PUT | `/api/depositos/{id}` | 200 `Deposito` | 404, 409, 422 |
| DELETE | `/api/depositos/{id}` | 204 | 404, 409 `deposito_con_dependencias` |
| POST | `/api/depositos/{id}/desactivar` | 204 | 404 |
| POST | `/api/depositos/{id}/reactivar` | 204 | 404 |
| GET | `/api/depositos/{id}/ubicaciones` | 200 `{ items }` | 404 |
| POST | `/api/ubicaciones` | 201 `Ubicacion` | 409 duplicada, 422 huérfana/tipo |
| GET | `/api/ubicaciones/{id}` | 200 `Ubicacion` | 404 |
| PUT | `/api/ubicaciones/{id}` | 200 `Ubicacion` | 404, 409, 422 ciclo |
| DELETE | `/api/ubicaciones/{id}` | 204 | 404, 409 `ubicacion_con_dependencias` |
| POST | `/api/ubicaciones/{id}/desactivar` | 204 | 404 |
| POST | `/api/ubicaciones/{id}/reactivar` | 204 | 404 |
| GET | `/api/productos/{id}/ubicaciones` | 200 `{ items }` | 404 |
| PUT | `/api/productos/{id}/ubicaciones` | 204 | 404 ubicación inactiva/inexistente, 422 ids |

`PUT` es parcial: un campo ausente conserva su valor. `direccion: ""` limpia la dirección del
depósito y `padre_id: null` mueve la ubicación a la raíz. `activo` no viaja por `PUT`: su sola
presencia es 422 y el estado se cambia con `/desactivar` o `/reactivar`.

## Esquema (sin migración nueva)

`0003` ya define:

- `depositos(nombre UNIQUE, identificador UNIQUE, direccion NULL, activo)`.
- `ubicaciones(deposito_id NOT NULL → depositos ON DELETE RESTRICT, padre_id NULL →
  ubicaciones ON DELETE RESTRICT, nombre, tipo, activo)` con
  `UNIQUE NULLS NOT DISTINCT (deposito_id, padre_id, nombre)`.
- `producto_ubicacion_permitida(producto_id, ubicacion_id)` con PK compuesta.
- Índices `idx_ubicaciones_deposito` y `idx_ubicaciones_padre`.

## Decisiones de implementación

- **Integridad en el servicio, dentro de la transacción**: el orden de las validaciones (depósito
  activo → padre del mismo depósito → ciclo) evita escrituras parciales y respuestas ambiguas.
- **Ciclos con CTE recursiva**: al mover una ubicación se camina la ascendencia del nuevo padre y
  se rechaza si aparece la propia ubicación. Un `CHECK` no puede expresar esto en PostgreSQL.
- **Borrado físico restringido**: se cuenta hijos/asignaciones antes del `DELETE`; el `23503` se
  mapea igual como 409, por si otro proceso gana la carrera. La alternativa documentada es la baja
  lógica.
- **Cambio de depósito**: si la ubicación tiene hijos o asignaciones es 409. Si no, y no se indica
  `padre_id`, pasa a ser raíz para no violar la regla de mismo depósito.
- **Asignación como reemplazo**: `PUT` borra e inserta el conjunto dentro de una transacción;
  deduplica ids, rechaza ids ≤ 0 (422) y ubicaciones inactivas o de depósitos inactivos (404).
- **Árbol en el cliente**: la API devuelve una lista plana con `padre_id`; el frontend la aplana
  en profundidad con un guard para no entrar en loops si el dato llegara inconsistente.

## Estrategia de pruebas

- Unitarias: longitudes, `trim`, dirección vacía → `null`, enum de tipo, mapeo de errores.
- Integración: CRUD de depósito, unicidad, borrado con dependencias, jerarquía (raíz/hija,
  duplicados, padre de otro depósito, ciclo), cambio de depósito con hijos, asignación
  (reemplazo, deduplicación, vacío, inactiva, producto inexistente) y RBAC/401.
- Frontend: schemas (nullable, enum, trim) y cliente (query, métodos, body, errores).
- SC-005: la migración `0003` declara `deposito_id NOT NULL`; se verifica con `\d ubicaciones` y
  los tests HTTP de huérfana.

## Riesgos y mitigaciones

| Riesgo | Mitigación |
|---|---|
| Carrera entre validación y `INSERT` | constraints de la BD + mapeo de `23505`/`23503` |
| Árbol profundo o inconsistente | CTE recursiva acotada por el dato; guard en el cliente |
| Borrar un depósito en uso | conteo de dependencias + 409 + alternativa de baja lógica |
| UI de árbol compleja | lista plana aplanada por profundidad, sin componente recursivo |

## Fases

1. Modelos y validadores puros.
2. Servicio con transacciones y reglas de jerarquía.
3. Handlers, rutas y OpenAPI.
4. Frontend: schemas, cliente, página y navegación.
5. Tests, gates y documentación.

## Complexity Tracking

Sin desviaciones de la constitución: tres tablas preexistentes, un servicio y una pantalla.
