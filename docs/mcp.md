# Servidores MCP

Los **Model Context Protocol (MCP)** dan herramientas a los agentes que trabajan en este
repositorio. La configuración de proyecto vive en [`opencode.json`](../opencode.json) y se fusiona
con la global del usuario.

## Resumen

| Servidor | Origen | Transporte | Rol | Estado |
|---|---|---|---|---|
| `postgres` (DBHub) | **Externo** (`@bytebase/dbhub`) | stdio local | Verificar el esquema real antes de escribir SQL/migraciones; explorar datos de prueba; solo lectura | Configurado y verificado |
| `playwright` (`@playwright/mcp`) | **Externo** | stdio local | Validación visual/E2E y capturas sobre el frontend en Vite | Configurado y verificado |
| Context7 | Externo remoto | HTTP | Documentación de librerías | Disponible globalmente; opcional |

El mínimo del TP —**2 servidores MCP, al menos 1 externo**— se cumple con DBHub y Playwright:
ambos son paquetes de terceros y no fueron desarrollados por el equipo.

## PostgreSQL (DBHub)

**Para qué**: verificar el esquema real antes de escribir SQL o structs de SQLx (regla de
`AGENTS.md`), explorar datos de prueba y diagnosticar migraciones, sin riesgo de escritura.

**Requisitos**

- `DATABASE_URL` definida en `.env` (DBHub carga el `.env` del directorio de trabajo).
- `npx` disponible (Node.js 20+).

**Configuración** (`opencode.json`):

```json
{
  "mcp": {
    "servers": {
      "postgres": {
        "type": "local",
        "command": ["npx", "-y", "@bytebase/dbhub@latest", "--config", "dbhub.toml"]
      }
    }
  }
}
```

`dbhub.toml`:

```toml
[[sources]]
id = "stockmante"
db_type = "postgres"
dsn = "${DATABASE_URL}"

[[tools]]
name = "execute_sql"
source = "stockmante"
readonly = true
max_rows = 200
```

**Garantías**

- `readonly = true` en la herramienta `execute_sql`: las sentencias de escritura y DDL se
  rechazan.
- `max_rows = 200`: los `SELECT` devuelven como máximo 200 filas.
- El DSN nunca se versiona; `dbhub.toml` solo referencia `${DATABASE_URL}`.

**Evidencia observada** (spec 000, 2026-10-26):

| Paso | Resultado |
|---|---|
| `initialize` | `DBHub MCP Server 1.5.0` |
| `tools/list` | `execute_sql` |
| Lectura | `SELECT count(*) FROM productos` → valor real de la base |
| Escritura | `CREATE TABLE mcp_no_debe_poder (id int)` → `READONLY_VIOLATION` |

**Prueba manual**

```bash
npx -y @bytebase/dbhub@latest --config dbhub.toml
```

El servidor queda escuchando por stdio; el agente lo levanta al iniciar la sesión.

**Ejemplos de uso**

```sql
SELECT table_name FROM information_schema.tables
 WHERE table_schema = 'public' ORDER BY table_name;
```

```sql
SELECT conname, pg_get_constraintdef(oid)
  FROM pg_constraint
 WHERE conrelid = 'productos'::regclass;
```

## Playwright MCP

**Para qué**: validación visual y pruebas E2E autónomas sobre el frontend servido por Vite
(`http://localhost:5173`), incluidas capturas para la evidencia de las specs 018/019.

**Requisitos**

- Node.js 20+ y npx.
- Frontend corriendo: `cd frontend && npm run dev`.

**Configuración** (`opencode.json`):

```json
"playwright": {
  "type": "local",
  "command": ["npx", "-y", "@playwright/mcp"]
}
```

**Evidencia**: el servidor inicia sin errores y permite navegar, inspeccionar y capturar la app
en `http://localhost:5173` (spec 000, CA-04). Las capturas resultantes están en
[`specs/018-rediseno-visual-desktop/evidencia/`](../specs/018-rediseno-visual-desktop/evidencia/)
y [`specs/019-rediseno-visual-mobile/`](../specs/019-rediseno-visual-mobile/).

## Context7 — opcional, remoto

El entorno del usuario ya expone Context7 (documentación de librerías). **No se duplica** en
`opencode.json` para evitar dos servidores con el mismo nombre. Si se necesita en otra máquina,
agregar el remoto `https://mcp.context7.com/mcp` en la configuración personal:

```json
"context7": {
  "type": "remote",
  "url": "https://mcp.context7.com/mcp"
}
```

No es necesario para el mínimo de 2 servidores y no se usó como dependencia del código.

## Filesystem / GitHub — no incluidos

- **Filesystem**: el agente ya tiene lectura y edición del workspace; un MCP extra duplicaría
  permisos y ampliaría la superficie de riesgo.
- **GitHub**: `gh` CLI cubre PRs, issues y checks. El MCP oficial requiere un token con permisos
  acotados y se configura a nivel personal, no en el repositorio.

## MCP de inventario custom — no construido (decisión)

El asistente **no** usa un MCP de inventario. Sus tools de LangChain leen la API Rust con la
cookie del usuario y las escrituras se proponen y confirman por `POST /api/movimientos`. Construir
un MCP de escritura habría duplicado permisos, transacciones y reglas de stock, y habría violado
la regla de que el chatbot nunca escribe directo en la base.

Los 2 MCP usados (DBHub y Playwright) alcanzan el mínimo de 2 servidores, 1 externo. La
arquitectura del asistente se documenta en
[`docs/arquitectura.md`](./arquitectura.md#asistente) y
[`asistente/README.md`](../asistente/README.md).
