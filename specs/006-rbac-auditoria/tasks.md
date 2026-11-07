# Tasks — RBAC transversal y auditoría (006)

**Spec**: [spec.md](./spec.md) | **Plan**: [plan.md](./plan.md) | **Contratos**:
[contracts/api.md](./contracts/api.md) | **Creado**: 2026-10-26 | **Ampliado**: 2026-11-07

**Tests**: sí. Matriz HTTP transversal + verificación SQL de auditoría + guard estructural.

**Ampliación 2026-11-07**: T001–T011 son la base implementada; T001a y T012–T026 implementados
el 2026-11-07; T027 cierra con los gates registrados en `progress.md`.

## Fase 1 — Auditoría

- [X] T001 Crear `migrations/0008_auditoria.sql` con `auditoria(id, tabla_ref, registro_id,
  accion, usuario_id, cuando)` e índices; aplicar en `stockmante` y `stockmante_test`.
  **Aceptación**: migración 8 aplicada en ambas bases.
- [X] T001a Registrar en `progress.md` la verificación de esquema previa a `0008` (tipo de
  `usuarios.id`, FK y columnas) o un waiver acotado. **Aceptación**: evidencia o waiver.
- [X] T002 Crear `services/auditoria.rs` con constantes de tabla/acción y
  `registrar(&mut Transaction, ...)`.
- [X] T003 Instrumentar usuarios: alta, edición, baja, reactivación y contraseña temporal dentro
  de la transacción; el actor viaja como `actor_id`.
- [X] T004 Instrumentar productos: alta, edición (incluye precio), baja y reactivación con el
  mismo patrón transaccional.
- [X] T005 Pasar el actor desde `RequiereGestion(pub Usuario)` en los handlers de usuarios y
  productos.

## Fase 2 — Verificación

- [X] T006 Agregar `actor_de_test` en `tests/unit/common/mod.rs` y actualizar las llamadas
  directas a los servicios en `test_usuarios_email`, `test_usuarios_alta` y `test_ultimo_admin`.
- [X] T007 RED/GREEN: `tests/integration/test_rbac_matriz.rs` con 4 tests: matriz completa
  (OPERACION → 403, GESTION → no 401/403), auditoría de productos, auditoría de usuarios y 403
  que no audita. **Aceptación**: 4/4.
- [X] T008 Registrar `test_rbac_matriz` en `Cargo.toml`.

## Fase 3 — Frontend

- [X] T009 Crear `frontend/src/routes/admin/+layout.svelte`: OPERACION → `/caja`; solo GESTION
  renderiza las páginas de administración.
- [X] T009a Verificar que la UI de productos y depósitos ofrece `Desactivar` como alternativa a la
  baja física. **Evidencia**: `admin/productos` llama a `desactivarProducto`; `admin/depositos`
  llama a `desactivarDeposito`/`desactivarUbicacion`; quickstart SC-007 lo recorre.

## Fase 4 — Cierre

- [X] T010 Correr `cargo test` (115/115), `cargo sqlx prepare --check` (OK, sin diferencias;
  2026-11-07), `npm test` (136/136), `npm run check` (0/0).
- [X] T011 Escribir artefactos (`plan.md`, `research.md`, `data-model.md`,
  `contracts/api.md`, `quickstart.md`, `tasks.md`, `progress.md`) y actualizar la spec
  (`0006` → `0008`), `specs/README.md` y `.specify/feature.json`.

## Fase 5 — Ampliación de auditoría (FR-019, 2026-11-07)

**Goal**: auditar todas las escrituras de gestión de 001–005, además de usuarios y productos.

- [X] T012 RED: extender `tests/integration/test_rbac_matriz.rs` con auditoría de categorías
  (alta, edición, baja, reactivación) y un 403 que no audita. **Aceptación**: falla antes de
  instrumentar.
- [X] T013 Instrumentar `backend/src/services/categorias.rs`: `CREAR`, `ACTUALIZAR`, `DESACTIVAR`,
  `REACTIVAR` en la misma transacción; el handler pasa el actor de `RequiereGestion`.
- [X] T014 Instrumentar `backend/src/services/depositos.rs` (depósitos): `CREAR`, `ACTUALIZAR`,
  `ELIMINAR`, `DESACTIVAR`, `REACTIVAR`; el handler pasa el actor. El `DELETE` con dependencias
  (409) no audita.
