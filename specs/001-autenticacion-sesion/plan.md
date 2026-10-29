# Implementation Plan: Autenticación y sesión segura (spec base)

**Spec**: [spec.md](./spec.md) | **Creado**: 2026-10-12 | **Dividido**: 2026-10-25

**Input**: Feature specification from `/specs/001-autenticacion-sesion/spec.md`

## Summary

Implementar la base del sistema y el caso de uso de autenticación: sesión en servidor con una
sesión vigente por usuario, RBAC de dos familias (gestión y operación), contraseña temporal
con cambio obligatorio y bootstrap del administrador inicial. Esta spec también posee la
**fase foundational** (esquema completo, `AppError`, pool, config, mailer, reloj) que el resto
de las specs consume.

Backend Rust/Axum/SQLx + frontend SvelteKit SPA/Tailwind, alineados a la constitución
(Principios I–V, verificación de esquema previa a SQL, `AppError` con mapeo 4xx de dominio y
500 genérico de infraestructura).

## Technical Context

**Language/Version**: Backend — Rust 1.99.0 estable (edición 2021; verificado con `rustc --version`). Frontend — **SvelteKit 3.0 + Svelte 5 + Tailwind CSS 4** (TypeScript 6 estricto dentro de SvelteKit; no es una app Node.js — Node 24 solo se usa como herramienta de build/dev del toolchain de SvelteKit).

**Primary Dependencies**: Backend - Axum 0.8, axum-extra 0.10 (cookies), SQLx 0.8 (**macros `query!`/`query_as!`/`query_scalar!` + `backend/.sqlx/` versionado**; C1 resuelto en T031), tokio, serde, argon2 0.5, tower-http 0.6 (CORS), `lettre` 0.11 (SMTP real async sobre rustls; incorporado en T029), `CorreoDev` sigue siendo el default de desarrollo. `validator` fue retirado en T031 (no se usaba). Frontend - **SvelteKit 3 (SPA adapter-static), Svelte 5, Tailwind CSS 4**, Zod 4.6, TypeScript 6 estricto, Vite 8, Vitest 4.

**Storage**: PostgreSQL 15+ (verificado en PostgreSQL 18 local); migraciones SQL versionadas en `/migrations` (raíz, según constitución).

**Testing**: `cargo test` (unit + integration; los targets con queries macro requieren `DATABASE_URL` al compilar), `cargo sqlx prepare --check` (gate real desde T031: 14 queries en `backend/.sqlx/`), build offline con `SQLX_OFFLINE=true`; frontend - `vitest` para `frontend/src/lib/api/*.ts` y stores; smoke E2E manual/Playwright fuera de alcance de v1 de este plan.

**Target Platform**: Servidor Linux/Windows con contenedor o proceso directo; navegadores desktop para caja/almacén.

**Project Type**: web-service (API REST JSON) + web-app (SPA)

**Performance Goals**: login y CRUD ≤ 2 s p95 en LAN (objetivo no bloqueante; se mide manualmente en V8 del quickstart).

**Constraints**: Sin `unwrap`/`panic`/`expect` en rutas de producción; errores de dominio → `AppError` específica 409/422 (o 401/403/404); infraestructura → `AppError::Interno` 500 genérico; una sesión por usuario (garantizada por índice único parcial; T027); cookie de sesión HttpOnly, SameSite=Lax y `Secure` cuando `APP_ENV=production` (T030); expiración absoluta de 8 h sin sliding (FR-001).

