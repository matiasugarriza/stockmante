# Progreso de implementación — Módulo 1 (espejo local de reanudación)

> Engram quedó temporalmente no disponible al cerrar la sesión anterior
> (`could not resolve an authoritative OpenCode runtime session`).
> Este archivo es el espejo local para retomar; sincronizar con Engram al iniciar la próxima sesión.

## Último estado verificado (2026-10-20)

- **tasks.md**: T001–T025 `[X]` (Phase 1 Setup + Phase 2 Foundational + **Phase 3 US1 completa**).
- **Tests backend**: 24/24 (`cargo test`) — 5 integration HTTP + 19 unit.
  - `tests/unit/test_auth_login.rs`, `test_sesion_unica.rs`, `test_cambio_password.rs` (T017–T019)
  - `tests/integration/test_auth_http.rs` (T021–T022)
- **Tests frontend**: 19/19 (`npm test`) + `npm run check` 0 errores + `npm run build` OK.
- **Gates**: `cargo sqlx prepare --check` OK (sin macros `query!`; queries en runtime API);
  grep sin `unwrap`/`panic!`/`expect` en `backend/src` ✅.
- **BD**: migraciones 0001–0003 en `stockmante` y `stockmante_test` (PG18 local, es-AR).
  Bootstrap `cargo run --bin bootstrap` idempotente: crea depósito "Principal" +
  `admin@stockmante.local` con password temporal impresa solo en DEV (`APP_ENV`).
- **Commits previos**: `e2f2705` (phase1), `4adbab7` (phase2), `59bb4b1` (frontend client).

## Arquitectura US1 (implementado)

- `backend/src/app.rs`: `AppEstado` + `construir_router(estado)` (compartido por main y tests).
- `backend/src/services/auth.rs`: `login` (transacción: revoca anterior + inserta nueva),
  `logout` (idempotente), `sesion_actual` (NoAutenticado / SesionExpirada vía `Reloj`),
  `cambiar_password` (política + hash viejo inservible). Email normalizado a minúsculas.
- `backend/src/auth/extractors.rs`: `AuthUsuario`, `RequiereGestion` (listo para US2).
- `backend/src/handlers/auth.rs`: las 4 rutas de `contracts/api.md`; cookie
  HttpOnly + SameSite=Lax + `max_age` (usa crate `time`, porque `cookie 0.18` no acepta
  `SystemTime` en `expires`).
- `backend/src/bin/bootstrap.rs`: admin + depósito; credenciales solo en consola DEV.
- Frontend: `lib/schemas/auth.ts` (Zod 4 — `z.email()`), `lib/api/auth.ts`,
  `lib/stores/sesion.ts`, `routes/login/+page.svelte` (login + cambio obligatorio),
  layout con redirect a `/login`.

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

## Siguiente paso

Phase 4 — US2 Gestión de usuarios (T026–T031):

1. Tests RED: `test_usuarios_email.rs`, `test_ultimo_admin.rs`, `test_familia_rol.rs`.
2. `services/usuarios.rs`: crear (temp + `EnvioCorreo`), listar, obtener, actualizar,
   desactivar (guarda último admin + revocar sesiones), reactivar, regenerar temporal.
3. `handlers/usuarios.rs` bajo `RequiereGestion` (extractor ya existe); 201/409/422/403/404.
4. Frontend: `api/usuarios.ts`, componentes `components/usuarios/*`, `routes/admin/usuarios`.

Luego: US3 → US4 → US5 → US6 → Polish (README T048, gates T049, V1–V8 T050).

## Notas de entorno

- Node 24.20 / Rust 1.99 / sqlx-cli 0.9 / PG18 / Zod 4.6.7 / SvelteKit 3.0.1 / TS 6.0.3.
- `cargo test` requiere `[[test]]` entries para `tests/unit/*` e `tests/integration/*`.
- Tests de BD usan `DATABASE_URL_TEST` o derivan `…/stockmante_test` de `DATABASE_URL`
  (`backend/tests/unit/common/mod.rs`); emails únicos por test para paralelismo.
- Passwords de BD: solo las conoce el usuario (`.env` local, gitignored).
