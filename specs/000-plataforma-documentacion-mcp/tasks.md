# Tasks — Documentación de API y MCP (000)

**Spec**: [spec.md](./spec.md) | **Plan**: [plan.md](./plan.md) | **Creado**: 2026-10-26

**Tests**: sí. TDD para `docs_enabled` y para el contrato del JSON (`test_openapi`). La
documentación escrita se verifica por inspección.

## Fase 1 — Setup

- [X] T001 Agregar `utoipa = { version = "6", features = ["chrono"] }` y
  `utoipa-swagger-ui = { version = "10", features = ["axum", "vendored"] }` a `backend/Cargo.toml`.
  **Aceptación**: `cargo check` compila.
- [X] T002 RED: extender `backend/tests/unit/test_config.rs` con los casos de `DOCS_ENABLED`
  (ausente en development → `true`; `APP_ENV=production` → `false`; `true`/`false` explícitos;
  valor inválido → `Validacion`). **Aceptación**: falla antes de implementar.
- [X] T003 Implementar `docs_enabled` en `backend/src/config.rs` y validarlo. **Aceptación**: T002
  en verde.
- [X] T004 Crear `backend/src/docs.rs` con `ApiDoc`, `cookieAuth`, `ErrorRespuesta` y `router()`
  condicional; registrar `pub mod docs;` en `backend/src/lib.rs` y hacer `merge` en `app.rs`.
  **Aceptación**: `cargo check` compila; rutas 404 con el flag apagado.
- [X] T005 RED/GREEN: crear `backend/tests/unit/test_openapi.rs` y su `[[test]]`; verificar rutas
  clave, `cookieAuth` y schemas `Producto`/`Categoria`/`Usuario`. **Aceptación**: verde.

## Fase 2 — Anotaciones

- [X] T006 Anotar auth (`login`, `logout`, `sesion`, `cambiar-password`) con `ToSchema` en DTOs y
  `#[utoipa::path]`; login/logout con `security(())`.
- [X] T007 Anotar usuarios (5 rutas + DTOs + `ListaUsuarios` + `Usuario`).
- [X] T008 Anotar categorías (5 rutas + DTOs + `Categoria`).
- [X] T009 Anotar productos (6 rutas + DTOs + `Producto`/`ListaProductos`).
- [X] T010 Anotar salud. **Aceptación**: el JSON lista todas las rutas `/api/*` existentes.

## Fase 3 — MCP

- [X] T011 Crear `dbhub.toml` readonly y `opencode.json` con `mcp.servers.postgres`.
- [X] T012 Verificar `npx -y @bytebase/dbhub@latest --config dbhub.toml` contra `stockmante`:
  `execute_sql` de lectura funciona y una escritura se rechaza. Registrar evidencia.

## Fase 4 — Documentación

- [X] T013 Crear `docs/api.md`, `docs/arquitectura.md`, `docs/mcp.md`.
- [X] T014 Reemplazar `README.md` (setup local, comandos, mapa de specs, docs y MCP).
- [X] T015 Agregar `DOCS_ENABLED` documentado a `.env.example`.

## Fase 5 — Cierre

- [X] T016 Ejecutar `cargo test`, `cargo sqlx prepare --check`, `npm test`, `npm run check` y
  `npm run build`.
- [X] T017 Revisar que el JSON no exponga secretos ni rutas internas; registrar en
  `progress.md`.
