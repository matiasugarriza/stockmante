# Tasks: Base técnica y autenticación (spec 001)

**Spec**: [spec.md](./spec.md) | **Plan**: [plan.md](./plan.md) | **Creado**: 2026-10-12 | **Dividido**: 2026-10-25

**Input**: Design documents from `/specs/001-autenticacion-sesion/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/api.md, contracts/validation.md

**Tests**: Sí — exigidos por constitución Principio IV (TDD RED→GREEN en lógica de dominio) y quality gates. Cada tarea de lógica incluye o depende de su test.

**Alcance**: esta spec posee la **base técnica compartida** (Phase 1 + Phase 2) y el caso de uso
**US1** (autenticación y sesión). Las fases de US2–US6 y el cierre transversal se movieron a sus
propias specs (ver [trazabilidad](#tareas-movidas-a-otras-specs)).

**Skills obligatorios** (ver `SKILLS.md`): Habilidad 2 antes de cada SQL; M1-A…M1-G según el tipo de tarea.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: paralelizable (archivos distintos, sin dependencias incompletas)
- **[Story]**: US1 (única user story de esta spec)
- Paths exactos según plan.md (`backend/`, `frontend/`, `/migrations` en raíz)

## Path Conventions

- Backend: `backend/src/…`, `backend/tests/…`
- Frontend SvelteKit: `frontend/src/…`
- Migraciones: `migrations/` en la **raíz** del repo (constitución)

---

## Phase 1: Setup (infraestructura compartida)

**Purpose**: Inicializar monorepo backend + frontend + entorno, sin lógica de negocio.

- [X] T001 Crear estructura de directorios del monorepo según plan.md: `backend/src/{models,services,handlers,auth,mail}`, `backend/tests/{unit,integration}`, `frontend/src/{lib/api,lib/schemas,lib/stores,lib/components,routes}`, `migrations/`, `.env.example` en la raíz. **Aceptación**: los paths existen; sin código de negocio aún.
- [X] T002 Inicializar `backend/Cargo.toml` con dependencias del plan (axum, sqlx postgres runtime-tokio, tokio, serde, serde_json, validator, argon2, tower-http cookies/cors, uuid, chrono, thiserror, lettre opcional feature). **Aceptación**: `cargo check` compila con un `main.rs` vacío que solo imprime versión.
- [X] T003 [P] Inicializar proyecto SvelteKit en `frontend/` en modo SPA (`@sveltejs/adapter-static`), Tailwind CSS, TypeScript estricto, vitest. **Aceptación**: `npm run dev` levanta la SPA; `npm test` corre un smoke test vacío en verde. Frontend = SvelteKit + Tailwind (no app Node).
- [X] T004 [P] Crear `.env.example` en la raíz con `DATABASE_URL`, `PUBLIC_API_URL`, `SESSION_COOKIE_NAME`, `SESSION_EXPIRY_HOURS`, `APP_URL`, `APP_ENV`, `EMAIL_FROM`, `SMTP_URL`. **Aceptación**: archivo presente; ningún secreto real; alineado con quickstart.md.
- [X] T005 Configurar SQLx con `migrations_dir = "migrations"` (raíz) vía `sqlx.toml` o flags de `sqlx-cli` documentados en README. **Aceptación**: `sqlx migrate info` funciona desde la raíz apuntando a `/migrations`.

**Checkpoint**: monorepo compila/arranca vacío; migraciones configuradas.

---

## Phase 2: Foundational (bloqueante para TODAS las specs de usuario)

**Purpose**: Esquema de datos del módulo, `AppError`, pool, config, primitivas de auth compartidas.

**⚠️ CRITICAL**: No empezar US1 ni ninguna otra spec de usuario (002–006) hasta completar esta fase.

> La fase se planificó acá porque la base es compartida. Las tablas de catálogo y depósitos se
> crearon en esta fase para no bloquear 003/005; su **propiedad de esquema** es de esas specs
> (ver ledger en [specs/README.md](../README.md)).

- [X] T006 Verificar con Habilidad 2 (MCP o `sqlx table list` / `sqlx describe`) que la BD de desarrollo no tiene nombres en conflicto con las tablas del Módulo 1. **Aceptación**: reporte de tablas actuales; si hay conflicto, renombrar en data-model antes de migrar. *(2026-10-13: `\dt` sobre `stockmante` → solo `_sqlx_migrations`; sin conflictos.)*
- [X] T007 Crear migración `migrations/0001_usuarios_sesiones.sql`: tabla `usuarios` (campos y CHECK de data-model.md: email UNIQUE, `familia_rol CHECK IN ('GESTION','OPERACION')`, `activo`, `password_hash`, `password_temporal`) y `sesiones` (UUID PK, FK usuario, `expira_en`, `revocada_en`). **Aceptación**: `sqlx migrate run` aplica sin error; `sqlx describe usuarios` / `sesiones` coinciden con data-model.md.
- [X] T008 Crear migración `migrations/0002_categorias_productos.sql`: `categorias` (nombre UNIQUE), `productos` (SKU CHECK `^[A-Z0-9][A-Z0-9-]{2,29}$` + UNIQUE, `codigo_barras` UNIQUE, FK categoría, costo/precio NUMERIC ≥0, stock_minimo ≥0, activo). **Aceptación**: `sqlx migrate run` OK; constraints visibles en `sqlx describe productos`. *(Propiedad de esquema: 003. El `UNIQUE NULLS NOT DISTINCT` original se corrigió en `0004` — 003/T003.)*
- [X] T009 Crear migración `migrations/0003_depositos_ubicaciones.sql`: `depositos`, `ubicaciones` (FK deposito NOT NULL, FK padre, UNIQUE deposito+padre+nombre), `producto_ubicacion_permitida` (PK compuesta). **Aceptación**: `sqlx migrate run` OK; no se puede insertar ubicación sin deposito_id. *(Propiedad de esquema: 005.)*
- [X] T010 [P] Implementar `backend/src/error.rs`: enum `AppError` con variantes de contracts (M1-E) + `IntoResponse` con shape `{ "error": { "codigo", "mensaje" } }`; Database/Interno → 500 genérico sin exponer `sqlx::Error`. **Aceptación**: unit test de mapeo de status/codigo para cada variante de dominio en verde.
- [X] T011 [P] Implementar `backend/src/config.rs` + `backend/src/db.rs`: carga de env, pool SQLx, constantes de sesión (cookie name, expiry horas). **Aceptación**: `cargo test` de config con env de test; pool construible con `DATABASE_URL` de test.
- [X] T012 Implementar helpers Argon2id en `backend/src/auth/password.rs`: `hash_password`, `verify_password`, `generar_password_temporal` (CSPRNG, 14 caracteres; ≥12 y política de ≥8, 1 mayúscula, 1 dígito). **Aceptación**: tests roundtrip hash/verify; temporal cumple política y longitud; RED→GREEN. *(Evidencia 2026-10-25: `TEMP_LEN = 14`; `test_password` en verde.)*
- [X] T013 [P] Implementar `backend/src/mail/mod.rs` con `trait EnvioCorreo { async fn enviar_password_temporal(...) }` + impl `CorreoDev` (log/consola), sin exponer la temporal fuera de desarrollo. **Aceptación**: unit test que el dev stub registra el intento sin red y que el stub por defecto no imprime la credencial. *(El transporte SMTP real es T029.)*
- [X] T014 [P] Implementar `backend/src/auth/reloj.rs` con `trait Reloj` + impl real (chrono) para expiración de sesiones testeable. **Aceptación**: test de inyección de reloj falso.
- [X] T015 [P] Crear `backend/src/main.rs` con router Axum raíz `/api`, CORS para `APP_URL`, state pool+config, y módulos vacíos de handlers. **Aceptación**: `GET /api/salud` → 200 `{ "ok": true }`; cualquier ruta desconocida → 404 con el sobre de error estándar. *(Endpoint operativo, sin datos de negocio; contrato en `contracts/api.md`.)*
- [X] T016 [P] Scaffolding frontend: `frontend/src/lib/api/client.ts` (fetch con `credentials: 'include'` y `ApiError`), `frontend/src/lib/schemas/`, layout raíz con Tailwind. **Aceptación**: `npm test` de un test unitario de `ApiError` parseando un body de error; build SPA OK.

**Checkpoint**: esquema completo del módulo en BD; AppError y primitivas listas; las specs 001–006 pueden empezar.

---

## Phase 3: User Story 1 — Autenticación y sesión segura (P1) 🎯 MVP

**Goal**: Login con mensaje genérico, sesión única por usuario (cookie HttpOnly), logout, vista de sesión, cambio obligatorio de contraseña temporal.

**Independent Test**: Con usuario bootstrap/dev: login OK → cookie emitida; login password mala / inactivo → 401 mismo body; segundo login revoca el primero; logout invalida; cambiar password temporal marca `password_temporal=false`.

### Tests FIRST (RED) para US1

> Escribir estos tests y verlos fallar antes del servicio/handler.
>
> *(2026-10-20: los tests T017–T019 y el servicio T020 se escribieron en la misma
> tanda por la deuda de contexto de la sesión; la cobertura y el mapeo de casos
> T-AUTH-01..05 están verificados en verde, pero el paso RED formal no se observó.)*
>
> **Waiver P-IV (2026-10-25)**: se registra la desviación para T017–T019. No se
> puede reconstruir el RED retroactivamente; la mitigación es que la cobertura
> está en verde y que 003–006 deben observar RED antes de implementar. La
> regularización de gobernanza (enmienda o waiver permanente) es T032.

- [X] T017 [P] [US1] Unit test servicio login: credenciales inválidas → `CredencialesInvalidas` (mismo resultado email inexistente, password mal, inactivo) en `backend/tests/unit/test_auth_login.rs`. **Aceptación**: test en rojo hasta implementar servicio.
- [X] T018 [P] [US1] Unit test sesión única: segundo login revoca `revocada_en` de la anterior en `backend/tests/unit/test_sesion_unica.rs`; incluye desactivar usuario con sesión (SC-004, T-AUTH-04) → siguiente request 401. **Aceptación**: RED inicial. *(Evidencia: `desactivar_usuario_deja_sesiones_invalidas`.)*
- [X] T019 [P] [US1] Unit test cambio de password temporal: tras cambio, `password_temporal == false` y hash viejo no verifica en `backend/tests/unit/test_cambio_password.rs`. **Aceptación**: RED inicial.

### Implementation para US1

- [X] T020 [US1] Implementar servicio `backend/src/services/auth.rs`: `login`, `logout`, `sesion_actual`, `cambiar_password` según Habilidad M1-A y research R1 (una sesión vigente, expiración con `Reloj`). **Aceptación**: T017–T019 en verde; `cargo test` completo en verde.
- [X] T021 [US1] Implementar extractores `backend/src/auth/extractors.rs`: `AuthUsuario` (cookie → usuario; 401 `no_autenticado`/`sesion_expirada`) y `RequiereGestion` (familia OPERACION → 403 `sin_permisos`). **Aceptación**: integration test Axum con cookie válida/inválida; test de 403 para OPERACION en endpoints de gestión. *(Evidencia: `test_usuarios_http.rs::operario_recibe_403_en_la_gestion_de_usuarios`; el gate global de temporal es T028.)*
- [X] T022 [US1] Implementar handlers `backend/src/handlers/auth.rs`: `POST /api/auth/login` (Set-Cookie HttpOnly, SameSite=Lax, `Secure` en producción; 204 o 401 genérico), `POST /api/auth/logout`, `GET /api/auth/sesion`, `POST /api/auth/cambiar-password`. **Aceptación**: integration tests de cada endpoint contra BD de test; contrato de contracts/api.md. *(El flag `Secure` en producción es T030.)*
- [X] T023 [US1] Seed/bootstrap: migración o binario `backend/src/bin/bootstrap.rs` que crea depósito "Principal" (si no existe) y admin inicial con password temporal impresa solo en consola DEV. **Aceptación**: en BD limpia, `sqlx migrate run` + bootstrap deja un admin activo; en prod no imprime secretos (flag/env). *(El depósito "Principal" es dato inicial de 005; se crea acá porque comparte el bootstrap.)*
- [X] T024 [P] [US1] Frontend: schema Zod `frontend/src/lib/schemas/auth.ts`, cliente `frontend/src/lib/api/auth.ts`, store `frontend/src/lib/stores/sesion.ts`. **Aceptación**: vitest de schemas + cliente mock 401/204.
- [X] T025 [US1] Frontend: ruta `frontend/src/routes/login/+page.svelte` y pantalla de cambio obligatorio de password; layout consulta sesión al arrancar y redirige. **Aceptación**: manual V1/V2 de quickstart.md; sin menú de admin si no hay sesión.

**Checkpoint**: MVP de autenticación usable end-to-end.

---

## Correcciones post-análisis (2026-10-25)

> Abiertas por `/speckit.analyze` sobre 001. No reabren T001–T025: son cambios acotados de implementación, pruebas y gobernanza. El Módulo 1 no se considera cerrado hasta que T026–T032 estén `[X]` o formalmente waiveadas.

- [X] T026 [US1] Corregir la semántica de expiración: documentar y probar **expiración absoluta** de 8 h desde el login (sin sliding), y que `ultima_actividad` no participa del cálculo. **Aceptación**: test que una sesión deja de valer a las 8 h aunque haya actividad; test que no se renueva por requests; `SESSION_EXPIRY_HOURS` documentado. **Archivos**: `backend/tests/unit/test_sesion_unica.rs`, `specs/001-autenticacion-sesion/data-model.md`. *(Evidencia: `la_actividad_no_renueva_la_sesion` — actividad a 7h59 deja la sesión vigente, a 8h01 da `SesionExpirada` y `expira_en` conserva login+8h; el test fue de caracterización porque el comportamiento ya era absoluto. `data-model.md` §sesiones y `.env.example` documentan la expiración absoluta y `SESSION_EXPIRY_HOURS`.)*
- [X] T027 [US1] Garantizar la sesión única en BD: agregar índice único parcial `UNIQUE (usuario_id) WHERE revocada_en IS NULL` (o lock `FOR UPDATE` del usuario en login) y manejar la violación `23505` sin filtrar detalles. **Aceptación**: test de dos logins concurrentes deja una sola sesión vigente; la cookie vieja responde 401. **Archivos**: nueva migración `migrations/0005_sesion_unica.sql` (no editar `0001`, ya aplicada), `backend/src/services/auth.rs`. *(Evidencia: migración 0005 aplicada en dev y test — repara duplicados heredados y crea el índice único parcial; `login` bloquea el usuario con `SELECT ... FOR UPDATE` y mapea la violación de unicidad a 500 genérico. Verdes: `la_bd_impide_dos_sesiones_vigentes_por_usuario`, `dos_logins_concurrentes_dejan_una_sola_sesion_vigente`, `segundo_login_revoca_la_sesion_anterior`.)*
- [X] T028 [US1] Implementar el gate global de contraseña temporal: `RequierePasswordCambiada` (o middleware) aplicado a todo endpoint autenticado salvo `GET /auth/sesion` y `POST /auth/cambiar-password`; el orden debe ser temporal → familia. **Aceptación**: OPERACION y GESTION con temporal reciben 403 `cambio_password_requerido` en endpoints protegidos; sin temporal pasan; tests unitarios y de integración. **Archivos**: `backend/src/auth/extractors.rs`, `backend/src/handlers/*`, `specs/001-autenticacion-sesion/contracts/api.md`. *(Evidencia: `auth/gate.rs::exigir_password_cambiada` como middleware del grupo autenticado en `app.rs`; `RequiereGestion` evalúa temporal antes que familia. Verdes: `el_gate_de_password_temporal_gana_a_la_familia` (OPERARIO → `cambio_password_requerido`), `gestor_con_password_temporal_no_puede_gestionar_hasta_cambiarla` (GESTION) y `salud_y_sesion_siguen_abiertos_con_temporal_pendiente`; los tests sin temporal siguen pasando. El contrato y el data-model ya describían el gate global.)*
- [X] T029 [US1] Implementar el transporte SMTP real de la temporal (`lettre`) detrás de `SMTP_URL`; en producción, si el envío falla, el alta se revierte (fail-closed). **Aceptación**: test con transporte falso que falla y no deja usuario; test de configuración de producción sin `SMTP_URL` falla al arrancar o al crear; `CorreoDev` sigue siendo el default de desarrollo. **Archivos**: `backend/src/mail/`, `backend/src/config.rs`, `backend/tests/unit/test_mail.rs`. *(Evidencia: `lettre` 0.11 con `builder`+`smtp-transport`+`tokio1-rustls-tls`; `CorreoSmtp::desde_url`; `AppConfig::validar()` exige `SMTP_URL` en producción y `main` elige SMTP o `CorreoDev`. Verdes: `si_el_correo_falla_no_queda_usuario`, `produccion_sin_smtp_url_no_valida`, `smtp_rechaza_url_o_remitente_invalidos`. Excepción de RED: el rollback fail-closed ya existía (caracterización) y `CorreoSmtp`/`validar` son API nueva que no compilaba antes de existir; `.env.example` documenta SMTP real.)*
- [X] T030 [US1] Endurecer la cookie de sesión (FR-002a): `Secure` cuando `APP_ENV=production`; mantener HttpOnly, SameSite=Lax y `path=/`. **Aceptación**: test del atributo `Secure` en producción y ausencia en desarrollo; contrato actualizado. **Archivos**: `backend/src/handlers/auth.rs`, `specs/001-autenticacion-sesion/contracts/api.md`. *(Evidencia: `la_cookie_lleva_secure_solo_en_produccion` — con `APP_ENV=production` la cookie suma `Secure` y conserva HttpOnly/SameSite=Lax/path=/; en desarrollo no lo lleva. `cookie_sesion` y `cookie_sesion_borrada` usan `config.es_produccion()`.)*
- [X] T031 [US1] Resolver C1 (SQLx): elegir entre macros `query!`/`query_as!` + `.sqlx` versionado o enmienda constitucional que permita runtime API con un gate equivalente; retirar `validator` si no se usa. **Aceptación**: decisión registrada en `plan.md`/constitución; `cargo sqlx prepare --check` deja de ser no-op o queda justificado con un gate alternativo; dependencia `validator` eliminada o usada. **Archivos**: `backend/Cargo.toml`, `backend/src/**/*.rs`, `.specify/memory/constitution.md`. *(Evidencia: opción A — `query!`/`query_as!`/`query_scalar!` en `services/auth.rs` y `bin/bootstrap.rs` (14 queries), `backend/.sqlx/` versionado, `validator` retirado; `cargo sqlx prepare --check` verde y `SQLX_OFFLINE=true cargo check --all-targets` sin BD. Decisión registrada en `plan.md`; las queries de 002/003 se migraron el 2026-10-25 como cierre del follow-up (`.sqlx` regenerado y `prepare --check` verde).)*
- [X] T032 [US1] Gobernanza: proponer/aprobar la enmienda P-III (excepción anti-enumeración), la enmienda «Stack fijo» (SQLx) y el waiver P-IV (RED no observado); actualizar el Sync Impact Report y la versión constitucional. **Aceptación**: enmiendas aprobadas y commiteadas con `docs: amend constitution to vX.Y.Z`; `tasks.md`/`progress.md` reflejan el resultado. **Archivos**: `.specify/memory/constitution.md`, `specs/001-autenticacion-sesion/tasks.md`. *(Evidencia: constitución **1.2.0** (MINOR) con P-III enmendado — excepción acotada a `POST /api/auth/login` — y sección Governance → «Waivers registrados» con W-001 (P-IV, T017–T019). La propuesta «Stack fijo» quedó descartada al resolver C1 con macros en T031. Validado: sin placeholders, sin espacios finales, sin BOM, versión y fechas ISO coincidentes.)*

### Trazabilidad de hallazgos

| Hallazgo | Tarea |
|---|---|
| H1 / M3 (expiración absoluta / `ultima_actividad`) | T026 |
| H2 (sesión única concurrente) | T027 |
| H3 (gate temporal global) | T028 |
| H4 (SMTP real / fail-closed) | T029 |
| M2 (cookie `Secure`) | T030 (FR-002a) |
| M4 / C1 (stack y SQLx) | T031 |
| C2 / C3 (constitución y waiver) | T032 |
| M1 (longitud de temporal) | T012 (documentación) |
| M5 (`/api/salud`) | T015 |
| M8 (SC-004) | T018 (evidencia) |
| M9 (`RequiereGestion`) | T021 |

---

## Tareas movidas a otras specs

La spec original (`specs/001-user-auth-catalog/`) agrupaba 6 casos de uso. A partir del 2026-10-25
cada caso de uso tiene su propia spec y **reinicia la numeración en `T001`**:

| Tareas originales | Spec actual | Contenido |
|---|---|---|
| T026–T031 | [002-gestion-usuarios](../002-gestion-usuarios/tasks.md) | US2 — CRUD de usuarios |
| T032–T036 | [003-catalogo-productos](../003-catalogo-productos/tasks.md) | US3 — productos y categorías |
| T037–T039 | [004-busqueda-caja](../004-busqueda-caja/spec.md) | US4 — búsqueda para caja |
| T040–T044 | [005-depositos-ubicaciones](../005-depositos-ubicaciones/spec.md) | US5 — depósitos y ubicaciones |
| T045–T047 | [006-rbac-auditoria](../006-rbac-auditoria/spec.md) | US6 — RBAC transversal y auditoría |
| T048–T052 | [specs/README.md](../README.md) — C1–C5 | Cierre transversal del Módulo 1 |

> La numeración `T001–T025` de esta spec se conserva igual que en el original.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: sin dependencias.
- **Foundational (Phase 2)**: depende de Phase 1 — **bloquea todas las specs de usuario**.
- **US1 (Phase 3)**: solo Phase 2. **MVP**.

### Dependencias de otras specs sobre esta

| Spec | Necesita de 001 |
|---|---|
| 002 | extractores de sesión y `RequiereGestion` (T021), primitivas de password y correo (T012–T013) |
| 003 | `RequiereGestion`, `AppError`, modelos base |
| 004 | catálogo de 003 + auth |
| 005 | auth y bootstrap del depósito "Principal" (T023) |
| 006 | endpoints de 001–005 |

### Within US1

- Tests RED → servicio → extractores → handlers → frontend → commit convencional.
- `sqlx migrate run` + Habilidad 2 antes de cada query nueva.
- `cargo sqlx prepare --check` al cerrar cada tarea que toque SQL.

### Parallel Opportunities

- T003, T004, T005 (setup) en paralelo tras T001–T002.
- T010–T016 (AppError, config, mail, reloj, scaffolding) marcadamente [P].
- Tests unitarios de US1 (T017–T019) en paralelo.

---

## Implementation Strategy

### MVP First (base + US1)

1. Phase 1 + Phase 2 completos.
2. Phase 3 (US1) completa.
3. **STOP**: validar V1/V2/V4 de quickstart (login, sesión única, mensaje genérico).
4. Demo: login + logout + cambio de password temporal.

### Estado

Base y US1 **implementadas**; T026–T032 cerradas con evidencia (2026-10-25). La gobernanza quedó regularizada en la constitución **1.2.0** (excepción acotada de P-III + waiver W-001) y 001 se declara **cerrada**. El Módulo 1 continúa en [003-catalogo-productos](../003-catalogo-productos/tasks.md) (US3, T004).

---

## Notes

- Cada tarea cierra con al menos un commit convencional.
- [P] implica archivos distintos; no paralelizar dos escrituras al mismo archivo.
- No implementar movimientos de stock, alertas ni chatbot en este módulo (fuera de alcance de spec).
- Si un test de la fase FOUNDATIONAL falla por esquema, corregir migración antes de avanzar de fase (Habilidad 2).