- [X] T015 Instrumentar `backend/src/services/depositos.rs` (ubicaciones): `CREAR`, `ACTUALIZAR`,
  `ELIMINAR`, `DESACTIVAR`, `REACTIVAR`; el handler pasa el actor. El `DELETE` con hijos/asignaciones
  (409) no audita.
- [X] T016 Instrumentar la asignación producto–ubicación en `backend/src/services/depositos.rs`:
  `ASIGNAR_UBICACIONES` sobre `productos` (`registro_id = producto_id`), incluso con `[]`; el
  handler de productos pasa el actor.
- [X] T017 Extender `tests/integration/test_rbac_matriz.rs`: consultas SQL por entidad, autor y
  timestamp, y 403 sin filas para las nuevas acciones. **Aceptación**: suite en verde.
- [X] T018 Actualizar `data-model.md`, `contracts/api.md`, `quickstart.md`, `progress.md` y
  `specs/README.md` (FR-019/SC-030) con el alcance ampliado.

## Fase 6 — Cierre de hallazgos del análisis (2026-11-07)

**Goal**: cubrir la matriz completa (incluido el endpoint masivo), las lecturas permitidas, la
degradación de rol, SC-007 directo y la evidencia de gates.

**Independent Test**: ejecutar `test_rbac_matriz` ampliado y comprobar 403/200 según la matriz, el
caso de degradación, el 409 de SC-007 y las filas de auditoría por entidad.

- [X] T019 Agregar `GET /api/productos/ubicaciones` a la matriz de `contracts/api.md`, a
  `test_rbac_matriz.rs` y al conteo de `progress.md`.
- [X] T020 Agregar aserciones de lecturas permitidas a OPERACION en
  `backend/tests/integration/test_rbac_matriz.rs`: `GET /productos/{id}`, `/productos/buscar`,
  `GET /categorias` y `GET /categorias/{id}` → 200.
- [X] T021 Agregar test de degradación de rol en `backend/tests/integration/test_rbac_matriz.rs`:
  sesión GESTION degradada a OPERARIO → siguiente request a `/api/usuarios` responde 403.
- [X] T022 Agregar caso directo de SC-007 en `backend/tests/integration/test_rbac_matriz.rs`:
  `DELETE` de depósito con ubicaciones y de ubicación con hijos → 409; `desactivar` → 204.
- [X] T023 Dejar un único guard en `frontend/src/routes/admin/+layout.svelte` (OPERACION → `/caja`)
  y eliminar los redirects a `/` de `frontend/src/routes/admin/{productos,categorias,depositos,usuarios}/+page.svelte`.
- [X] T024 Registrar evidencia de RED (o waiver) y documentar el alcance real de
  `cargo sqlx prepare --check` para el `INSERT` de auditoría (runtime) en
  `specs/006-rbac-auditoria/progress.md` y `plan.md`.
- [X] T025 Corregir `contracts/api.md` (matriz `POST/PUT` de usuarios sin `DELETE`), `plan.md`
  (`GET /ubicaciones/{id}` y nota de migración) y el conteo de combinaciones en `progress.md`.
- [X] T026 Alinear la nullabilidad de `usuario_id`: `Option<i32>` + `LEFT JOIN` en el consumidor o
  `NOT NULL` con autor definido para acciones de sistema.
- [X] T027 Correr gates completos (`cargo test`, `cargo sqlx prepare --check`, `npm test`,
  `npm run check`, `npm run build`) y actualizar la evidencia.

## Trazabilidad

| Requisito | Tareas |
|---|---|
| FR-003 (verificación) | T007, T019, T020 |
| FR-019 (auditoría de todas las escrituras) | T001–T005, T012–T017 |
| SC-007 | T007, T009a, T022 |
| SC-008 | T007, T020 |
| SC-030 (una fila por escritura, ninguna por rechazo) | T012–T017, T027 |
| US6 escenario 2 (lectura de caja) | T020 |
| US6 escenario 3 | T003–T005, T012–T017 |
| Guard frontend y alternativa de baja lógica | T009, T009a, T023 |
| Caso borde 403 sin auditoría | T007, T017 |
| Hallazgos del análisis (matriz, lecturas, degradación, SC-007, gates) | T019–T026 |
| Soporte (helper/tests, registro, gates, artefactos) | T001a, T006, T008, T010, T011, T027 |
