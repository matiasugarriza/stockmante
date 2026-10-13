# Implementation Plan: Módulo 1 — Usuarios, Autenticación y Catálogo Base

**Branch**: `001-user-auth-catalog` | **Date**: 2026-10-12 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/001-user-auth-catalog/spec.md`

## Summary

Implementar el módulo fundacional de Stockmante: autenticación por sesión en servidor con una sesión vigente por usuario, RBAC de dos familias (gestión y operación), CRUD de usuarios con baja lógica y contraseñas temporales por correo, catálogo de productos/categorías con SKU normalizado y búsqueda optimizada para caja, y depósitos con ubicaciones jerárquicas más asignación producto–ubicación. Backend Rust/Axum/SQLx + frontend SvelteKit SPA/Tailwind, alineados a la constitución v1.1.1 (Principios I–V, verificación de esquema previa a SQL, `AppError` con mapeo 4xx dominio / 500 genérico infraestructura).

## Technical Context

**Language/Version**: Backend — Rust 1.75+ (estable). Frontend — **SvelteKit (modo SPA) + Tailwind CSS** (TypeScript dentro de SvelteKit; no es una app Node.js — Node solo se usa como herramienta de build/dev del toolchain de SvelteKit si hace falta)

**Primary Dependencies**: Backend — Axum 0.7+, SQLx 0.8 (compile-time `query!`/`query_as!`), tokio, serde, validator, argon2, tower-http (cookies/CORS), lettre (SMTP, opcional tras env). Frontend — **SvelteKit (SPA adapter), Tailwind CSS**, Zod, TypeScript estricto.

**Storage**: PostgreSQL 15+; migraciones SQL versionadas en `/migrations` (raíz, según constitución).

**Testing**: `cargo test` (unit + integration con BD de test), `cargo sqlx prepare --check`; frontend — `vitest` para `$lib/api/*.ts` y stores; smoke E2E manual/Playwright fuera de alcance de v1 de este plan.

**Target Platform**: Servidor Linux/Windows con contenedor o proceso directo; navegadores desktop para caja/almacén.

**Project Type**: web-service (API REST JSON) + web-app (SPA)

**Performance Goals**: SC-002 — 95% de búsquedas por SKU/código de barras ≤ 1s; login y CRUD ≤ 2s p95 en LAN.

**Constraints**: Sin `unwrap`/`panic`/`expect` en rutas de producción; errores de dominio → `AppError` específica 409/422 (o 401/403/404); infraestructura → `AppError::Interno` 500 genérico; una sesión por usuario; cookie de sesión HttpOnly.

**Scale/Scope**: ~10–50 usuarios concurrentes, ~10k productos, ~5 depósitos, ~2k ubicaciones; 30–40 endpoints; ~6–8 pantallas UI del módulo.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Gate (Principio) | Estado | Evidencia en este plan |
|---|---|---|
| I — Simplicidad / YAGNI | ✅ PASS | Sin colas, sin Redis, sin OAuth/MFA, sin microservicios; sesiones en la misma BD; un monorepo de dos apps. |
| II — Mantenibilidad | ✅ PASS | Handlers Axum delgados; lógica en `services/`; DTOs y `AppError` separados; frontend con `$lib/api/*.ts` tipado central. |
| III — Claridad en Mensajes | ✅ PASS | Mensajes de error accionables en español de negocio; JSON de API estable sin adornos; login genérico documentado como excepción (spec Supuestos). |
| IV — Testabilidad Primero | ✅ PASS | Lógica de negocio sin I/O directa (traits para correo y reloj); TDD RED→GREEN en reglas de dominio; tests de `$lib/api` y stores; `cargo sqlx prepare --check`. |
| V — Integridad de Datos / Errores | ✅ PASS | Módulo no mueve stock (transacciones explícitas se aplican al 100% en módulos de movimientos); `AppError` con variante por dominio 409/422 y 500 genérico; unicidad FK/índices; verificación de esquema ANTES de cada tarea SQL (Habilidad 2 / MCP). |

**Resultado post-diseño (Phase 1)**: PASS — sin violaciones; no se llena Complexity Tracking.

## Project Structure

### Documentation (this feature)

```text
specs/001-user-auth-catalog/
├── plan.md              # Este archivo
├── research.md          # Phase 0 — decisiones técnicas
├── data-model.md        # Phase 1 — modelo de datos
├── quickstart.md        # Phase 1 — validación end-to-end
├── contracts/
│   ├── api.md           # Contratos HTTP (endpoints, payloads, errores)
│   └── validation.md    # Esquemas de validación backend/frontend
└── tasks.md             # Phase 2 (/speckit.tasks — aún no creado)
```

### Source Code (repository root)

```text
backend/
├── Cargo.toml
├── migrations/          # symlink o copia controlada → ver nota /migrations
├── src/
│   ├── main.rs
│   ├── config.rs
│   ├── error.rs         # AppError + IntoResponse
│   ├── db.rs            # pool
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
├── svelte.config.js     # adapter-static (SPA)
├── src/
│   ├── lib/
│   │   ├── api/         # clientes tipados (auth, usuarios, productos, depositos)
│   │   ├── schemas/     # Zod
│   │   ├── stores/      # sesión, catálogo
│   │   └── components/  # forms, tablas, modales, layouts
│   ├── routes/
│   │   ├── login/
│   │   ├── caja/        # búsqueda productos
│   │   ├── admin/       # usuarios, productos, depositos, ubicaciones
│   │   └── +layout.svelte
│   └── tests/
└── ...

migrations/              # SQLx migrations en RAÍZ (constitución)
└── 0001_*.sql …

.env.example             # DATABASE_URL, SESSION_*, EMAIL_*, APP_URL
README.md                # ENTREGABLE del Módulo 1: setup local + documentación del módulo (ver sección Configuración inicial y README)
```

**Structure Decision**: monorepo web (`backend/` + `frontend/` + `/migrations` en raíz). El plan no crea código en esta fase; esta estructura es la que los tasks de implementación deben seguir. La constitución manda `/migrations` en la raíz del repo; SQLx apunta `migrations_dir` allí (o se usa `sqlx migrate` con `-m migrations`).

## Modelado de datos (resumen — detalle en data-model.md)

Entidades: `usuarios`, `sesiones`, `categorias`, `productos`, `depositos`, `ubicaciones`, `producto_ubicacion_permitida`. Unicidades: `usuarios.email`, `productos.sku`, `productos.codigo_barras` (NULLS NOT DISTINCT), índices de búsqueda por nombre/SK. Baja lógica vía `activo BOOLEAN` + `eliminado_en TIMESTAMPTZ NULL` cuando aplique.

## Arquitectura (resumen — detalle en contracts/)

- **Auth**: `POST /api/auth/login`, `POST /api/auth/logout`, `POST /api/auth/cambiar-password`, `GET /api/auth/sesion`.
- **Usuarios** (gestión): CRUD `/api/usuarios`, `POST /api/usuarios/{id}/desactivar|reactivar|password-temporal`.
- **Catálogo**: CRUD `/api/productos`, `GET /api/productos/buscar`, CRUD `/api/categorias`.
- **Depósitos**: CRUD `/api/depositos`, CRUD `/api/ubicaciones`, `PUT /api/productos/{id}/ubicaciones`.
- **Guards**: extractor `AuthUsuario` + `RequiereGestion`; middleware de expiración.
- **UI**: layouts admin/caja; tablas paginadas; modales de alta/edición; formularios Zod.

## Estrategia de pruebas (resumen — detalle en contracts/validation.md)

| Capa | Enfoque |
|---|---|
| Dominio (Rust) | Unit tests puros: SKU, unicidad, último admin, jerarquía ubicaciones — RED→GREEN |
| Handlers | Integration tests con `sqlx::test` o BD Docker; happy path + 409/422/401/403 |
| Auth/RBAC | Tests de sesión única, desactivación con sesión viva, denegación Operario→gestión |
| Frontend | Vitest: Zod schemas, cliente `$lib/api`, stores de sesión |
| SQLx | `cargo sqlx prepare --check` sin diferencias en cada cierre de tarea |
| E2E manual | quickstart.md — login, alta usuario, búsqueda SKU, ubicación huérfana |

## Configuración inicial y README (entregable del Módulo 1)

### Creación de `README.md` en la raíz del repo

Dentro de la implementación del Módulo 1 se **crea (o se reescribe) `README.md`** en la raíz del monorepo. No es documentación opcional: es parte del alcance de este módulo y debe cerrarse junto a la última tarea de backend/frontend, alineada a `quickstart.md` (fuente de comandos y variables).

**Estructura requerida del README:**

```markdown
# Stockmante

Descripción breve del sistema (gestión de inventario).

## Stack
- Frontend: SvelteKit (SPA) + Tailwind CSS
- Backend: Rust + Axum + SQLx
- Base de datos: PostgreSQL

## Requisitos del entorno local
- Rust (rustup)
- PostgreSQL 15+
- sqlx-cli
- Toolchain de SvelteKit para el frontend

## Configuración del entorno local
1. Clonar el repo
2. Copiar .env.example → .env y explicar cada variable
   (DATABASE_URL, SESSION_COOKIE_NAME, SESSION_EXPIRY_HOURS,
    APP_URL, EMAIL_FROM, SMTP_URL)
3. sqlx migrate run
4. Arrancar backend (cargo run) — nota sobre admin bootstrap
5. Arrancar frontend SvelteKit (comando dev del proyecto)

## Comandos útiles
| Comando | Para qué |
|---------|----------|
| sqlx migrate run | Aplicar migraciones |
| cargo test | Tests backend |
| cargo sqlx prepare --check | Verificar caché de queries SQLx |
| (dev del frontend) | Levantar SPA SvelteKit |
| (test del frontend) | Tests vitest de $lib/api y stores |

## Módulo 1 — Usuarios, Autenticación y Catálogo Base
- Qué incluye: login/sesión (una por usuario), RBAC (gestión vs operación),
  usuarios con baja lógica y contraseña temporal, productos/categorías con
  SKU normalizado y búsqueda para caja, depósitos y ubicaciones jerárquicas.
- Roles: Administrador/Gerente (idénticos en este módulo) y Operario/Cajero.
- Seguridad de login: mensaje genérico de fallo (anti-enumeración).
- Fuera de alcance de v1: movimientos de stock, transferencias, facturación,
  alertas de stock mínimo.
- Docs de diseño de este módulo: specs/001-user-auth-catalog/
  (spec, plan, data-model, contracts, quickstart).

## Calidad
Sin unwrap/panic en producción; tests en verde; spec actualizada si cambia
el comportamiento.
```

**Reglas del README:**
- Idioma: español (equipo de proyecto).
- Sin pasos inventados: los comandos deben coincidir con `quickstart.md`.
- Las credenciales del admin bootstrap solo se explican ("se imprimen en consola
  de dev"), nunca se pegan claves reales en el README.
- Si el comportamiento del módulo cambia respecto de lo planificado, actualizar
  README en el mismo PR (quality gate de constitución: docs junto al cambio).

### Fuente de verdad de comandos y variables

[quickstart.md](./quickstart.md) mantiene la guía de validación detallada
(escenarios V1–V8). El README es la puerta de entrada resumida; en caso de
conflicto, prevalece quickstart.md y se corrige el README en el mismo commit.

## Complexity Tracking

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| _(ninguna)_ | — | Constitution Check PASS sin justificación de deudas. |

## Siguiente paso

`/speckit.tasks` para descomponer en tareas TDD con unidades de trabajo (commits) por área: migraciones+esquema → error/AppError → auth+sesiones → usuarios → productos/categorías → depósitos/ubicaciones → frontend → **README.md raíz (setup local + doc Módulo 1)**.
