# Quickstart — RBAC transversal y auditoría (006)

**Objetivo**: verificar la matriz RBAC y la auditoría básica. Requiere PostgreSQL con
`stockmante` y `stockmante_test` migradas hasta `0008` y el backend en `:3000`.

## Preparación

```powershell
cd C:\Users\Matias\Documents\tpOpencode\stockmante-v0\stockmante
cargo sqlx migrate run --source migrations --database-url "postgres://stockmante:stockmante@localhost:5432/stockmante"
cargo sqlx migrate run --source migrations --database-url "postgres://stockmante:stockmante@localhost:5432/stockmante_test"
cd backend
cargo test --test test_rbac_matriz
cargo test
```

Resultado esperado: `test_rbac_matriz` 4/4 y la suite completa en verde.

## Verificación manual de RBAC

Con una sesión OPERACION (`-c cookies-op.txt`):

1. `POST /api/productos` con un cuerpo válido → **403** `sin_permisos`.
2. `PUT /api/productos/1` → **403**.
3. `POST /api/usuarios` → **403**.
4. `GET /api/usuarios` → **403**.
5. `GET /api/productos` → **403**.
6. `GET /api/productos/buscar?q=yerba` → **200** (lectura de caja permitida).
7. `GET /api/productos/1` → **200/404** (lectura de caja permitida).
8. `DELETE /api/depositos/1` → **403**.

Con una sesión GESTION (`-c cookies-ge.txt`): repetir 1–5 y 8 → nunca 401/403; se obtiene la
respuesta de negocio (201/200/404/422).

## Verificación manual de auditoría

Con la sesión GESTION:

1. Crear una categoría y un producto: `POST /api/productos` → 201.
2. Editar el precio: `PUT /api/productos/{id}` con `{ "precio_venta": 175 }` → 200.
3. `POST /api/productos/{id}/desactivar` → 204.
4. `POST /api/productos/{id}/reactivar` → 204.
5. Consultar:

```sql
SELECT tabla_ref, registro_id, accion, usuario_id, cuando
FROM auditoria
WHERE tabla_ref = 'productos' AND registro_id = <id>
ORDER BY id;
```

Debe devolver `CREAR`, `ACTUALIZAR`, `DESACTIVAR`, `REACTIVAR`, todas con el `usuario_id` del
gestor y `cuando` no nulo.

6. Repetir con un usuario: alta, `PUT`, desactivar, reactivar y `password-temporal`. La consulta
   debe devolver `CREAR`, `ACTUALIZAR`, `DESACTIVAR`, `REACTIVAR`, `PASSWORD_TEMPORAL`.

## Verificación del 403 sin auditoría

1. Anotar `SELECT count(*) FROM auditoria WHERE usuario_id = <id_operario>;`.
2. Intentar `POST /api/productos` con la sesión OPERACION → 403.
3. Repetir el conteo: debe ser idéntico (0 si el operario nunca completó una gestión).

## Verificación de SC-007

```sql
-- Productos y depósitos con dependencias no se borran físicamente.
-- La API responde 409 y ofrece baja lógica.
```

- `DELETE /api/depositos/{id}` con ubicaciones → 409 `deposito_con_dependencias`.
- `DELETE /api/ubicaciones/{id}` con hijos → 409 `ubicacion_con_dependencias`.
- `POST /api/.../desactivar` → 204 como alternativa.

## Frontend

1. Iniciar sesión como OPERACION y navegar manualmente a `/admin/productos` → redirige a `/caja`.
2. El header no muestra Usuarios/Categorías/Productos/Depósitos.
3. Iniciar sesión como GESTION: las rutas `/admin/*` se muestran y funcionan.

## Evidencia a registrar

- Salida de `test_rbac_matriz` y de la suite.
- Filas de `auditoria` antes/después de un 403.
- Captura/nota de la redirección de `/admin` para OPERACION.
