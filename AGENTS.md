# AGENTS.md - Directivas para Agentes IA

## 🎯 Objetivo General

Desarrollar un sistema de gestión y control de inventario robusto, modular y seguro compuesto por un frontend en SvelteKit, un backend en Rust (Axum) y una base de datos PostgreSQL.

---

## 🛠️ Stack Tecnológico

- **Frontend:** SvelteKit (SPA mode) + Tailwind CSS
- **Backend:** Rust + Axum + SQLx ORM
- **Base de Datos:** PostgreSQL
- **IA / Chatbot:** LangChain + API de LLM (OpenAI/Anthropic)
- **Herramientas MCP:** MCP Server de PostgreSQL, MCP Server de Filesystem, MCP Server de GitHub, MCP Server Customizado para Inventario
- **Contexto & Memoria:** Gentle_AI + Engram
- **SDD:** Spec Kit

---

## 🎯 Metodología de Trabajo: Spec-Driven Development (SDD)

1. **Fase de Especificación:** Prohibido escribir código sin antes verificar o redactar la especificación en `specs/<NNN-feature>/spec.md`, generada con `/speckit.specify` y, cuando corresponda, completada con `/speckit.plan` y `/speckit.tasks`.
2. **Fase de Verificación de Esquema:** Utiliza el servidor MCP de PostgreSQL (o `sqlx-cli` como fallback) para contrastar la Spec contra el esquema real de la BD antes de generar código.
3. **Fase de Memoria con Engram:** Antes y después de cada sesión de desarrollo, consulta/actualiza la memoria con Engram para registrar decisiones de arquitectura.

---

## ✅ Quality Gates (por tarea)

- `cargo test` (backend) en verde y `cargo sqlx prepare --check` sin diferencias.
- Suite frontend en verde; la lógica de `$lib/api/*.ts` y de los stores debe tener tests.
- Sin `unwrap()` / `panic!` / `expect()` en rutas de producción del backend.
- Spec actualizada si el comportamiento cambió respecto de lo planificado.

---

## 📏 Reglas Arquitectónicas y de Código

### 1. Backend en Rust (Axum)

- **Manejo de Errores:** Prohibido el uso de `panic!`, `unwrap()` o `expect()` en producción. Todo debe manejar tipos `Result<T, AppError>`.
- **Atomicidad Operativa:** Todos los movimientos de stock (entradas, salidas, mermas, ajustes) DEBEN ejecutarse dentro de una transacción explícita de SQLx (`pool.begin()`).
- **Validación con MCP:** Antes de generar o modificar una consulta SQL o struct de `SQLx`, utiliza el MCP de PostgreSQL para verificar que los nombres de tablas y columnas coincidan exactamente con la base de datos real.

### 2. Frontend en SvelteKit

- Mantén componentes pequeños, reactivos y modulares.
- Utiliza **Tailwind CSS** para un diseño limpio, responsivo y funcional.
- Toda interacción con la API de Rust debe realizarse mediante funciones centralizadas y tipadas (`fetch`).

### 3. Asistente / Chatbot Integrado

- El chatbot debe poder consultar la base de datos y realizar acciones de inventario mediante herramientas (`Tools`).
- Se debe validar la autenticación y permisos del usuario antes de permitir que el asistente ejecute un movimiento de stock.

---

## 🔄 Flujo de Trabajo

1. Al recibir una tarea, revisa siempre `SKILLS.md` para aplicar las buenas prácticas de la tarea correspondiente.
2. Si la tarea involucra consultas a la base de datos, consulta primero el esquema real mediante el **servidor MCP de PostgreSQL**.
3. Realiza cambios incrementales y verifica que no rompan código previo.
