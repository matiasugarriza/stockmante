# Plan de implementación: Auditoría consultable (015)

**Spec**: `spec.md` · **Migración**: ninguna (lee tabla `auditoria`).

## Contexto técnico

- Tabla `auditoria` existe (006) con tabla_ref, registro_id, accion, usuario_id, cuando.
- Falta endpoint de consulta y pantalla.

## Decisiones de diseño

1. `GET /api/auditoria` con filtros tabla_ref, usuario_id, desde, hasta, paginado.
2. Solo GESTION.
3. Frontend: `/admin/auditoria`.

## Archivos

- `backend/src/services/auditoria.rs` (agregar listar)
- `backend/src/handlers/auditoria.rs`
- `frontend/src/routes/admin/auditoria/+page.svelte`