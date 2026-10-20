# Tasks: Módulo 1 — Usuarios, Autenticación y Catálogo Base

**Input**: Design documents from `/specs/001-user-auth-catalog/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/api.md, contracts/validation.md

**Tests**: Sí — exigidos por constitución Principio IV (TDD RED→GREEN en lógica de dominio) y quality gates. Cada tarea de lógica incluye o depende de su test.

**Organization**: Tareas agrupadas por user story para implementación y prueba independientes.

**Skills obligatorios** (ver `SKILLS.md`): Habilidad 2 antes de cada SQL; M1-A…M1-G según el tipo de tarea.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: paralelizable (archivos distintos, sin dependencias incompletas)
- **[Story]**: US1…US6 (solo en fases de user story)
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
- [X] T004 [P] Crear `.env.example` en la raíz con `DATABASE_URL`, `PUBLIC_API_URL`, `SESSION_COOKIE_NAME`, `SESSION_EXPIRY_HOURS`, `APP_URL`, `EMAIL_FROM`, `SMTP_URL`. **Aceptación**: archivo presente; ningún secreto real; alineado con quickstart.md.
- [X] T005 Configurar SQLx con `migrations_dir = "migrations"` (raíz) vía `sqlx.toml` o flags de `sqlx-cli` documentados en README. **Aceptación**: `sqlx migrate info` funciona desde la raíz apuntando a `/migrations`.

**Checkpoint**: monorepo compila/arranca vacío; migraciones configuradas.

---

## Phase 2: Foundational (bloqueante para TODAS las user stories)

**Purpose**: Esquema de datos del módulo, `AppError`, pool, config, primitivas de auth compartidas.

**⚠️ CRITICAL**: No empezar US1–US6 hasta completar esta fase.

- [X] T006 Verificar con Habilidad 2 (MCP o `sqlx table list` / `sqlx describe`) que la BD de desarrollo no tiene nombres en conflicto con las tablas del Módulo 1. **Aceptación**: reporte de tablas actuales; si hay conflicto, renombrar en data-model antes de migrar. *(2026-10-13: `\dt` sobre `stockmante` → solo `_sqlx_migrations`; sin conflictos.)*
- [X] T007 Crear migración `migrations/0001_usuarios_sesiones.sql`: tabla `usuarios` (campos y CHECK de data-model.md: email UNIQUE, `familia_rol CHECK IN ('GESTION','OPERACION')`, `activo`, `password_hash`, `password_temporal`) y `sesiones` (UUID PK, FK usuario, `expira_en`, `revocada_en`). **Aceptación**: `sqlx migrate run` aplica sin error; `sqlx describe usuarios` / `sesiones` coinciden con data-model.md.
- [X] T008 Crear migración `migrations/0002_categorias_productos.sql`: `categorias` (nombre UNIQUE), `productos` (SKU CHECK `^[A-Z0-9][A-Z0-9-]{2,29}$` + UNIQUE, `codigo_barras` UNIQUE NULLS NOT DISTINCT, FK categoría, costo/precio NUMERIC ≥0, stock_minimo ≥0, activo). **Aceptación**: `sqlx migrate run` OK; constraints visibles en `sqlx describe productos`.
- [X] T009 Crear migración `migrations/0003_depositos_ubicaciones.sql`: `depositos`, `ubicaciones` (FK deposito NOT NULL, FK padre, UNIQUE deposito+padre+nombre), `producto_ubicacion_permitida` (PK compuesta). **Aceptación**: `sqlx migrate run` OK; no se puede insertar ubicación sin deposito_id.
- [X] T010 [P] Implementar `backend/src/error.rs`: enum `AppError` con variantes de contracts (M1-E) + `IntoResponse` con shape `{ "error": { "codigo", "mensaje" } }`; Database/Interno → 500 genérico sin exponer `sqlx::Error`. **Aceptación**: unit test de mapeo de status/codigo para cada variante de dominio en verde.
- [X] T011 [P] Implementar `backend/src/config.rs` + `backend/src/db.rs`: carga de env, pool SQLx, constantes de sesión (cookie name, expiry horas). **Aceptación**: `cargo test` de config con env de test; pool construible con `DATABASE_URL` de test.
- [X] T012 Implementar helpers Argon2id en `backend/src/auth/password.rs`: `hash_password`, `verify_password`, `generar_password_temporal` (CSPRNG ≥12). **Aceptación**: tests roundtrip hash/verify; temporal cumple política (≥8, 1 mayúscula, 1 dígito); RED→GREEN.
- [X] T013 [P] Implementar `backend/src/mail/mod.rs` con `trait EnvioCorreo { async fn enviar_password_temporal(...) }` + impl `CorreoDev` (log/consola). **Aceptación**: unit test que el dev stub registra el intento sin red.
- [X] T014 [P] Implementar `backend/src/auth/reloj.rs` con `trait Reloj` + impl real (chrono) para expiración de sesiones testeable. **Aceptación**: test de inyección de reloj falso.
- [X] T015 [P] Crear `backend/src/main.rs` con router Axum raíz `/api`, CORS para `APP_URL`, state pool+config, y módulos vacíos de handlers. **Aceptación**: `cargo run` responde 404 JSON de error estándar en `GET /api/salud`; healthcheck simple `GET /api/salud` → 200.
- [X] T016 [P] Scaffolding frontend: `frontend/src/lib/api/client.ts` (fetch con `credentials: 'include'` y `ApiError`), `frontend/src/lib/schemas/`, layout raíz con Tailwind. **Aceptación**: `npm test` de un test unitario de `ApiError` parseando un body de error; build SPA OK.

**Checkpoint**: esquema completo del módulo en BD; AppError y primitivas listas; las 6 US pueden empezar.

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
- [X] T023 [US1] Seed/bootstrap: migración o binario `backend/src/bin/bootstrap.rs` que crea depósito "Principal" (si no existe) y admin inicial con password temporal impresa solo en consola DEV. **Aceptación**: en BD limpia, `sqlx migrate run` + bootstrap deja un admin activo; en prod no imprime secretos (flag/env).
- [X] T024 [P] [US1] Frontend: schema Zod `frontend/src/lib/schemas/auth.ts`, cliente `frontend/src/lib/api/auth.ts`, store `frontend/src/lib/stores/sesion.ts`. **Aceptación**: vitest de schemas + cliente mock 401/204.
- [X] T025 [US1] Frontend: ruta `frontend/src/routes/login/+page.svelte` y pantalla de cambio obligatorio de password; layout consulta sesión al arrancar y redirige. **Aceptación**: manual V1/V2 de quickstart.md; sin menú de admin si no hay sesión.

**Checkpoint**: MVP de autenticación usable end-to-end.

---

## Phase 4: User Story 2 — Gestión de usuarios (P1)

**Goal**: CRUD usuarios por familia GESTION, contraseña temporal por correo, desactivar/reactivar, guarda de último administrador.

**Independent Test**: Admin crea Operario → recibe temporal; email duplicado → 409; desactivar último admin → 409; desactivar con sesión → sesión cae; Operario → 403 en CRUD.

### Tests FIRST (RED)

- [ ] T026 [P] [US2] Unit test: alta con email duplicado → `EmailDuplicado` en `backend/tests/unit/test_usuarios_email.rs`. **Aceptación**: RED.
- [ ] T027 [P] [US2] Unit test: desactivar último `familia_rol=GESTION` activo → `UltimoAdministrador` en `backend/tests/unit/test_ultimo_admin.rs`. **Aceptación**: RED.
- [ ] T028 [P] [US2] Unit test: mapa rol→familia (ADMIN/GERENTE→GESTION; OPERARIO/CAJERO→OPERACION) en `backend/tests/unit/test_familia_rol.rs`. **Aceptación**: RED.

### Implementation para US2

- [ ] T029 [US2] Implementar servicio `backend/src/services/usuarios.rs`: crear (genera temp + `EnvioCorreo`), listar, obtener, actualizar, desactivar (con guarda último admin y revocación de sesiones), reactivar, regenerar password temporal. **Aceptación**: T026–T028 verdes; sin DELETE físico de usuarios.
- [ ] T030 [US2] Implementar handlers `backend/src/handlers/usuarios.rs` bajo extractor `RequiereGestion`: rutas de contracts/api.md (GET/POST/PUT usuarios, desactivar/reactivar/password-temporal). **Aceptación**: integration tests 201/409/422/403/404; sin `password_hash` en respuestas.
- [ ] T031 [P] [US2] Frontend: schemas + cliente `frontend/src/lib/api/usuarios.ts`, modal/tabla `frontend/src/lib/components/usuarios/*`, ruta `frontend/src/routes/admin/usuarios/+page.svelte`. **Aceptación**: vitest schemas; UI lista/crea/desactiva con mensajes `ApiError`.

**Checkpoint**: US1+US2 independientes; RBAC de gestión operativo en usuarios.

---

## Phase 5: User Story 3 — Catálogo de productos y categorías (P2)

**Goal**: CRUD productos con SKU normalizado, categorías, baja lógica si hay referencias, precios solo gestión.

**Independent Test**: Crear producto TEST-001 OK; SKU repetido → 409; SKU mal formado → 422; DELETE con asignación → 409; desactivar sale de caja.

### Tests FIRST (RED)

- [ ] T032 [P] [US3] Unit test: `normalizar_sku` + validación regex (`sku-001`→`SKU-001`; `bad` → Validacion) en `backend/tests/unit/test_sku.rs`. **Aceptación**: RED.
- [ ] T033 [P] [US3] Unit test: SKU duplicado y código de barras duplicado → 409 en `backend/tests/unit/test_producto_unicidad.rs`. **Aceptación**: RED.

### Implementation para US3

- [ ] T034 [US3] Implementar servicio `backend/src/services/productos.rs` + `backend/src/services/categorias.rs`: CRUD, normalización SKU, mapeo unique violation → `SkuDuplicado`/`CodigoBarrasDuplicado`, DELETE solo sin referencias else `ProductoConReferencias`, desactivar. **Aceptación**: T032–T033 verdes.
- [ ] T035 [US3] Implementar handlers `backend/src/handlers/productos.rs` y `categorias.rs`: escritura con `RequiereGestion`; lectura de detalle autenticada. **Aceptación**: integration tests contracts/api.md.
- [ ] T036 [P] [US3] Frontend: schemas producto/categoría, cliente API, rutas `admin/productos`, `admin/categorías`, formularios con Zod. **Aceptación**: vitest + UI alta/edición/baja.

**Checkpoint**: catálogo administrable.

---

## Phase 6: User Story 4 — Búsqueda rápida para caja (P2)

**Goal**: `GET /api/productos/buscar` optimizado (SKU/código exacto; nombre ≥2 chars; solo activos por defecto).

**Independent Test**: Con productos de prueba, búsqueda por SKU exacto y por fragmento de nombre; SC-002 en LAN ≤1s; inactivos no aparecen.

- [ ] T037 [US4] Migración/índice búsqueda: extensión `pg_trgm` + índice GIN en `LOWER(nombre)` (o LIKE prefix documentado) en `migrations/0004_busqueda_productos.sql`. **Aceptación**: `sqlx migrate run`; `EXPLAIN` muestra uso de índice en consulta de prueba.
- [ ] T038 [US4] Implementar servicio/handler `buscar_productos` en `backend/src/services/productos.rs` y handler: parámetros `q`, `campo`; validación min 2 en nombre; excluye inactivos salvo `incluir_inactivos=true` (gestión). **Aceptación**: integration tests por campo; unit de validación `q`.
- [ ] T039 [P] [US4] Frontend: ruta `frontend/src/routes/caja/+page.svelte` con input de búsqueda (SKU/código/nombre), debounce, lista de resultados con precio/stock mínimo visible. **Aceptación**: manual V6; vitest del cliente de búsqueda.

**Checkpoint**: caja usable con catálogo US3.

---

## Phase 7: User Story 5 — Depósitos y ubicaciones jerárquicas (P3)

**Goal**: CRUD depósitos/ubicaciones sin huérfanas; asignación de ubicaciones permitidas al producto.

**Independent Test**: Crear depósito→estantería→nivel; ubicación sin depósito → 422; PUT ubicaciones del producto reemplaza conjunto; DELETE depósito con hijos → 409.

### Tests FIRST (RED)

- [ ] T040 [P] [US5] Unit test: crear ubicación con deposito inexistente o distinto al padre → error en `backend/tests/unit/test_ubicaciones.rs`. **Aceptación**: RED.
- [ ] T041 [P] [US5] Unit test: DELETE depósito con dependencias → `DepositoConDependencias` en `backend/tests/unit/test_depositos.rs`. **Aceptación**: RED.

### Implementation para US5

- [ ] T042 [US5] Implementar servicio `backend/src/services/depositos.rs` y `ubicaciones.rs`: jerarquía, validación de depósito coherente con padre, reemplazo de `producto_ubicacion_permitida`. **Aceptación**: T040–T041 verdes.
- [ ] T043 [US5] Implementar handlers `/api/depositos`, `/api/ubicaciones`, `PUT /api/productos/{id}/ubicaciones` con `RequiereGestion`. **Aceptación**: integration tests contracts.
- [ ] T044 [P] [US5] Frontend: rutas `admin/depositos`, árbol/lista de ubicaciones, UI de asignación de ubicaciones a producto. **Aceptación**: manual V7; vitest cliente.

**Checkpoint**: estructura física del inventario configurada (sin movimientos de stock aún — fuera de alcance).

---

## Phase 8: User Story 6 — RBAC transversal y auditoría básica (P2)

**Goal**: Verificar matriz de permisos completa; auditoría de altas/bajas/cambios sensibles de usuarios y productos (autor + timestamp).

**Independent Test**: Batería de requests Operario vs Admin sobre todos los endpoints de escritura; SC-008; filas de auditoría creadas en desactivaciones y cambios de precio.

- [ ] T045 [US6] Implementar servicio mínimo de auditoría `backend/src/services/auditoria.rs` + migración `migrations/0005_auditoria.sql` (tabla simple: tabla_ref, registro_id, accion, usuario_id, cuando). Invocar en usuarios (desactivar/reactivar/update) y productos (update precio, desactivar). **Aceptación**: integration test verifica fila creada.
- [ ] T046 [US6] Integration test de matriz RBAC en `backend/tests/integration/test_rbac_matriz.rs`: cada endpoint de escritura con sesión OPERARIO → 403; con GESTION → no-403. **Aceptación**: test completo en verde.
- [ ] T047 [P] [US6] Frontend: layouts `admin` vs `caja` según `familia_rol`; ocultar rutas de gestión a operación (UX; backend ya deniega). **Aceptación**: manual con cuenta Operario no ve menú de gestión.

**Checkpoint**: seguridad transversal verificada.

---

## Phase 9: Polish & Cross-Cutting

**Purpose**: Docs, gates, validación quickstart, limpieza.

- [ ] T048 Escribir/actualizar `README.md` en la raíz según estructura de plan.md (setup local, env, comandos, sección Módulo 1). **Aceptación**: un newcomer puede seguir README sin leer la spec; comandos iguales a quickstart.md.
- [ ] T049 Ejecutar y dejar en verde: `cargo test`, `cargo sqlx prepare --check`, `npm test`; grep sin `unwrap`/`panic!`/`expect` en `backend/src` rutas de producción. **Aceptación**: comandos documentados en el commit message o PR.
- [ ] T050 Ejecutar escenarios V1–V8 de `specs/001-user-auth-catalog/quickstart.md` y registrar resultados en el PR. **Aceptación**: checklist V1–V8 tildada o notas de fallo justificadas.
- [ ] T051 [P] Revisar mensajes de error en español accionables (Principio III) en handlers; corregir genéricos indebidos fuera de login. **Aceptación**: revisión manual + tests de codigo de error.
- [ ] T052 Commit final de cierre del módulo y actualización de estado en spec si cambió comportamiento. **Aceptación**: `git log` convencional; spec/README al día.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: sin dependencias.
- **Foundational (Phase 2)**: depende de Phase 1 — **bloquea todas las US**.
- **US1 (Phase 3)**: solo Phase 2. **MVP**.
- **US2 (Phase 4)**: Phase 2 + extractores de sesión de US1 (T021) para probar desactivación con sesión; idealmente tras US1.
- **US3 (Phase 5)**: Phase 2 + extractores GESTION (parte de T021/base); independiente de US2 en datos.
- **US4 (Phase 6)**: US3 (necesita productos).
- **US5 (Phase 7)**: Phase 2 + auth; independiente de US3–US4 en datos (asignación referencia productos → mejor tras US3).
- **US6 (Phase 8)**: US1–US5 (matriz sobre endpoints ya existentes).
- **Polish (Phase 9)**: historias deseadas (MVP al menos US1+US2).

### User Story Dependencies

- **US1**: after Foundational — sin dependencias de otras US.
- **US2**: after Foundational + US1 (sesión/RBAC).
- **US3**: after Foundational + auth GESTION (US1).
- **US4**: after US3.
- **US5**: after Foundational; integración con productos tras US3.
- **US6**: after US1–US5.

### Within Each User Story

- Tests RED → servicios → handlers → frontend → commit convencional.
- `sqlx migrate run` + Habilidad 2 antes de cada query nueva.
- `cargo sqlx prepare --check` al cerrar cada tarea que toque SQL.

### Parallel Opportunities

- T003, T004, T005 (setup) en paralelo tras T001–T002.
- T010–T015 (AppError, config, mail, reloj) marcadamente [P].
- Tests unitarios de una misma US marcados [P].
- Tras Foundational, US3 y US5 pueden avanzar en paralelo con US2 si hay capacidad.

---

## Parallel Example: User Story 1

```text
# Juntos (sin tocar los mismos archivos):
Task T017: unit test auth login (backend/tests/unit/test_auth_login.rs)
Task T018: unit test sesión única (backend/tests/unit/test_sesion_unica.rs)
Task T019: unit test cambio password (backend/tests/unit/test_cambio_password.rs)

# Luego:
Task T020: servicio auth.rs (implementa los tests rojos)
```

---

## Implementation Strategy

### MVP First (US1 + mínimos de Foundational)

1. Phase 1 + Phase 2 completos.
2. Phase 3 (US1) completa.
3. **STOP**: validar V1/V2/V4 de quickstart (login, sesión única, mensaje genérico).
4. Demo: login + logout + cambio de password temporal.

### Incremental Delivery

1. Foundational → 2. US1 (MVP auth) → 3. US2 (gestión de personal) → 4. US3+US4 (catálogo+caja) → 5. US5 (depósitos) → 6. US6 (hardening RBAC) → 7. Polish (README + gates).

### Criterios de aceptación globales del módulo (spec SC)

- SC-001…SC-008 verificados en T050 (quickstart V1–V8).
- Quality gates constitución en T049.

---

## Notes

- Cada tarea cierra con al menos un commit convencional en la rama de feature.
- [P] implica archivos distintos; no paralelizar dos escrituras al mismo archivo.
- No implementar movimientos de stock, alertas ni chatbot en este módulo (fuera de alcance de spec).
- Si un test de la fase FOUNDATIONAL falla por esquema, corregir migración antes de avanzar de fase (Habilidad 2).
