# AI Engineering — bitácora del proceso

Este documento complementa el [`README.md`](../README.md) y responde a los criterios de
**Esquema de Trabajo y AI Engineering**: qué herramientas se usaron, cómo se estructuró el
contexto, cuáles fueron los prompts clave, cómo se iteró y qué loops de autocorrección se
ejecutaron.

## 1. Entorno de desarrollo asistido

| Herramienta | Rol en el proyecto | Evidencia |
|---|---|---|
| **OpenCode / DeepSeek Harness** | Agente principal de desarrollo, lectura/edición del workspace, ejecución de comandos y tests | `opencode.json`, `.opencode/commands/`, `.dsh/skills/` |
| **Spec Kit (Spec-Driven Development)** | Generación de `spec.md`, `plan.md`, `tasks.md`, `research.md`, contratos y quickstarts | `.specify/`, `specs/` (20 specs, 153 archivos `.md`) |
| **Skills nativas** | Procedimientos reutilizables para especificar, planificar, implementar, analizar y converger | `.dsh/skills/speckit-*`, `.github/skills/speckit-*` |
| **MCP PostgreSQL (DBHub)** | Verificación del esquema real antes de escribir SQL o structs de SQLx | `opencode.json`, `dbhub.toml`, [`docs/mcp.md`](./mcp.md) |
| **MCP Playwright** | Validación visual/E2E y capturas de pantalla sobre el frontend en Vite | `opencode.json`, `scripts/capturar-rediseno.mjs` |
| **Playwright (sin MCP)** | Scripts de captura reproducible y mediciones de layout | `scripts/*.mjs` |
| **Vitest / Cargo test / svelte-check** | Red de seguridad automática de cada cambio | `frontend/`, `backend/tests/`, `asistente/src/*.test.ts` |
| **Gentle_AI** | Registro local de skills para delegar trabajo a subagentes | `.atl/skill-registry.md` (local, ignorado por git) |
| **Git + commits convencionales** | Unidad de trabajo revisable por feature | 94 commits, `feat(007)`, `feat(017)`, etc. |

**No se usó Engram**: la constitución retiró esa dependencia en la versión 1.1.1. La memoria
persistente del proyecto vive en las specs, los `progress.md`, los `tasks.md` y la propia
constitución; no se afirma lo contrario en el README.

## 2. Jerarquía de contexto (cómo se guio al agente)

El contexto se estructuró en capas, de lo más estable a lo más específico:

1. **Constitución** (`.specify/memory/constitution.md`, v1.3.4): principios no negociables
   —simplicidad/YAGNI, mantenibilidad, claridad de mensajes, testabilidad e integridad de
   datos—, stack fijo y reglas de seguridad del chatbot.
2. **`AGENTS.md`**: reglas operativas del repositorio (stack, quality gates, arquitectura,
   prohibición de `unwrap`/`panic!`, verificación de esquema, flujo SDD).
3. **`SKILLS.md`**: catálogo de patrones y snippets obligatorios por tipo de tarea (auth,
   catálogo, depósitos, errores, frontend, stock, chatbot, etc.).
4. **Specs por caso de uso** (`specs/<NNN>-*/`): alcance, requisitos FR, criterios SC,
   escenarios, contratos HTTP, modelo de datos y supuestos.
5. **`plan.md` / `tasks.md` / `research.md`**: decisiones técnicas, orden de implementación y
   trazabilidad tarea → requisito.

Esta jerarquía evita que el agente invente patrones: cada cambio se contrasta primero contra la
constitución y la spec del caso de uso, y recién después se escribe código.

## 3. Flujo SDD aplicado

```text
/speckit.specify  →  /speckit.clarify  →  /speckit.plan  →  /speckit.tasks
        →  /speckit.implement  →  /speckit.analyze  →  /speckit.converge
```

- **Specify**: se redacta el caso de uso y sus criterios de aceptación antes de tocar código.
- **Clarify**: se resuelven hasta 5 ambigüedades con el usuario y se codifican en la spec.
- **Plan**: se fijan arquitectura, contratos, migraciones y estrategia de tests.
- **Tasks**: se descompone en tareas con dependencias y gates por fase.
- **Implement**: se implementa en incrementos, con tests y documentación en el mismo cambio.
- **Analyze / Converge**: se detectan inconsistencias entre spec, plan, tasks y código; lo que
  falta se agrega como tareas nuevas en lugar de improvisar.

