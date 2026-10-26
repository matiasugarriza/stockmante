# Tasks: Gestión de usuarios (002)

**Spec**: [spec.md](./spec.md) | **Plan**: [plan.md](./plan.md) | **Creado**: 2026-10-25

**Input**: Design documents from `/specs/002-gestion-usuarios/`

**Prerequisites**: plan.md, spec.md, data-model.md, contracts/api.md, contracts/validation.md

**Tests**: Sí — Principio IV (TDD RED→GREEN) y quality gates.

**Estado**: todas las tareas `[X]`; el caso de uso está implementado y pusheado.

## Trazabilidad con la spec original

| Original | Actual |
|---|---|
| T026 | T001 |
| T027 | T002 |
| T028 | T003 |
| T029 | T004 |
| T030 | T005 |
| T031 | T006 |

> T007 no proviene de una tarea original: se añadió en el análisis del 2026-10-25 para mapear
> `test_usuarios_alta.rs`, que `plan.md` ya listaba y ninguna tarea cubría.

---

## Phase 1: User Story 2 — Gestión de usuarios (P1)

**Goal**: CRUD usuarios por familia GESTION, contraseña temporal por correo, desactivar/reactivar,
guarda del último gestor activo, paginación.

**Independent Test**: Admin crea Operario → recibe temporal; email duplicado → 409; desactivar
último gestor → 409; desactivar con sesión → sesión cae; Operario → 403 en CRUD.

### Tests FIRST (RED)

> *(2026-10-25: RED observado. Los 4 targets unitarios nuevos no compilaban por
> `services::usuarios` inexistente; el integration de T005 arrancó 6/6 en rojo porque las rutas
> `/api/usuarios` todavía no existían.)*

- [X] T001 [P] [US2] Unit test: unicidad de email (mayúsculas/espacios) → `EmailDuplicado` y formato inválido → `Validacion` (FR-004, FR-008) en `backend/tests/unit/test_usuarios_email.rs`. **Aceptación**: RED. *(Evidencia: 3 tests verdes en `f1fc3e8`; regresión de emails con múltiples `@`, puntos consecutivos y en bordes en `test_usuarios_http.rs` (`2878b4b`).)*
- [X] T002 [P] [US2] Unit test: desactivar último `familia_rol=GESTION` activo → `UltimoAdministrador` en `backend/tests/unit/test_ultimo_admin.rs`. **Aceptación**: RED. *(Verde en `f1fc3e8`.)*
- [X] T003 [P] [US2] Unit test: mapa rol→familia (ADMIN/GERENTE→GESTION; OPERARIO/CAJERO→OPERACION) en `backend/tests/unit/test_familia_rol.rs`. **Aceptación**: RED. *(Verde en `f1fc3e8`.)*
- [X] T007 [P] [US2] Unit test: el alta entrega la temporal por correo, nunca expone el hash y revierte la transacción si el envío falla, en `backend/tests/unit/test_usuarios_alta.rs`. **Aceptación**: RED. *(ID asignado en el análisis del 2026-10-25 sin renumerar T001–T006. Evidencia: 1 test en `f1fc3e8`; el fail-closed se añadió en `1d7fffd`. Cierra el target que `plan.md` ya listaba.)*

### Implementation

- [X] T004 [US2] Implementar servicio `backend/src/services/usuarios.rs`: crear (genera temporal + `EnvioCorreo`), listar, obtener, actualizar, desactivar (guarda del último gestor y revocación de sesiones), reactivar, regenerar password temporal. **Aceptación**: T001–T003 verdes; sin DELETE físico. *(Commit `f1fc3e8`.)*
- [X] T005 [US2] Implementar handlers `backend/src/handlers/usuarios.rs` bajo `RequiereGestion`: GET/POST/PUT, desactivar/reactivar/password-temporal. **Aceptación**: integration tests 201/200/204/409/422/403/404; sin `password_hash` en respuestas. *(Commit `f1fc3e8`; endurecido en `977bd90`.)*
- [X] T006 [P] [US2] Frontend: schemas + cliente, tabla/modal de usuarios, ruta `frontend/src/routes/admin/usuarios/+page.svelte`, link solo para GESTION. **Aceptación**: vitest; UI lista/crea/desactiva con mensajes `ApiError`. *(Commit `f1fc3e8`; paginación en `8fa8390`.)*

**Checkpoint**: US2 cerrada; RBAC de gestión operativo en usuarios.

---

## Verificación y gates

- [X] `cargo test` en verde (45/45 al cierre de US2; 62/62 al 2026-10-25 tras cerrar 001 y migrar a macros).
- [X] `cargo sqlx prepare --check` sin diferencias; `SQLX_OFFLINE=true cargo check --all-targets` OK
  tras migrar `services/usuarios.rs` a macros (2026-10-25, `67032c7`).
- [X] `npm test`, `npm run check` (0/0) y `npm run build` OK.
- [X] Grep sin `unwrap`/`panic!`/`expect` en `backend/src`.
- [X] Verificación independiente: aprobado con advisories, 0 blockers (`f1fc3e8..d87a04f`) —
  detalle en [progress.md](./progress.md).
- [X] Push a `origin/main` (`6e1d722..d87a04f`).

## Notes

- La contraseña temporal y el bloqueo por cambio obligatorio son FR-008a de 001.
- El alta envía el correo dentro de la transacción: si falla, no queda cuenta huérfana.
- Los locks de gestores se toman en orden de `id` para evitar deadlocks entre bajas concurrentes.
