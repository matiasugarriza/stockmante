# Research — Documentación de API y MCP (000)

**Fecha**: 2026-10-26

## Decisión 1: generador de OpenAPI

| Opción | Veredicto |
|---|---|
| `utoipa` + `utoipa-swagger-ui` | **Elegida** |
| `aide` | Buena ergonomía, pero exige reescribir el router con su `ApiRouter`; más invasivo. |
| `paperclip` / `apistos` | Madurez y adopción menores; otro wrapper de router. |
| OpenAPI escrito a mano | Se desincroniza del código; el usuario pidió documentación mantenible. |

**Elección**: `utoipa` 6 (code-first, anotaciones `#[utoipa::path]` y `ToSchema`) +
`utoipa-swagger-ui` 10 con features `axum` y `vendored`. `vendored` evita descargar Swagger UI en
cada build (el entorno de CI puede no tener red).

**Compatibilidad verificada**: `rustc 1.99.0`, `axum 0.8`, `utoipa 6.0`, `utoipa-swagger-ui 10.0.2`
(requiere Rust ≥ 1.88). `SwaggerUi` implementa `From<SwaggerUi> for Router<S>`, así que se puede
`merge` dentro de `construir_router` antes de `with_state`.

**Tipos especiales**:
- `rust_decimal::Decimal` → `#[schema(value_type = f64)]` (el JSON ya lo serializa como número).
- `chrono::DateTime<Utc>` → feature `chrono` de `utoipa`.

## Decisión 2: ubicación y encendido de la documentación

- `GET /docs` (Swagger UI) y `GET /api-docs/openapi.json`.
- Flag `DOCS_ENABLED`: por defecto `true` fuera de producción y `false` con `APP_ENV=production`.
  Un valor explícito (`true`/`false`) gana. Esto evita publicar el contrato interno en producción
  por accidente, sin bloquear el desarrollo.
- Las rutas se montan fuera del grupo autenticado; no requieren sesión.

## Decisión 3: MCP de PostgreSQL

| Opción | Veredicto |
|---|---|
| `@bytebase/dbhub` | **Elegida**: un solo binario, `readonly = true`, `max_rows`, soporta `.env`. |
| `@modelcontextprotocol/server-postgres` | Archivado/deprecado; sin modo readonly explícito. |
| `postgres-mcp` (Python) | Requiere Python y otra cadena de instalación. |
| MCP custom en Rust | Trabajo de mantenimiento sin valor para el Módulo 1. |

Configuración (`dbhub.toml`):

```toml
[[sources]]
db_type = "postgres"
dsn = "${DATABASE_URL}"
readonly = true
max_rows = 200
```

DBHub carga `.env` desde el directorio de trabajo; el DSN nunca se versiona.

## Decisión 4: MCP de filesystem / GitHub / inventario

- **Filesystem**: el agente ya tiene herramientas de lectura/edición del workspace; un MCP extra
  duplicaría permisos. Se documenta como opcional.
- **GitHub**: `gh` CLI ya cubre PRs e issues; el MCP remoto de GitHub requiere token y permisos.
  Se documenta como opcional.
- **Inventario custom**: el chatbot y los movimientos de stock están fuera del Módulo 1 (ver
  `specs/README.md`); construir un MCP de inventario ahora viola YAGNI. Se documenta como trabajo
  futuro, sin implementarlo.

## Decisión 5: documentación escrita

- `docs/api.md`: mapa de endpoints, códigos, auth y enlace a Swagger.
- `docs/arquitectura.md`: capas backend, SPA, migraciones y decisiones.
- `docs/mcp.md`: cómo habilitar DBHub, ejemplos de consultas y límites.
- `README.md` raíz: setup, comandos, mapa de specs y enlaces.
