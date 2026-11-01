# Especificación de Funcionalidad: Documentación de API y Servidores MCP

**Spec**: `000-plataforma-documentacion-mcp` | **Tipo**: plataforma (transversal) | **Prioridad**: P0

**Creado**: 2026-10-26 | **Última Actualización**: 2026-10-31 | **Estado**: En Revisión / Mejorado

**Entrada**: Pedido del usuario (2026-10-26): Antes de continuar con las specs 004–006, documentar la API del backend (OpenAPI/Swagger UI) y configurar los servidores MCP del proyecto (DBHub para PostgreSQL y Playwright para E2E/UI).

**Depende de**: Specs 001–003 implementadas (superficie HTTP y modelos existentes).

> No es un caso de uso de negocio: es la infraestructura de documentación y de herramientas del entorno de desarrollo para 004–006. No agrega entidades ni altera contratos de negocio existentes.

---

## Problema y Resultado Esperado

Hoy la API solo está descrita en archivos Markdown dispersos (`specs/*/contracts/api.md`); no existe un contrato legible por máquina ni una referencia interactiva. Además, la falta de integración MCP versionada obliga a los agentes de desarrollo a depender de memoria o ejecuciones manuales de `psql` e inspecciones ciegas del frontend.

**Resultado Esperado:**

1. **OpenAPI 3.1 & Swagger UI:** Documentación generada dinámicamente desde el código Axum/Rust (`utoipa`), expuesta en `/docs`, con soporte de esquemas DTO exactos, sobre de error uniforme del Módulo 1 y autenticación basada en cookie.
2. **PostgreSQL MCP (DBHub):** Configuración versionada en modo **solo lectura** para que los agentes inspeccionen el esquema de la base de datos antes de generar SQL.
3. **Playwright MCP:** Integración para la validación visual y pruebas E2E autónomas sobre la aplicación frontend servida por **Vite** (`http://localhost:5173`).
4. **Documentación de Arquitectura:** Archivos en la carpeta `docs/` enlazados desde el `README.md` principal.

---

## Requisitos Funcionales

### Documentación de API (OpenAPI & Swagger)

- **FR-DOC-01**: El backend DEBE exponer `GET /api-docs/openapi.json` con un documento OpenAPI 3.1 que incluya todas las rutas `/api/*` existentes, sus parámetros, DTOs de request/response con tipos explícitos (ej. UUID, date-time ISO8601) y el esquema de seguridad por cookie de sesión (`cookieAuth`).
- **FR-DOC-02**: El backend DEBE servir Swagger UI en `GET /docs` apuntando al endpoint OpenAPI.
- **FR-DOC-03**: Las rutas de documentación DEBEN controlarse mediante la variable de entorno `DOCS_ENABLED`. En `APP_ENV=production` el valor por defecto es `false`; fuera de producción es `true`. Se DEBE permitir la sobreescritura explícita mediante `.env.local`.
- **FR-DOC-04**: Las rutas de documentación son públicas (no exigen sesión) y NO DEBEN exponer secretos ni variables de entorno sensibles (solo el identificador público de la cookie de sesión).

### Integración de Servidores MCP

- **FR-MCP-01**: El repositorio DEBE incluir el archivo `dbhub.toml` en la raíz con la configuración del MCP PostgreSQL en modo **solo lectura** (`readonly = true`, `max_rows = 100`).
- **FR-MCP-02**: La configuración de DBHub NO DEBE contener credenciales cableadas; la cadena de conexión se extraerá de la variable `DATABASE_URL` del entorno.
- **FR-MCP-03**: El repositorio DEBE incluir la configuración de **Playwright MCP** (`@modelcontextprotocol/server-playwright`) para permitir la interacción del navegador sin cabeza (headless) sobre la URL de desarrollo de Vite (`http://localhost:5173`).
- **FR-MCP-04**: La documentación en `docs/mcp.md` DEBE detallar el uso de los MCPs (DBHub, Playwright, Filesystem) y explicar por qué el MCP de inventario custom queda fuera de alcance del Módulo 1.

---

## Criterios de Aceptación

- **CA-01**: `cargo test --test test_openapi` en verde: El JSON responde 200, valida contra OpenAPI 3.1 y contiene las rutas del Módulo 1, el esquema `cookieAuth` y los schemas de `Producto`, `Categoria` y `Usuario`.
- **CA-02**: Con `DOCS_ENABLED=true`, `GET /api-docs/openapi.json` responde 200 y `GET /docs` sirve Swagger UI. Con `DOCS_ENABLED=false`, ambas rutas responden 404 Not Found.
- **CA-03**: `npx -y @bytebase/dbhub@latest --config dbhub.toml` conecta con la base de datos `stockmante` y rechaza cualquier instrucción de escritura (`INSERT`, `UPDATE`, `DELETE`, `DROP`).
- **CA-04**: Playwright MCP ejecuta exitosamente sobre `http://localhost:5173`, permitiendo al agente tomar capturas de pantalla y verificar el renderizado de los componentes Svelte.
- **CA-05**: Los archivos `docs/api.md`, `docs/arquitectura.md` y `docs/mcp.md` existen y están enlazados desde `README.md`.
- **CA-06**: Los agentes de OpenCode logran consultar el esquema PostgreSQL vía DBHub y validar la UI vía Playwright sin entrar en bucles de reintento.

---

## Fuera de Alcance

- Autenticación adicional para Swagger UI más allá de la cookie existente (no se implementarán API Keys dedicadas).
- Generación automática de clientes SDK o publicación del esquema JSON en portales externos.
- MCP de inventario custom o integración obligatoria con la API de GitHub.
- Rediseño de los contratos de negocio: La documentación debe reflejar estrictamente la implementación actual.

---

## Supuestos y Dependencias

- **Backend Stack:** `utoipa` 6 + `utoipa-swagger-ui` 10 son compatibles con Axum 0.8 y Rust 1.99.
- **PostgreSQL MCP:** DBHub se valida como el servidor de solo lectura por defecto.
- **Frontend Stack:** Vite sirve la aplicación frontend en `http://localhost:5173` y es accesible localmente por Chromium/Playwright.
- **Entorno de Agente:** OpenCode V2 lee servidores MCP desde la configuración del proyecto (`opencode.json` o `.opencode/mcp.json`).
