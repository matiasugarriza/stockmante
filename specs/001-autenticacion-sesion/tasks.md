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
- [X] T012 Implementar helpers Argon2id en `backend/src/auth/password.rs`: `hash_password`, `verify_password`, `generar_password_temporal` (CSPRNG ≥12). **Aceptación**: tests roundtrip hash/verify; temporal cumple política (≥8, 1 mayúscula, 1 dígito); RED→GREEN.
- [X] T013 [P] Implementar `backend/src/mail/mod.rs` con `trait EnvioCorreo { async fn enviar_password_temporal(...) }` + impl `CorreoDev` (log/consola). **Aceptación**: unit test que el dev stub registra el intento sin red.
- [X] T014 [P] Implementar `backend/src/auth/reloj.rs` con `trait Reloj` + impl real (chrono) para expiración de sesiones testeable. **Aceptación**: test de inyección de reloj falso.
- [X] T015 [P] Crear `backend/src/main.rs` con router Axum raíz `/api`, CORS para `APP_URL`, state pool+config, y módulos vacíos de handlers. **Aceptación**: `cargo run` responde 404 JSON de error estándar en `GET /api/salud`; healthcheck simple `GET /api/salud` → 200.
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

- [X] T017 [P] [US1] Unit test servicio login: credenciales inválidas → `CredencialesInvalidas` (mismo resultado email inexistente, password mal, inactivo) en `backend/tests/unit/test_auth_login.rs`. **Aceptación**: test en rojo hasta implementar servicio.
- [X] T018 [P] [US1] Unit test sesión única: segundo login revoca `revocada_en` de la anterior en `backend/tests/unit/test_sesion_unica.rs`. **Aceptación**: RED inicial.
- [X] T019 [P] [US1] Unit test cambio de password temporal: tras cambio, `password_temporal == false` y hash viejo no verifica en `backend/tests/unit/test_cambio_password.rs`. **Aceptación**: RED inicial.

### Implementation para US1

- [X] T020 [US1] Implementar servicio `backend/src/services/auth.rs`: `login`, `logout`, `sesion_actual`, `cambiar_password` según Habilidad M1-A y research R1 (una sesión vigente, expiración con `Reloj`). **Aceptación**: T017–T019 en verde; `cargo test` completo en verde.
- [X] T021 [US1] Implementar extractores `backend/src/auth/extractors.rs`: `AuthUsuario` (cookie → usuario) y rechazo 401 `sesion_expirada`/no autenticada. **Aceptación**: integration test Axum con cookie válida/inválida.
- [X] T022 [US1] Implementar handlers `backend/src/handlers/auth.rs`: `POST /api/auth/login` (Set-Cookie HttpOnly, SameSite=Lax; 204 o 401 genérico), `POST /api/auth/logout`, `GET /api/auth/sesion`, `POST /api/auth/cambiar-password`. **Aceptación**: integration tests de cada endpoint contra BD de test; contrato de contracts/api.md.
- [X] T023 [US1] Seed/bootstrap: migración o binario `backend/src/bin/bootstrap.rs` que crea depósito "Principal" (si no existe) y admin inicial con password temporal impresa solo en consola DEV. **Aceptación**: en BD limpia, `sqlx migrate run` + bootstrap deja un admin activo; en prod no imprime secretos (flag/env). *(El depósito "Principal" es dato inicial de 005; se crea acá porque comparte el bootstrap.)*
- [X] T024 [P] [US1] Frontend: schema Zod `frontend/src/lib/schemas/auth.ts`, cliente `frontend/src/lib/api/auth.ts`, store `frontend/src/lib/stores/sesion.ts`. **Aceptación**: vitest de schemas + cliente mock 401/204.
- [X] T025 [US1] Frontend: ruta `frontend/src/routes/login/+page.svelte` y pantalla de cambio obligatorio de password; layout consulta sesión al arrancar y redirige. **Aceptación**: manual V1/V2 de quickstart.md; sin menú de admin si no hay sesión.

**Checkpoint**: MVP de autenticación usable end-to-end.

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

Base y US1 **implementadas y cerradas**; el resto del Módulo 1 continúa en 002–006.

---

## Notes

- Cada tarea cierra con al menos un commit convencional.
- [P] implica archivos distintos; no paralelizar dos escrituras al mismo archivo.
- No implementar movimientos de stock, alertas ni chatbot en este módulo (fuera de alcance de spec).
- Si un test de la fase FOUNDATIONAL falla por esquema, corregir migración antes de avanzar de fase (Habilidad 2).
