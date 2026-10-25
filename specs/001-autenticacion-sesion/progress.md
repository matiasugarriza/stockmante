# Progreso de implementación — Base técnica y autenticación (001)

**Spec**: [spec.md](./spec.md) | **Actualizado**: 2026-10-25 | **Estado**: Implementado y cerrado

> Espejo local de reanudación. El Módulo 1 se dividió por caso de uso el 2026-10-25: este archivo
> cubre solo la **base compartida** (Phase 1–2) y **US1**. US2 vive en
> [002/progress.md](../002-gestion-usuarios/progress.md) y US3 en
> [003/progress.md](../003-catalogo-productos/progress.md).

## Último estado verificado (2026-10-25)

- **tasks.md**: T001–T025 `[X]` (Phase 1 + Phase 2 + Phase 3 US1).
- **Tests backend**: los targets de base y US1 en verde; al cierre de US2 la suite completa
  era 45/45 (16 integration HTTP + 29 unit).
  - Base/US1: `test_auth_login.rs`, `test_sesion_unica.rs`, `test_cambio_password.rs`,
    `test_password.rs`, `test_apperror.rs`, `test_mail.rs` (2), `test_config.rs` (1),
    `test_auth_http.rs` (6 HTTP).
- **Tests frontend**: al cierre de US2, 49/49 (`npm test`) + `npm run check` 0 errores/0
  warnings + `npm run build` OK. De esa suite, esta spec posee los tests de auth y el
  cliente HTTP base; los de usuarios pertenecen a 002.
- **Gates**: `cargo sqlx prepare --check` OK (sin macros `query!`; queries en runtime API);
  grep sin `unwrap`/`panic!`/`expect` en `backend/src` ✅.
- **BD**: migraciones 0001–0004 aplicadas en `stockmante` y `stockmante_test` (PG18 local).
  La 0002 y 0003 se crearon acá como fase foundational; la 0004 es de 003.
  Bootstrap `cargo run --bin bootstrap` idempotente: crea depósito "Principal" +
  `admin@stockmante.local` con password temporal impresa solo en DEV (`APP_ENV`).
- **Push**: `6e1d722..d87a04f main -> main` (11 commits). `main` sincronizada con `origin/main`.

## Arquitectura US1 (implementado)

- `backend/src/app.rs`: `AppEstado` + `construir_router(estado)` (compartido por main y tests).
- `backend/src/services/auth.rs`: `login` (transacción: revoca anterior + inserta nueva),
  `logout` (idempotente), `sesion_actual` (NoAutenticado / SesionExpirada vía `Reloj`),
  `cambiar_password` (política + hash viejo inservible). Email normalizado a minúsculas.
- `backend/src/auth/extractors.rs`: `AuthUsuario`, `RequiereGestion`.
- `backend/src/handlers/auth.rs`: las 4 rutas de `contracts/api.md`; cookie
  HttpOnly + SameSite=Lax + `max_age` (usa crate `time`, porque `cookie 0.18` no acepta
  `SystemTime` en `expires`).
- `backend/src/bin/bootstrap.rs`: admin + depósito; credenciales solo en consola DEV.
- Frontend: `lib/schemas/auth.ts` (Zod 4 — `z.email()`), `lib/api/auth.ts`,
  `lib/stores/sesion.ts`, `routes/login/+page.svelte` (login + cambio obligatorio),
  layout con redirect a `/login`.

## Fase foundational

| Migración | Tablas | Spec dueña del esquema |
|---|---|---|
| `0001_usuarios_sesiones.sql` | `usuarios`, `sesiones` | 001 |
| `0002_categorias_productos.sql` | `categorias`, `productos` | 003 |
| `0003_depositos_ubicaciones.sql` | `depositos`, `ubicaciones`, `producto_ubicacion_permitida` | 005 |

La fase se planificó y ejecutó en 001 para no bloquear las user stories; el ledger completo
está en [specs/README.md](../README.md).

## Gotchas de entorno (importante)

- **SvelteKit 3 eliminó `$lib`**: el alias ahora es **`#lib`** (subpath imports de
  `package.json`). Vite los resuelve extensionless, pero **TS no sondea extensiones
  para `imports`** → se agregó `compilerOptions.paths` en `frontend/tsconfig.json`
  (`#lib/* → ./src/lib/*`). No reintroducir `$lib` (kit lo rechaza con error).
- PowerShell `-replace` con `"\$lib/"` interpola `$lib` como vacío → corrompe archivos.
  Usar comillas simples o el edit tool.
- `psql` no está en PATH (instalación EDB); usar `sqlx` o la ruta completa de EDB.
- `cargo sqlx prepare --check` pasa sin `.sqlx/` porque no se usan macros `query!`
  (runtime API de sqlx con SQL literal).
- `cargo fmt --check` ya fallaba en archivos previos a US2 (`bootstrap.rs`, `app.rs`,
  `config.rs`, `handlers/auth.rs`, `mail/envio.rs`, `main.rs` y tests previos): no
  correr `cargo fmt` global (formatea fuera de alcance). Los archivos nuevos se
  formatearon con `rustfmt` puntual.
- Node 24.20 / Rust 1.99 / sqlx-cli 0.9 / PG18 / Zod 4.6.7 / SvelteKit 3.0.1 / TS 6.0.3.
- `cargo test` requiere `[[test]]` entries para `tests/unit/*` e `tests/integration/*`.
- Tests de BD usan `DATABASE_URL_TEST` o derivan `…/stockmante_test` de `DATABASE_URL`
  (`backend/tests/unit/common/mod.rs`); emails únicos por test para paralelismo.

## Siguiente paso

001 no tiene trabajo pendiente. El Módulo 1 continúa en
[003-catalogo-productos](../003-catalogo-productos/progress.md) (US3). Después: 004 (búsqueda),
005 (depósitos) y 006 (RBAC/auditoría); el cierre transversal C1–C5 está en
[specs/README.md](../README.md).

## Pendientes administrativos

- `.atl/` está en `.gitignore` (`979315a`); el plugin lo regenera localmente.
- No hay commits locales sin push.
