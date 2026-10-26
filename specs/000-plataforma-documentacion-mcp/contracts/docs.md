# Contrato — documentación y MCP (000)

## Rutas HTTP

| Método | Ruta | Auth | Éxito | Deshabilitado |
|---|---|---|---|---|
| GET | `/api-docs/openapi.json` | pública | 200 `application/json` (OpenAPI 3.1) | 404 |
| GET | `/docs` | pública | 302 → `/docs/` | 404 |
| GET | `/docs/` | pública | 200 `text/html` (Swagger UI) | 404 |

- `DOCS_ENABLED` ausente: `true` si `APP_ENV` ≠ `production`; `false` si `APP_ENV=production`.
- `DOCS_ENABLED=true|false` (sin distinguir mayúsculas ni espacios) gana sobre el default.
  Cualquier otro valor → error de configuración al arrancar (`validar()`).
- El documento declara `securitySchemes.cookieAuth` (`apiKey`, `in: cookie`,
  `name: stockmante_sesion`) y lo aplica por defecto; login, logout y salud declaran
  `security: []`.

## Esquema de error

Todas las respuestas 4xx/5xx del Módulo 1 conservan:

```json
{ "error": { "codigo": "string", "mensaje": "string" } }
```

## Configuración MCP

`opencode.json` (raíz):

```json
{
  "$schema": "https://opencode.ai/config.json",
  "mcp": {
    "servers": {
      "postgres": {
        "type": "local",
        "command": ["npx", "-y", "@bytebase/dbhub@latest", "--config", "dbhub.toml"],
        "enabled": true
      }
    }
  }
}
```

`dbhub.toml` (raíz):

```toml
[[sources]]
db_type = "postgres"
dsn = "${DATABASE_URL}"
readonly = true
max_rows = 200
```

- Sin credenciales en el repositorio: el DSN sale de `.env`/entorno.
- El servidor expone `execute_sql` en modo lectura; las escrituras se rechazan.
- `context7` ya está configurado globalmente; no se duplica en el proyecto.
