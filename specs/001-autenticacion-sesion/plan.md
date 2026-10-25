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

**Language/Version**: Backend — Rust 1.75+ (estable). Frontend — **SvelteKit (modo SPA) + Tailwind CSS** (TypeScript dentro de SvelteKit; no es una app Node.js — Node solo se usa como herramienta de build/dev del toolchain de SvelteKit si hace falta)

**Primary Dependencies**: Backend — Axum 0.7+, SQLx 0.8, tokio, serde, validator, argon2, tower-http (cookies/CORS), lettre (SMTP, opcional tras env). Frontend — **SvelteKit (SPA adapter), Tailwind CSS**, Zod, TypeScript estricto.

**Storage**: PostgreSQL 15+; migraciones SQL versionadas en `/migrations` (raíz, según constitución).

**Testing**: `cargo test` (unit + integration con BD de test), `cargo sqlx prepare --check`; frontend — `vitest` para `lib/api/*.ts` y stores; smoke E2E manual/Playwright fuera de alcance de v1 de este plan.

**Target Platform**: Servidor Linux/Windows con contenedor o proceso directo; navegadores desktop para caja/almacén.

**Project Type**: web-service (API REST JSON) + web-app (SPA)

**Performance Goals**: login y CRUD ≤ 2 s p95 en LAN.

**Constraints**: Sin `unwrap`/`panic`/`expect` en rutas de producción; errores de dominio → `AppError` específica 409/422 (o 401/403/404); infraestructura → `AppError::Interno` 500 genérico; una sesión por usuario; cookie de sesión HttpOnly.

**Scale/Scope**: ~10–50 usuarios concurrentes; base compartida para los 6 casos de uso del Módulo 1.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Gate (Principio) | Estado | Evidencia en este plan |
|---|---|---|
| I — Simplicidad / YAGNI | ✅ PASS | Sin colas, sin Redis, sin OAuth/MFA, sin microservicios; sesiones en la misma BD; un monorepo de dos apps. |
| II — Mantenibilidad | ✅ PASS | Handlers Axum delgados; lógica en `services/`; DTOs y `AppError` separados; frontend con `lib/api/*.ts` tipado central. |
| III — Claridad en Mensajes | ✅ PASS | Mensajes de error accionables en español de negocio; JSON de API estable sin adornos; login genérico documentado como excepción (spec Supuestos). |
| IV — Testabilidad Primero | ✅ PASS | Lógica de negocio sin I/O directa (traits para correo y reloj); TDD RED→GREEN en reglas de dominio; tests de `lib/api` y stores; `cargo sqlx prepare --check`. |
| V — Integridad de Datos / Errores | ✅ PASS | Módulo no mueve stock; `AppError` con variante por dominio 409/422 y 500 genérico; unicidad FK/índices; verificación de esquema ANTES de cada tarea SQL. |

**Resultado post-diseño (Phase 1)**: PASS — sin violaciones; no se llena Complexity Tracking.

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
constitución manda `/migrations` en la raíz del repo.

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
- **Guards**: extractor `AuthUsuario` + `RequiereGestion`; la familia OPERACION recibe 403 y una contraseña temporal pendiente recibe 403 `cambio_password_requerido`.
- **Sesión**: tabla `sesiones`, cookie HttpOnly + SameSite=Lax; el login revoca la sesión anterior en la misma transacción.
- **UI**: layout consulta `/auth/sesion` al arrancar, redirige a `/login` y exige el cambio de contraseña temporal.
- **Otras specs**: usuarios (002), catálogo (003), búsqueda (004), depósitos (005), RBAC/auditoría (006).

## Estrategia de pruebas

| Capa | Enfoque |
|---|---|
| Dominio (Rust) | Unit tests puros de login, sesión única y cambio de contraseña — RED→GREEN |
| Handlers | Integration tests con BD de test; login/logout/sesión/cambio + 401/403/422 |
| RBAC | Extractores y bloqueo por contraseña temporal |
| Frontend | Vitest: Zod de auth, cliente y store de sesión |
| SQLx | `cargo sqlx prepare --check` sin diferencias |
| E2E manual | V1, V2 y V4 del [quickstart](./quickstart.md) |

## Complexity Tracking

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| _(ninguna)_ | — | Constitution Check PASS sin justificación de deudas. |
