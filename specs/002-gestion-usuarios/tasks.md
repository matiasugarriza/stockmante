# Tasks: Gestión de usuarios (002)

**Spec**: [spec.md](./spec.md) | **Plan**: [plan.md](./plan.md) | **Creado**: 2026-10-25

**Input**: Design documents from `/specs/002-gestion-usuarios/`

**Prerequisites**: plan.md, spec.md, data-model.md, contracts/api.md, contracts/validation.md

**Tests**: Sí — Principio IV (TDD RED→GREEN) y quality gates.

**Estado**: Phase 1 (T001–T007) y Phase 2 (T008–T016, `FR-008b`) **completas y verificadas** (2026-11-01).

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
> T008 en adelante son la Phase 2 del ajuste 2026-11 (`FR-008b`).

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

## Phase 2: FR-008b — Credenciales manual/automáticas con visualización en pantalla (2026-11)

**Goal**: el Administrador elige la contraseña (manual o automática) en el alta y en el regenerado,
y el sistema muestra obligatoriamente una modal con correo + clave en claro para entrega inmediata,
además del correo por el transporte configurado.

**Independent Test**: alta con clave manual válida → modal muestra esa clave; alta sin clave →
modal muestra la automática de 14 caracteres; clave manual débil → 422 y nada persistido;
regenerar (manual o automática) → modal con la clave nueva; los GET no exponen claves.

### Tests FIRST (RED)

> *(2026-11-01: RED observado. T008–T009 no compilaban contra la firma nueva de
> `crear(..., password)` (E0061/E0308) y T010 quedaba en rojo de aserciones
> (201 con forma de item y 204 en regenerar). Frontend: 7 tests en rojo con los
> nuevos `urlInicioSso`/envoltura/política antes de implementar.)*

- [X] T008 [P] [US2] Unit test en `backend/tests/unit/test_usuarios_alta.rs`: (a) alta con `password` manual válida devuelve **esa misma** clave en claro y permite loguearse con ella sin exponer el hash; (b) alta con `password` `pass1` → `AppError::Validacion` (422) y nada persistido (FR-008b, plan D5). **Aceptación**: RED observado y GREEN tras T011. *(Evidencia: `alta_con_password_manual_devuelve_esa_clave` y `alta_con_password_manual_debil_es_validacion` verdes.)*
- [X] T009 [P] [US2] Unit test: (a) alta **sin** `password` devuelve una clave automática de exactamente 14 caracteres con ≥1 mayúscula y ≥1 dígito; (b) ambos modos envían el correo por `EnvioCorreo` (plan D4) y la clave del correo coincide con la devuelta. **Aceptación**: RED observado y GREEN. *(Evidencia: `alta_sin_password_genera_automatica_de_14_caracteres`.)*
- [X] T010 [P] [US2] Integration HTTP en `backend/tests/integration/test_usuarios_http.rs`: (a) `POST /api/usuarios` 201 con body `{ usuario, password_temporal }`; (b) 422 con clave manual `pass1`; (c) `POST /api/usuarios/{id}/password-temporal` sin body → 200 `{ password_temporal }` de 14, con `{ password: "Manual99" }` → 200, con clave débil → 422; (d) `GET /api/usuarios` **nunca** incluye la clave en claro. **Aceptación**: GREEN en `alta_devuelve_201_sin_secretos_y_la_temporal_sirve_para_login` (actualizado a la envoltura), `alta_con_password_manual_valida_y_debil` y `password_temporal_responde_200_con_clave_y_valida_manual`; 13/13 en el integration.

### Implementation

