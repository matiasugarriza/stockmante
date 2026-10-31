# Plan de implementación: Alertas de stock mínimo (011)

**Spec**: `spec.md` · **Migración**: ninguna (usa 0009).

## Contexto técnico

- `GET /api/stock?bajo_minimo=true` ya existe (010) y devuelve productos bajo mínimo con diferencia.
- `productos.stock_minimo` existe desde 0002.
- Correo: `EnvioCorreo` trait + `CorreoDev`/`CorreoSmtp`.

## Decisiones de diseño

1. Reusar el endpoint existente; agregar `GET /api/alertas/stock-minimo` como alias semántico (opcional) — o solo usar el filtro. Decisión: usar el filtro existente y agregar un panel en `/admin/stock` con badge.
2. Notificación por correo: endpoint `POST /api/alertas/stock-minimo/notificar` (GESTION) que envía un resumen a los usuarios GESTION. Opcional; implementar si el correo está configurado.
3. Frontend: badge en header con conteo de productos bajo mínimo (polling al cargar).

## Archivos

| Archivo | Cambio |
|---|---|
| `frontend/src/routes/admin/stock/+page.svelte` | sección "Bajo mínimo" destacada |
| `frontend/src/routes/+layout.svelte` | badge de alertas |
| `docs/api.md` | (sin cambios de backend) |

## Estrategia de pruebas

- Frontend: tests de componente (opcional).

## Riesgos

- Notificación por correo requiere SMTP; en dev se usa CorreoDev (se registra en bandeja).