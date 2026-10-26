# Implementation Plan: Gestión de usuarios (retro-documentado)

**Spec**: [spec.md](./spec.md) | **Creado**: 2026-10-25 | **Estado**: Implementado

**Input**: Feature specification from `/specs/002-gestion-usuarios/spec.md`

## Summary

CRUD de usuarios para la familia GESTION con baja lógica, contraseña temporal por correo,
guarda del último gestor activo y paginación. Retro-documenta la implementación ya realizada
bajo `f1fc3e8` y sus follow-ups, para que el caso de uso US2 tenga spec, plan y tareas propias.

## Technical Context

**Language/Version**: Rust 1.99 (backend), TypeScript 6 / SvelteKit 3 (frontend).

**Primary Dependencies**: Axum 0.7+, SQLx 0.8 (macros `query!`/`query_as!`/`query_scalar!`,
compile-time checked), tokio, serde, argon2, tower-http, Zod 4, Tailwind CSS.

**Storage**: PostgreSQL 18; tablas `usuarios` y `sesiones` (propiedad de 001).

**Acceso a datos**: consultas SQLx con macros compile-time checked (Principio IV y «Stack fijo»
de la constitución) y `backend/.sqlx` versionado para builds offline (`SQLX_OFFLINE=true`).

**Testing**: `cargo test` (unit + integration con `stockmante_test`), vitest, `cargo sqlx
prepare --check`, `npm run check`, `npm run build`.

**Constraints**: sin `unwrap`/`panic`/`expect` en rutas de producción; `AppError` con 409/422/403/404;
RBAC por `RequiereGestion`; sin `DELETE` físico de usuarios.

**Scale/Scope**: 10–50 usuarios; paginación por defecto 20, máximo 100.

## Constitution Check

| Gate (Principio) | Estado | Evidencia |
|---|---|---|
| I — Simplicidad / YAGNI | ✅ PASS | Sin tablas extra; baja lógica con `activo`; sin autoservicio de recuperación. |
| II — Mantenibilidad | ✅ PASS | Lógica en `services/usuarios.rs`; handlers delgados; DTOs y `AppError` separados. |
| III — Claridad en Mensajes | ✅ PASS | 409 `email_duplicado`, 409 `ultimo_administrador`, 422 `validacion` con mensajes en español. |
| IV — Testabilidad Primero | ✅ PASS | Tests unitarios de reglas + integración HTTP; RED observado antes de la implementación. |
| V — Integridad de Datos / Errores | ✅ PASS | Unicidad por índice + mapeo `23505`; guarda del último gestor con locks ordenados; sin `password_hash` en respuestas. |

**Resultado post-diseño**: PASS — sin violaciones.

## Project Structure

```text
backend/
├── src/services/usuarios.rs        # familia_de_rol, listar, obtener, crear, actualizar,
│                                   # desactivar, reactivar, regenerar_password_temporal
├── src/handlers/usuarios.rs        # router /api/usuarios (RequiereGestion)
├── src/models/usuario.rs           # Usuario público, DTOs, normalizar_email
└── tests/
    ├── unit/{test_usuarios_email,test_usuarios_alta,test_ultimo_admin,test_familia_rol}.rs
    └── integration/test_usuarios_http.rs

frontend/src/
├── lib/schemas/usuarios.ts
├── lib/api/usuarios.ts
├── lib/components/usuarios/{UsuarioForm,UsuarioTabla}.svelte
├── lib/utils/paginacion.ts
└── routes/admin/usuarios/+page.svelte
```

## Arquitectura implementada

- **`familia_de_rol`**: única fuente del mapa `ADMIN|GERENTE → GESTION`,
  `OPERARIO|CAJERO → OPERACION`; `actualizar` la recalcula al cambiar el rol.
- **`crear`**: genera temporal y envía por `EnvioCorreo` **dentro de la transacción**; si el
  envío falla no queda cuenta huérfana. Devuelve 201 con el usuario público (sin hash ni
  contraseña).
- **`listar`**: paginado (`pagina >= 1`, `1 <= por_pagina <= 100`), filtro
  `activo=true|false|all`, offset con `checked_mul` (extremos → 422, sin overflow).
- **`desactivar`**: baja lógica + revocación de sesiones; `bloquear_gestores_activos` toma
  `FOR UPDATE` sobre los gestores activos **antes** de la fila objetivo (mismo orden en
  `actualizar` → sin deadlock `40P01`); si es el último → `UltimoAdministrador` (409).
  La misma invariante se aplica al degradar el rol.
- **Unicidad de email**: sin pre-chequeo; violación `23505` de `usuarios_email_key` →
  `EmailDuplicado` (409). Email normalizado con `models::usuario::normalizar_email`.
- **`validar_email`**: exactamente una `@`, partes no vacías, sin whitespace, sin puntos
  consecutivos ni en bordes. Rechaza `a@b@c.com`, `a@@b.com`, `a@b..com`, `.a@b.com`, etc.
- **Handlers**: router anidado en `/api/usuarios`; todas las rutas con `RequiereGestion`;
  `DebeCambiarPassword` (403) mientras la temporal siga vigente.
- **Frontend**: schemas Zod, cliente tipado, tabla + modal; errores dentro del modal; link
  "Usuarios" visible solo para GESTION. Paginación de 20 con reinicio a página 1 al cambiar
  filtro y recorte a la última página válida.

## Estrategia de pruebas

| Capa | Enfoque |
|---|---|
| Dominio | Unicidad de email, último gestor, mapa rol→familia, alta con correo |
| Handlers | 201/200/204, 409 de unicidad y último gestor, 422, 403 de OPERACION, 401 sin sesión, 404 |
| Invariantes | 7 rutas de gestión rechazadas con temporal sin persistir cambios |
| Frontend | Zod, cliente API, paginación |
| SQLx | `cargo sqlx prepare --check` sin diferencias |

## Commits de referencia

| Commit | Contenido |
|---|---|
| `f1fc3e8` | US2 completa: servicio, handlers, frontend y tests |
| `977bd90` | FR-008a (`DebeCambiarPassword`), sobre de error de extractores, validación de email endurecida |
| `8fa8390` | Paginación del listado de usuarios |
| `2878b4b` | Regresiones de alto riesgo de los pendientes |
| `67032c7` | Migración de `services/usuarios.rs` a macros SQLx y regeneración de `backend/.sqlx` (2026-10-25) |

## Complexity Tracking

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| _(ninguna)_ | — | Constitution Check PASS sin deudas. |
