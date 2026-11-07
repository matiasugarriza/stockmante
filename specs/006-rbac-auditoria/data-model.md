# Data Model — RBAC transversal y auditoría (006)

**Spec**: [spec.md](./spec.md) | **Plan**: [plan.md](./plan.md)

## Delta de esquema: `auditoria`

Creada por `migrations/0008_auditoria.sql`.

| Columna | Tipo | Restricción | Uso |
|---|---|---|---|
| `id` | `BIGINT GENERATED ALWAYS AS IDENTITY` | PK | orden de inserción |
| `tabla_ref` | `VARCHAR(32)` | `NOT NULL` | `usuarios`, `productos`, `categorias`, `depositos` o `ubicaciones` |
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
| `ASIGNAR_UBICACIONES` | `productos` | `PUT /api/productos/{id}/ubicaciones` (reemplazo, incluso `[]`) |
| `CREAR` | `categorias` | alta de categoría |
| `ACTUALIZAR` | `categorias` | `PUT /api/categorias/{id}` |
| `DESACTIVAR` | `categorias` | baja lógica |
| `REACTIVAR` | `categorias` | reactivación |
| `CREAR` | `depositos` | alta de depósito |
| `ACTUALIZAR` | `depositos` | `PUT /api/depositos/{id}` |
| `ELIMINAR` | `depositos` | borrado físico permitido (sin dependencias) |
| `DESACTIVAR` | `depositos` | baja lógica |
| `REACTIVAR` | `depositos` | reactivación |
| `CREAR` | `ubicaciones` | alta de ubicación |
| `ACTUALIZAR` | `ubicaciones` | `PUT /api/ubicaciones/{id}` |
| `ELIMINAR` | `ubicaciones` | borrado físico permitido (sin dependencias) |
| `DESACTIVAR` | `ubicaciones` | baja lógica |
| `REACTIVAR` | `ubicaciones` | reactivación |

## Invariantes

1. Toda fila se inserta en la misma transacción que el cambio auditado.
2. Si el cambio se revierte, la fila desaparece con él.
3. Un request rechazado (401/403/409/422) no produce filas.
4. `registro_id` identifica el registro dentro de `tabla_ref`; no hay FK.
5. `usuario_id` es el autor validado por la sesión (`RequiereGestion`), nunca un dato del body.
6. `cuando` lo genera la base; el cliente no lo envía.
7. El borrado físico permitido de depósitos/ubicaciones se audita como `ELIMINAR`; la fila de
   auditoría sobrevive porque `registro_id` no tiene FK.
8. La asignación producto–ubicación se audita sobre el producto (`ASIGNAR_UBICACIONES`,
   `registro_id = producto_id`), no sobre la tabla puente.

## Fuera del modelo

- Diffs, valores anteriores/nuevos o snapshots.
- Consultas y reportes de auditoría.
- Auditoría de movimientos de stock, ventas, compras y eventos de autenticación.

## Verificación

```sql
SELECT tabla_ref, registro_id, accion, usuario_id, cuando
FROM auditoria
ORDER BY id DESC
LIMIT 20;
```

Toda fila debe tener `cuando NOT NULL`; `usuario_id` puede ser `NULL` solo si la cuenta autora se
borró físicamente (`ON DELETE SET NULL`), y los consumidores deben tolerarlo.