## 4. Prompts clave (extractos verbatim)

Los prompts completos y su evolución viven en las specs. Estos son los que más influyeron en el
diseño:

### 4.1 Pedido inicial del Módulo 1

> "Crear la especificación funcional para el Módulo 1: Gestión de Usuarios, Autenticación y
> Catálogo Base (Productos, Depósitos y Ubicaciones) de Stockmante. Alcance: CRUD de usuarios
> con roles Administrador/Gerente y Operario/Cajero; autenticación login/logout/sesión con
> RBAC; unicidad de correo; baja lógica de usuarios; CRUD de productos con SKU único y código
> de barras opcional/único; búsqueda rápida por SKU/código de barras/nombre/categoría para caja
> y almacén; baja lógica de productos; CRUD de depósitos y ubicaciones jerárquicas; asignación
> inicial de ubicaciones por producto; casos de uso para Administrador y Operario; edge cases
> (SKU duplicado, desactivar usuario en sesión, ubicaciones huérfanas)."

Fuente: [`specs/001-autenticacion-sesion/spec.md`](../specs/001-autenticacion-sesion/spec.md#L9).

**Qué produjo**: la primera spec del Módulo 1, que luego se dividió en seis specs por caso de
uso (001–006) sin perder el texto original.

### 4.2 Pedido de documentación viva y MCP

> "Antes de continuar con las specs 004–006, documentar la API del backend (OpenAPI/Swagger UI)
> y configurar los servidores MCP del proyecto (DBHub para PostgreSQL y Playwright para
> E2E/UI)."

Fuente: [`specs/000-plataforma-documentacion-mcp/spec.md`](../specs/000-plataforma-documentacion-mcp/spec.md#L7).

**Qué produjo**: `utoipa` + Swagger UI, `dbhub.toml` readonly, `opencode.json` con DBHub y
Playwright, y la evidencia de lectura/escritura rechazada.

### 4.3 Pedido del asistente

> "IA/chatbot de inventario" (fuera del alcance del Módulo 1) con el stack ya fijado por la
> constitución: **LangChain + LLM con Tools** y escrituras **solo** por la API.

Fuente: [`specs/017-asistente-inventario/spec.md`](../specs/017-asistente-inventario/spec.md).

**Qué produjo**: el servicio `asistente/` en Node + LangChain.js, las tools de lectura, la tool
`proponer_movimiento`, el proxy del backend con límites y la pantalla `/asistente` con
confirmación.

### 4.4 Pedido del rediseño visual

> "Rehacer la interfaz de todas las páginas de la aplicación tomando como modelo visual las
> muestras `captures/samples/light-2.png` y `captures/samples/dark-2.png` (panel de
> administración StarAdmin, variantes clara y oscura), y registrar el trabajo en esta spec 018."

Fuente: [`specs/018-rediseno-visual-desktop/spec.md`](../specs/018-rediseno-visual-desktop/spec.md).

**Qué produjo**: tokens semánticos claro/oscuro, marco de escritorio, 17 capturas de evidencia
y el rediseño incremental página por página.

### 4.5 Pedido del rediseño móvil

> "Adaptar y optimizar la experiencia visual de Stockmante en dispositivos móviles con una
> interfaz de alta densidad bajo estética cyberpunk dark, para que operarios y administradores
> consulten kardex, registren entradas/salidas y supervisen alertas críticas desde pantallas
> táctiles pequeñas sin perder claridad operativa ni rendimiento visual."

Fuente: [`specs/019-rediseno-visual-mobile/spec.md`](../specs/019-rediseno-visual-mobile/spec.md).

**Qué produjo**: la spec 019 y su implementación parcial (58/87 tareas), documentada como
trabajo en curso y fuera de los 10 CUs.

## 5. Metodología de iteración

- **Una spec por caso de uso**, no una spec por módulo. Cada una tiene su alcance, criterios,
  plan y tareas.
- **Cambios incrementales**: una tarea = un cambio revisable; los tests y la documentación van
  en el mismo commit.
- **Contratos primero**: los endpoints, códigos de error y mensajes se fijan en
  `specs/*/contracts/api.md` antes de implementar.
- **Migraciones con ledger global**: la numeración no se reutiliza; el mapa vive en
  [`specs/README.md`](../specs/README.md).
- **Decisiones registradas**: `research.md` guarda las alternativas descartadas y el porqué
  (por ejemplo, por qué DBHub y no un MCP propio; por qué LangChain.js y no un servicio Rust).
- **Clarify antes de codificar**: las preguntas abiertas se responden y se escriben en la spec,
  no quedan en la cabeza del desarrollador.

## 6. Loops de autocorrección

El proyecto no tiene un loop “mágico” de auto-mejora; tiene **gates reproducibles** que obligan
a corregir antes de cerrar cada tarea:

| Loop | Qué hace | Evidencia |
|---|---|---|
| **RED → GREEN** | Ante un bug o una funcionalidad nueva, primero se escribe el test que falla; luego se implementa; luego se refactoriza con la suite en verde | `tasks.md` de 007–017; “Registro de RED” en [`specs/017-asistente-inventario/tasks.md`](../specs/017-asistente-inventario/tasks.md) |
| **Quality gates** | `cargo test`, `cargo sqlx prepare --check`, `npm test`, `npm run check`, `npm run build` | `README.md`, `AGENTS.md`, `.specify/memory/constitution.md` |
| **Analyze / Converge** | `/speckit.analyze` busca inconsistencias spec/plan/tasks; `/speckit.converge` agrega lo que falta como tareas nuevas | `.dsh/skills/speckit-analyze`, `speckit-converge`; specs 004–016 |
| **Verificación de esquema** | Antes de cada SQL/migración se contrasta el esquema real (MCP PostgreSQL o `sqlx-cli`) | `dbhub.toml`, `docs/mcp.md`, `tasks.md` de 003, 010–014 |
| **Waivers explícitos** | Cuando el RED no se pudo observar, se registra un waiver con alcance y condición de cierre (W-001…W-006) | `.specify/memory/constitution.md` |
| **Playwright** | Capturas y mediciones de layout/contraste para detectar regresiones visuales | `scripts/capturar-rediseno.mjs`, `scripts/capturar-movil.mjs`, `specs/018/evidencia/` |

Ejemplos concretos de autocorrección:

- **008**: la primera versión de transferencias devolvía 500 por columnas inexistentes; el test
  HTTP lo detectó y el fix quedó en `fee6aef` (“transferencias sin 500 y saldos derivados de
  las líneas”).
- **012**: la anulación concurrente de compras se corrigió con un `UPDATE` condicional y un
  test de concurrencia (`test_compras_concurrencia`).
- **017**: el test de logs pasaba aislado y fallaba en paralelo por el cache de `tracing`; se
  movió a un binario propio (`test_asistente_log`).
- **018/019**: `svelte-check` y el guard de contraste de la paleta móvil se usaron como loop
  visual antes de cada captura.

## 7. Evidencia cuantitativa

| Métrica | Valor |
|---|---|
| Specs | 21 directorios (`000`–`020`) |
| Artefactos de spec | 155 archivos `.md` (spec, plan, tasks, research, contracts, quickstarts, checklists) |
| Planes / tareas | 20 `plan.md` + 20 `tasks.md` |
| Commits | 94 |
| Tests de backend | 294 funciones de test; 3 pruebas de carga ignoradas a propósito |
| Tests de frontend | 388 |
| Tests del asistente | 54 |
| Gates | `cargo test`, `cargo sqlx prepare --check`, `npm test`, `npm run check`, `npm run build` en verde |

## 8. Lecciones y decisiones

- **La especificación es el prompt más importante**: cuando la spec fija contratos y criterios,
  el agente implementa sin inventar y el evaluador puede verificar.
- **Los tests son el loop de autocorrección**: sin gates, el agente “cree” que terminó; con
  gates, el repositorio demuestra que terminó.
- **La seguridad se diseña, no se promptea**: el asistente no escribe en la base; propone y el
  usuario confirma a través de la API autenticada. El prompt de sistema lo dice, pero la regla
  real está en el código.
- **YAGNI**: no se construyó un MCP de inventario propio porque las tools de LangChain contra la
  API cubren el caso sin duplicar permisos ni reglas de stock.

## 9. Reproducción

```bash
# Contexto
cat AGENTS.md SKILLS.md .specify/memory/constitution.md

# Specs
ls specs/
cat specs/017-asistente-inventario/spec.md
cat specs/017-asistente-inventario/tasks.md

# Gates
cd backend && cargo test && cargo sqlx prepare --check
cd ../frontend && npm test && npm run check && npm run build
cd ../asistente && npm test && npm run check
```
