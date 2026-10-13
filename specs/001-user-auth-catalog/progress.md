# Progreso de implementación — Módulo 1 (espejo local de reanudación)

> Engram quedó temporalmente no disponible al cerrar esta sesión
> (`could not resolve an authoritative OpenCode runtime session`).
> Este archivo es el espejo local para retomar; sincronizar con Engram al iniciar la próxima sesión.

## Último estado verificado

- **Commits push**: `e2f2705` (phase1), `4adbab7` (phase2), `59bb4b1` (frontend client).
- **tasks.md**: T001–T016 marcadas `[X]` (Phase 1 Setup + Phase 2 Foundational).
- **Tests**: backend 8/8 (`cargo test`), frontend 4/4 (`npm test`).
- **BD**: migraciones 0001–0003 aplicadas en `stockmante` y `stockmante_test` (PG18 local, locale es-AR); esquema verificado con `\d`.
- **Smoke**: `GET /api/salud` → 200.

## Siguiente paso

Phase 3 — US1 Autenticación (T017–T025):

1. Tests RED: `backend/tests/unit/test_auth_login.rs`, `test_sesion_unica.rs`, `test_cambio_password.rs`.
2. `backend/src/services/auth.rs` (login/logout/sesion/cambiar-password, sesión única, expiración con `Reloj`).
3. `backend/src/auth/extractors.rs` (`AuthUsuario`, `RequiereGestion`).
4. `backend/src/handlers/auth.rs` (`POST /api/auth/login|logout|cambiar-password`, `GET /api/auth/sesion`, cookie HttpOnly SameSite=Lax).
5. `backend/src/bin/bootstrap.rs` (admin inicial + depósito "Principal"; password temporal solo consola dev).
6. Frontend: `lib/schemas/auth.ts`, `lib/api/auth.ts`, `lib/stores/sesion.ts`, `routes/login`.

Luego: US2 (T026–T031) → US3+US4 → US5 → US6 → Polish (README T048, quickstart T050, gates T049).

## Notas de entorno

- Node 24.20 / Rust 1.99 / sqlx-cli 0.9 / PG18.
- `cargo test` requiere `[[test]]` entries para `tests/unit/*`.
- `sqlx-cli 0.9` no trae `describe`; usar psql.
- Passwords de BD: solo las conoce el usuario (`.env` local, gitignored).
