# Progreso de implementación — Documentación de API y MCP (000)

**Spec**: [spec.md](./spec.md) | **Plan**: [plan.md](./plan.md) | **Actualizado**: 2026-10-26 |
**Estado**: completa y verificada; incluida en el commit único de cierre del Módulo 1 (2026-10-26),
como pidió el usuario (todo el proyecto en un solo commit).

## Estado por fase

| Fase | Tareas | Estado | Evidencia |
|---|---|---|---|
| 1. Setup | T001–T005 | Completa | `utoipa` 6 / `utoipa-swagger-ui` 10; `docs_enabled`; `test_config` 5/5; `test_openapi` 6/6 |
| 2. Anotaciones | T006–T010 | Completa | auth, usuarios, categorías, productos y salud anotados |
| 3. MCP | T011–T012 | Completa | DBHub 1.5.0 contra `stockmante`: lectura OK, escritura rechazada |
| 4. Documentación | T013–T015 | Completa | `README.md`, `docs/api.md`, `docs/arquitectura.md`, `docs/mcp.md`, `.env.example` |
| 5. Cierre | T016–T017 | Completa | gates abajo; revisión del JSON |

## Evidencia observada (2026-10-26)

### Documentación viva

- `DOCS_ENABLED` ausente en desarrollo → encendido; `APP_ENV=production` → apagado;
  `true`/`false`/`1`/`0` explícitos mandan; valor inválido → `422 validacion`.
- `docs.rs` expone `/docs` y `/api-docs/openapi.json` solo si el flag está encendido.
- `test_openapi` verifica rutas, `cookieAuth`, seguridad pública de login/logout y los schemas
  `Producto`, `Categoria`, `Usuario`, `ErrorRespuesta`.

### MCP (DBHub 1.5.0)

Comando: `npx -y @bytebase/dbhub@latest --config dbhub.toml` con `DATABASE_URL` del `.env`.

| Paso | Resultado observado |
|---|---|
| `initialize` | `DBHub MCP Server 1.5.0` |
| `tools/list` | `execute_sql` |
| Lectura | `SELECT count(*)::int AS productos FROM productos` → `{"productos":4}` |
| Escritura | `CREATE TABLE mcp_no_debe_poder (id int)` → `READONLY_VIOLATION` |

`readonly` y `max_rows` viven en `[[tools]]` (DBHub no los acepta por fuente); el DSN se interpola
desde `DATABASE_URL` y no se versiona.

### Gates de calidad

| Comando | Resultado |
|---|---|
| `cd backend && cargo test` | **90 tests, 0 fallos** (exit 0) |
| `cd backend && cargo sqlx prepare --check` | exit 0; aviso preexistente `potentially unused queries found in .sqlx` |
| `cd frontend && npm test` | **11 archivos, 104 tests, 0 fallos** |
| `cd frontend && npm run check` | 0 errores, 0 warnings |
| `cd frontend && npm run build` | OK |

### Revisión de exposición

El JSON se genera solo desde anotaciones `utoipa`; no incluye `DATABASE_URL`, credenciales,
rutas absolutas del filesystem ni variables de entorno. La cookie se documenta como esquema de
seguridad, sin valores reales. Swagger queda apagado en producción salvo elección explícita.

## Próximo paso

Spec 004 (búsqueda y caja), luego 005 y 006. El commit de todo el proyecto queda para el final.