**Scale/Scope**: ~10–50 usuarios concurrentes; base compartida para los 6 casos de uso del Módulo 1.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Gate (Principio) | Estado | Evidencia en este plan |
|---|---|---|
| I — Simplicidad / YAGNI | ✅ PASS | Sin colas, sin Redis, sin OAuth/MFA, sin microservicios; sesiones en la misma BD; un monorepo de dos apps. |
| II — Mantenibilidad | ✅ PASS | Handlers Axum delgados; lógica en `services/`; DTOs y `AppError` separados; frontend con `lib/api/*.ts` tipado central. |
| III — Claridad en Mensajes | ✅ PASS (C2 resuelto en T032) | Mensajes accionables en español; login genérico por anti-enumeración, excepción acotada de P-III registrada en la constitución 1.2.0. |
| IV — Testabilidad Primero | ✅ PASS con waiver W-001 (C3) | Lógica sin I/O directa (traits de correo y reloj); TDD RED→GREEN en general, con W-001 registrado en la constitución para el RED no observado en T017–T019. |
| V — Integridad de Datos / Errores | ✅ PASS | Módulo no mueve stock; `AppError` con variante por dominio 409/422 y 500 genérico; unicidad FK/índices; verificación de esquema ANTES de cada tarea SQL. |
| Stack fijo — SQLx compile-time checked | ✅ PASS (C1 resuelto en T031) | Macros `query!`/`query_as!`/`query_scalar!` en el código de 001 (`services/auth.rs`, `bin/bootstrap.rs`) con `.sqlx/` versionado; `cargo sqlx prepare --check` verifica de verdad y el build offline (`SQLX_OFFLINE=true`) funciona. Las queries de 002/003 quedan para sus propias tareas. |

**Resultado post-diseño (Phase 1)**: PASS. C1 se resolvió en T031 con macros + `.sqlx` versionado;
C2 y C3 quedaron regularizados en T032 con la constitución 1.2.0 (excepción acotada de P-III y
waiver W-001). El diseño no viola I–V ni «Stack fijo». Complexity Tracking queda en «ninguna» (no
hay deuda de diseño).

### Desviaciones constitucionales registradas (2026-10-25)

| Hallazgo | Conflicto | Resolución | Tarea |
|---|---|---|---|
| C1 | «Stack fijo» exige SQLx compile-time checked queries; la implementación usa runtime API y `prepare --check` no verifica SQL. | **Resuelto (T031):** opción A — migrar a macros `query!`/`query_as!` + `.sqlx` versionado, sin enmienda constitucional. | T031 ✅ |
| C2 | P-III prohíbe mensajes genéricos; el login usa «Credenciales inválidas» por anti-enumeración. | **Resuelto (T032):** P-III enmendado en la constitución 1.2.0 con una excepción acotada a `POST /api/auth/login`, documentada en el contrato. | T032 ✅ |
| C3 | P-IV exige RED observado; T017–T019 se escribieron junto al servicio y el RED no se observó. | **Resuelto (T032):** waiver W-001 registrado en la constitución 1.2.0, limitado a T017–T019 de 001; desde 003 el RED es obligatorio. | T032 ✅ |

**Enmienda P-III aplicada (T032, constitución 1.2.0):** `POST /api/auth/login` PUEDE devolver un
mensaje y código HTTP únicos para email inexistente, contraseña incorrecta y cuenta inactiva; la
excepción se limita a ese endpoint, debe estar documentada en la spec y no habilita mensajes
genéricos en ningún otro endpoint.

**Decisión adoptada (2026-10-25, T031):** opción A — `query!`/`query_as!`/`query_scalar!` con
`backend/.sqlx/` versionado. Sin enmienda para C1; la propuesta de runtime API con gate equivalente
queda descartada. Las queries de 002 (`services/usuarios.rs`) y 003 (`services/productos.rs`,
`services/categorias.rs`) se migraron el mismo día como cierre del follow-up: `.sqlx` regenerado,
`cargo sqlx prepare --check` verde y build offline sin BD.

## Project Structure

### Documentation

```text
specs/
├── README.md                    # mapa de specs, partición de FR/SC y ledger de migraciones
├── 001-autenticacion-sesion/    # esta spec (base + US1)
├── 002-gestion-usuarios/        # US2
├── 003-catalogo-productos/      # US3
├── 004-busqueda-caja/           # US4
├── 005-depositos-ubicaciones/   # US5
└── 006-rbac-auditoria/          # US6
```

