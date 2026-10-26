# Especificación de Funcionalidad: Documentación de API y servidores MCP

**Spec**: `000-plataforma-documentacion-mcp` | **Tipo**: plataforma (transversal) | **Prioridad**: P0

**Creado**: 2026-10-26 | **Estado**: Implementado (ver [progress.md](./progress.md))

**Entrada**: pedido del usuario (2026-10-26): antes de seguir con 004–006, documentar la API del
backend ("por ejemplo, swagger, pero la que mejor vaya con el proyecto") y agregar servidores MCP
al proyecto.

**Depende de**: 001–003 implementadas (superficie HTTP existente).

> No es un caso de uso de negocio: es la base de documentación y de herramientas para desarrollar
> 004–006. No agrega entidades ni cambia contratos de negocio.

## Problema y resultado esperado

Hoy la API solo está descrita en `specs/*/contracts/api.md`; no hay una referencia navegable ni un
contrato legible por máquina. Además, el proyecto no declara servidores MCP, así que cada sesión de
desarrollo depende de memoria y de `psql` manual.

**Resultado esperado**

1. OpenAPI 3.1 generado desde el código, visible en Swagger UI, con autenticación por cookie,
   esquemas de request/response y el sobre de error del Módulo 1.
2. Documentación escrita de la API, la arquitectura y los MCP disponibles.
3. Configuración MCP versionada y utilizable por OpenCode, con PostgreSQL en modo **solo lectura**
   para validar esquema antes de escribir SQL.

## Requisitos

- **FR-DOC-01**: el backend DEBE exponer `GET /api-docs/openapi.json` con un documento OpenAPI 3.1
  que incluya todas las rutas `/api/*` existentes, sus parámetros, cuerpos, respuestas y el esquema
  de seguridad por cookie de sesión.
- **FR-DOC-02**: el backend DEBE servir Swagger UI en `GET /docs` apuntando a ese JSON.
- **FR-DOC-03**: las rutas de documentación DEBEN poder desactivarse con `DOCS_ENABLED=false`; en
  `APP_ENV=production` el valor por defecto es `false` y fuera de producción es `true`.
- **FR-DOC-04**: las rutas de documentación son públicas (no exigen sesión) y no exponen secretos ni
  datos de configuración (solo el nombre de la cookie, que ya es público en el navegador).
- **FR-MCP-01**: el repositorio DEBE incluir una configuración MCP de proyecto con un servidor
  PostgreSQL en modo solo lectura (DBHub) y documentar cómo levantarlo.
- **FR-MCP-02**: la configuración NO DEBE contener credenciales; el DSN se toma de
  `DATABASE_URL` del entorno.
- **FR-MCP-03**: la documentación DEBE recomendar un MCP de filesystem/GitHub como opcionales y
  explicar por qué el MCP de inventario custom queda fuera de alcance del Módulo 1.

## Criterios de aceptación

- **CA-01**: `cargo test --test test_openapi` en verde: el JSON contiene las rutas del Módulo 1, el
  esquema `cookieAuth` y los schemas de `Producto`, `Categoria` y `Usuario`.
- **CA-02**: con `DOCS_ENABLED=true`, `GET /api-docs/openapi.json` responde 200 y `GET /docs`
  redirige/entrega el HTML de Swagger UI; con `DOCS_ENABLED=false` ambas responden 404.
- **CA-03**: `npx -y @bytebase/dbhub@latest --config dbhub.toml` conecta contra `stockmante` y
  rechaza escrituras.
- **CA-04**: `docs/api.md`, `docs/arquitectura.md` y `docs/mcp.md` existen y enlazan desde
  `README.md`.

## Fuera de alcance

- Autenticación de Swagger UI más allá de la cookie existente (no hay API keys).
- Publicación del JSON en un portal externo o generación de clientes.
- MCP de inventario custom y MCP de GitHub obligatorio (ver `docs/mcp.md`).
- Rediseño de los contratos de negocio: la documentación refleja lo implementado.

## Supuestos

- `utoipa` 6 + `utoipa-swagger-ui` 10 son compatibles con Axum 0.8 y Rust 1.99 (verificado en
  [research.md](./research.md)).
- DBHub es aceptable como MCP de PostgreSQL porque soporta `readonly = true` y límite de filas.
- OpenCode V2 lee servidores MCP de proyecto desde `mcp.servers.*` en `opencode.json`.
