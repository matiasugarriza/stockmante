# Plan de implementación — Documentación de API y MCP (000)

**Spec**: [spec.md](./spec.md) | **Research**: [research.md](./research.md) | **Creado**: 2026-10-26 | **Última actualización**: 2026-10-31

## Enfoque

1. **Dependencias**: `utoipa` 6 (`chrono`) y `utoipa-swagger-ui` 10 (`axum`, `vendored`).
2. **Configuración**: `AppConfig.docs_enabled` con default dependiente de `APP_ENV`; parseo tolerante (`true`/`false`, sin distinguir mayúsculas) y error de validación si el valor es otro. Se permite sobreescritura explícita mediante `.env.local`.
3. **Documento**: `backend/src/docs.rs` con `ApiDoc` (`#[derive(OpenApi)]`), esquema de seguridad `cookieAuth` (ApiKey en cookie) y `ErrorRespuesta`.
4. **Anotaciones**: `#[utoipa::path]` en cada handler existente y `ToSchema`/`IntoParams` en DTOs y modelos. Los handlers públicos declaran `security(())`.
5. **Router**: `construir_router` hace `merge(docs::router())` solo si `docs_enabled`.
6. **MCP**:
   - `opencode.json` con `mcp.servers.postgres` (DBHub, `@bytebase/dbhub@latest`) y `mcp.servers.playwright` (`@playwright/mcp`).
   - `dbhub.toml` readonly (`readonly = true`, `max_rows = 100`); el DSN se lee de `${DATABASE_URL}` y nunca se versiona.
7. **Documentación**: `docs/api.md`, `docs/arquitectura.md`, `docs/mcp.md`, `README.md` raíz.
8. **Playwright MCP**: servidor para validación visual y pruebas E2E autónomas sobre la app frontend servida por Vite en `http://localhost:5173`.

## Estructura de archivos

```text
backend/
  Cargo.toml                     + utoipa, utoipa-swagger-ui
  src/config.rs                  + docs_enabled
  src/docs.rs                    NUEVO: ApiDoc, cookieAuth, ErrorRespuesta, router()
  src/app.rs                     merge condicional de docs
  src/handlers/*.rs              anotaciones utoipa
  src/models/*.rs                derives ToSchema
  tests/unit/test_config.rs      + casos de DOCS_ENABLED
  tests/unit/test_openapi.rs     NUEVO: contrato del JSON
opencode.json                    NUEVO: MCP de proyecto (postgres + playwright)
dbhub.toml                       NUEVO: fuente PostgreSQL readonly
docs/api.md                      NUEVO
docs/arquitectura.md             NUEVO
docs/mcp.md                      NUEVO (incluye Playwright)
README.md                        reemplazado (estaba vacío)
.env.example                     + DOCS_ENABLED
```

## Decisiones de diseño

- **Sin duplicar contratos**: Swagger es la vista navegable; `specs/*/contracts/api.md` sigue siendo la fuente de decisiones y mensajes de negocio.
- **El JSON se genera en runtime** (`ApiDoc::openapi()`), no se versiona un archivo estático.
- **Errores**: se documenta el sobre `{ error: { codigo, mensaje } }` una vez (`ErrorRespuesta`) y se referencia en las respuestas 4xx/5xx relevantes.
- **Seguridad**: `cookieAuth` describe la cookie `stockmante_sesion`; el valor real es configurable, pero el nombre por defecto es el documentado.
- **Producción**: `DOCS_ENABLED` default `false` con `APP_ENV=production`; habilitarlo es una decisión explícita.
- **Playwright MCP**: paquete `@playwright/mcp` (no `@modelcontextprotocol/server-playwright`, que no existe en el registro npm). Navega a `http://localhost:5173` y toma capturas de pantalla para validar el renderizado de componentes Svelte.

## Fases

1. **Setup**: dependencias, config, `docs.rs`, router, test de configuración/OpenAPI.
2. **Anotaciones**: auth, usuarios, categorías, productos (schemas + paths).
3. **MCP**: `opencode.json` (postgres + playwright), `dbhub.toml`, verificación de conexión readonly.
4. **Documentación**: `docs/*.md`, `README.md`, `.env.example`.
5. **Cierre**: gates backend/frontend, `cargo sqlx prepare --check`, revisión de que el JSON no exponga datos sensibles, verificación de Playwright MCP.

## Riesgos y mitigaciones

| Riesgo | Mitigación |
|---|---|
| `utoipa` no entiende `Decimal` | `#[schema(value_type = f64)]` explícito. |
| El build de Swagger UI requiere red | feature `vendored`. |
| Las anotaciones se desincronizan | `test_openapi` verifica rutas y schemas clave; el JSON se genera del código. |
| El MCP permite escrituras | `readonly = true` + `max_rows`; el DSN apunta a la base local. |
| El JSON expone el nombre de la cookie | Es información ya visible en el navegador; no expone el valor ni secretos. |
| Paquete de Playwright MCP inexistente | Usar `@playwright/mcp` (verificado en npm), no `@modelcontextprotocol/server-playwright`. |
| Playwright necesita frontend corriendo | El agente debe levantar `cd frontend && npm run dev` antes de usar el MCP. |

## Consistencia (analyze)

- FR-DOC-01/02/03/04 cubiertos por `docs.rs`, `app.rs`, `config.rs` y `test_openapi.rs`.
- FR-MCP-01/02 cubiertos por `opencode.json`, `dbhub.toml` y `docs/mcp.md`.
- FR-MCP-03 cubierto por `opencode.json` con servidor `playwright` (`@playwright/mcp`).
- FR-MCP-04 cubierto por `docs/mcp.md` (sección Playwright + por qué el MCP de inventario custom queda fuera de alcance).
- CA-01/02/03/04/05/06 mapeados a tareas y gates de cierre.
- No hay entidades nuevas ni migraciones: `data-model.md` no aplica.
- Los contratos de negocio de 001–003 no cambian; solo se documentan.