### Source Code (repository root)

```text
backend/
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── config.rs
│   ├── error.rs         # AppError + IntoResponse
│   ├── db.rs            # pool
│   ├── extractores.rs   # JsonValido / PathValido / QueryValida
│   ├── models/          # entidades y DTOs
│   ├── services/        # lógica de negocio (auth, usuarios, productos, depositos)
│   ├── handlers/        # routers Axum
│   ├── auth/            # sesión, extractors, RBAC
│   └── mail/            # TraitEnvioCorreo + impls
└── tests/
    ├── integration/
    └── unit/

frontend/
├── package.json
├── src/
│   ├── lib/{api,schemas,stores,components,utils}/
│   └── routes/{login,caja,admin}/
└── ...

migrations/              # SQLx migrations en RAÍZ (constitución)
.env.example
README.md                # entregable del cierre transversal (C1)
```

**Structure Decision**: monorepo web (`backend/` + `frontend/` + `/migrations` en raíz). La
constitución manda `/migrations` en la raíz del repo. SvelteKit 3 usa el alias `#lib` (no
`$lib`); el ejemplo de la constitución P-II quedó obsoleto (L4/T032).

## Fase foundational (propiedad de esta spec)

El esquema completo del Módulo 1 se creó antes de las user stories para no bloquearlas:

| Migración | Tablas | Spec dueña |
|---|---|---|
| `0001_usuarios_sesiones.sql` | `usuarios`, `sesiones` | 001 |
| `0002_categorias_productos.sql` | `categorias`, `productos` | 003 |
| `0003_depositos_ubicaciones.sql` | `depositos`, `ubicaciones`, `producto_ubicacion_permitida` | 005 |

Ver el ledger completo en [specs/README.md](../README.md).

## Arquitectura (resumen)

- **Auth**: `POST /api/auth/login`, `POST /api/auth/logout`, `POST /api/auth/cambiar-password`, `GET /api/auth/sesion`.
- **Salud**: `GET /api/salud` → 200 `{ "ok": true }`; endpoint operativo sin datos de negocio (contrato en `contracts/api.md`; T015).
- **Guards**: `AuthUsuario` (sesión vigente), `RequiereGestion` (familia GESTION → 403 `sin_permisos`) y `RequierePasswordCambiada` (temporal pendiente → 403 `cambio_password_requerido`). El gate de temporal es global a todo endpoint autenticado salvo `GET /auth/sesion` y `POST /auth/cambiar-password`, independientemente de la familia (T028).
- **Sesión**: tabla `sesiones`, cookie HttpOnly + SameSite=Lax; el login revoca la sesión anterior en la misma transacción.
- **UI**: layout consulta `/auth/sesion` al arrancar, redirige a `/login` y exige el cambio de contraseña temporal.
- **Otras specs**: usuarios (002), catálogo (003), búsqueda (004), depósitos (005), RBAC/auditoría (006).

## Estrategia de pruebas

| Capa | Enfoque |
|---|---|
| Dominio (Rust) | Unit tests puros de login, sesión única y cambio de contraseña — la política exige RED→GREEN; en T017–T019 el RED no se observó (waiver W-001 de la constitución 1.2.0) |
| Handlers | Integration tests con BD de test; login/logout/sesión/cambio + 401/403/422 |
| RBAC | Extractores y bloqueo por contraseña temporal |
| Frontend | Vitest: Zod de auth, cliente y store de sesión |
| SQLx | `cargo sqlx prepare --check` sin diferencias (gate real desde T031; `.sqlx` en `backend/`) y build offline con `SQLX_OFFLINE=true` |
| E2E manual | V1, V2 y V4 del [quickstart](./quickstart.md) |

## Complexity Tracking

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| _(ninguna)_ | — | Constitution Check PASS sin justificación de deudas. |
