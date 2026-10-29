# Plan de implementación: Asistente de inventario (017)

**Spec**: `spec.md` · **Migración**: ninguna.

## Contexto técnico

- Stack IA: LangChain + LLM (OpenAI/Anthropic) + Tools (MCP servers).
- El backend expone endpoints de consulta (stock, movimientos, reportes) que el asistente consume como tools.
- Seguridad: validar permisos del usuario antes de ejecutar movimientos.

## Decisiones de diseño

1. Endpoint `POST /api/asistente/consulta` (GESTION/OPERACION) que recibe un mensaje y devuelve respuesta del LLM con tool calls.
2. Tools: consultar stock, buscar producto, historial de movimientos, reportes (read-only por defecto).
3. Escritura (movimientos) solo con confirmación explícita del usuario y permisos GESTION.
4. Frontend: widget de chat en `/asistente` o panel lateral.

## Archivos

- `backend/src/services/asistente.rs`, `handlers/asistente.rs`
- `frontend/src/routes/asistente/+page.svelte`

## Riesgos

- Requiere API key de LLM en env; sin ella, endpoint devuelve 503.
- Complejidad de integración LangChain en Rust (usar cliente HTTP directo a OpenAI API).