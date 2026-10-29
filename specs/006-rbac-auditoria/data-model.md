# Data Model — RBAC transversal y auditoría (006)

**Spec**: [spec.md](./spec.md) | **Plan**: [plan.md](./plan.md)

## Delta de esquema: `auditoria`

Creada por `migrations/0008_auditoria.sql`.

| Columna | Tipo | Restricción | Uso |
|---|---|---|---|
| `id` | `BIGINT GENERATED ALWAYS AS IDENTITY` | PK | orden de inserción |
| `tabla_ref` | `VARCHAR(32)` | `NOT NULL` | `usuarios` o `productos` |
| `registro_id` | `INT` | `NOT NULL` | id del registro afectado; sin FK |
| `accion` | `VARCHAR(24)` | `NOT NULL` | ver catálogo abajo |
| `usuario_id` | `INT` | `NULL` → `usuarios(id)` `ON DELETE SET NULL` | autor de la acción |
| `cuando` | `TIMESTAMPTZ` | `NOT NULL DEFAULT now()` | timestamp |

Índices:

- `idx_auditoria_registro (tabla_ref, registro_id, cuando DESC)`: historia de un registro.
- `idx_auditoria_usuario (usuario_id, cuando DESC)`: acciones de un autor.

## Catálogo de acciones

| Acción | Tabla | Se registra en |
|---|---|---|
| `CREAR` | `usuarios` | alta de usuario |
| `ACTUALIZAR` | `usuarios` | `PUT /api/usuarios/{id}` |
| `DESACTIVAR` | `usuarios` | baja lógica |
| `REACTIVAR` | `usuarios` | reactivación |
| `PASSWORD_TEMPORAL` | `usuarios` | reseteo de contraseña temporal |
| `CREAR` | `productos` | alta de producto |
| `ACTUALIZAR` | `productos` | `PUT /api/productos/{id}` (incluye precio) |
| `DESACTIVAR` | `productos` | baja lógica |
| `REACTIVAR` | `productos` | reactivación |

## Invariantes

1. Toda fila se inserta en la misma transacción que el cambio auditado.
2. Si el cambio se revierte, la fila desaparece con él.
3. Un request rechazado con 403 no produce filas.
4. `registro_id` identifica el registro dentro de `tabla_ref`; no hay FK.
5. `usuario_id` es el autor validado por la sesión (`RequiereGestion`), nunca un dato del body.
6. `cuando` lo genera la base; el cliente no lo envía.

## Fuera del modelo

- Diffs, valores anteriores/nuevos o snapshots.
- Consultas y reportes de auditoría.
- Auditoría de depósitos/ubicaciones (su bloqueo de borrado es SC-007, no FR-019).

## Verificación

```sql
SELECT tabla_ref, registro_id, accion, usuario_id, cuando
FROM auditoria
ORDER BY id DESC
LIMIT 20;
```

Toda fila debe tener `cuando NOT NULL` y, en producción, `usuario_id NOT NULL`.