- [X] T011 [US2] Backend alta: `CrearUsuarioDto.password: Option<String>`; `services::usuarios::crear(..., password: Option<&str>)` → `(Usuario, String)` con `elegir_password_temporal` (valida política si manual con `validar_politica_password`; automática con `generar_password_temporal`); handler devuelve 201 `{ usuario, password_temporal }` con modelo `UsuarioCreado` y anotación utoipa actualizada (plan D1, D2, D5). **Aceptación**: T008–T009 verdes; 23505 sigue → 409; correo y auditoría dentro de la transacción.
- [X] T012 [US2] Backend regenerar: `regenerar_password_temporal` acepta `password: Option<&str>` y devuelve la clave (misma rutina que el alta, plan D6); handler `POST /{id}/password-temporal` con body JSON **opcional** `{ password? }` vía extractor nuevo `JsonValidoOpcional` → 200 `{ password_temporal }` (antes 204; plan D3); mantiene revocación de sesiones y correo en transacción. **Aceptación**: T010 verdes; 404/403 sin cambios (`test_rbac_matriz` y gate de temporal actualizados al 200).
- [X] T013 [P] [US2] Frontend schemas + cliente: `CrearUsuarioSchema.password` opcional con política Zod (`PoliticaPasswordSchema`); `ActualizarUsuarioSchema` omite `password` (plan D8); `UsuarioCreadoSchema`/`ClaveTemporalGeneradaSchema`; `crearUsuario()` y `regenerarPasswordTemporal(id, password?)` devuelven las credenciales; tests vitest (`T-FE-USER-02`). **Aceptación**: `npm test` 174/174.
- [X] T014 [US2] Frontend alta: selector manual/automática en `UsuarioForm.svelte` (radios + input de clave con validación Zod cuando es manual; texto «Al crearla vas a ver la contraseña en pantalla; también se envía por correo»). **Aceptación**: alta manual/auto renderiza y bloquea claves débiles antes del request (`svelte-check` 0/0).
- [X] T015 [US2] Frontend credenciales: nuevo `CredencialesModal.svelte` (correo + clave, botón «Copiar contraseña» con feedback, aviso de cambio obligatorio y de envío por correo) y nuevo `PasswordTemporalModal.svelte` (selector manual/automática para el regenerado, escenario 5); cableado en `routes/admin/usuarios/+page.svelte` — tras el alta y tras regenerar se abre la modal (plan D6). **Aceptación**: `npm run check` 0/0; modal bloqueante exigida antes de cerrar (spec: «muestra obligatoriamente»).
- [X] T016 [P] [US2] Ajustes de docs: `quickstart.md` (V3 con método de contraseña + modal; fila `password-temporal` 204→200), OpenAPI/utoipa (bodies y responses nuevos + `docs.rs` registra `UsuarioCreado`, `ClaveTemporalGenerada`, `PasswordTemporalDto` y los DTOs de auth que faltaban). **Aceptación**: `cargo test` en verde con el OpenApi derivado compilando; contratos y quickstart coherentes con `contracts/api.md`.

**Checkpoint**: FR-008b completa — escenarios 1 y 5 de la spec verificados de punta a punta.

---

## Verificación y gates

- [X] `cargo test` en verde (45/45 al cierre de US2; 55/55 al 2026-10-25 tras cerrar 001, migrar a macros y deshacer el WIP de 003).
- [X] `cargo sqlx prepare --check` sin diferencias; `SQLX_OFFLINE=true cargo check --all-targets` OK
  tras migrar `services/usuarios.rs` a macros (2026-10-25, `67032c7`).
- [X] `npm test`, `npm run check` (0/0) y `npm run build` OK.
- [X] Grep sin `unwrap`/`panic!`/`expect` en `backend/src`.
- [X] Verificación independiente: aprobada con advisories, 0 blockers (`f1fc3e8..d87a04f`) —
  detalle en [progress.md](./progress.md).
- [X] Push a `origin/main` (`6e1d722..d87a04f`).
- [X] **Gates de Phase 2** (2026-11-01): `cargo test` **158/158** (todos los targets, sin
  fallos), `cargo sqlx prepare --check` exit 0 (sin tocar queries), `npm test` **174/174**,
  `npm run check` **0/0**, `npm run build` OK, grep sin `unwrap`/`panic!`/`expect` en los
  caminos nuevos. Commits: ver `git log` (SSO/registro en `0b23c23`; Phase 2 en el commit de
  cierre de este cambio).

## Notes

- La contraseña temporal y el bloqueo por cambio obligatorio son `FR-008a` de 001; la elección
  manual/automática y la modal de entrega en pantalla son `FR-008b` de 002 (ver
  [spec.md](./spec.md) y `specs/README.md` § Partición de FRs).
- El alta envía el correo dentro de la transacción: si falla, no queda cuenta huérfana (plan D4).
- Los locks de gestores se toman en orden de `id` para evitar deadlocks entre bajas concurrentes.
- La clave en claro solo vive en la respuesta 201/200 de alta/regenerado: los GET no la exponen
  y ningún log la registra (plan D2, D7).