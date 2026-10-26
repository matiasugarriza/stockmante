# Servidores MCP

Los MCP (Model Context Protocol) dan herramientas a los agentes que trabajan en este repositorio.
La configuración de proyecto vive en `opencode.json` y se fusiona con la global del usuario.

## PostgreSQL (DBHub) — incluido

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
db_type = "postgres"
dsn = "${DATABASE_URL}"
readonly = true
max_rows = 200
```

**Garantías**

- `readonly = true`: las sentencias de escritura y DDL se rechazan.
- `max_rows = 200`: los `SELECT` devuelven como máximo 200 filas.
- El DSN nunca se versiona; `dbhub.toml` solo referencia `${DATABASE_URL}`.

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

**Prueba manual**

```bash
npx -y @bytebase/dbhub@latest --config dbhub.toml
```

El servidor queda escuchando por stdio; OpenCode lo levanta solo al iniciar la sesión.

## Context7 — ya disponible globalmente

El entorno del usuario ya expone Context7 (documentación de librerías). **No se duplica** en
`opencode.json` para evitar dos servidores con el mismo nombre. Si se necesita en otra máquina,
agregar el remoto `https://mcp.context7.com/mcp` en la configuración personal.

## Filesystem / GitHub — opcionales

- **Filesystem**: el agente ya tiene lectura y edición del workspace; un MCP extra duplicaría
  permisos y ampliaría la superficie de riesgo. No se incluye.
- **GitHub**: `gh` CLI cubre PRs, issues y checks. Si se quiere el MCP oficial, requiere un token
  con permisos acotados y debe configurarse a nivel personal, no en el repositorio.

## MCP de inventario — trabajo futuro

El chatbot y los movimientos de stock están **fuera del alcance del Módulo 1**
(ver `specs/README.md`). Un MCP de inventario tendría que exponer herramientas como
`buscar_producto`, `consultar_stock` y `registrar_movimiento`, con autenticación y autorización
propias. Construirlo ahora violaría YAGNI y duplicaría reglas que todavía no existen. Se reevalúa
cuando se implemente el módulo de stock.
