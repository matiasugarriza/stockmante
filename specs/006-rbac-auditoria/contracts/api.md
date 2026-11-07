# API Contracts — RBAC transversal y auditoría (006)

**Base**: `/api` | **Auth**: cookie `stockmante_sesion` | **Errores**: `{ error: { codigo, mensaje } }`

Esta spec **no agrega endpoints públicos**. Su contrato es la matriz de acceso vigente sobre
001–005 y el efecto de auditoría de las escrituras.

## Matriz de acceso

| Superficie | Anónimo | OPERACION | GESTION |
|---|---|---|---|
| `GET /api/productos/{id}` | 401 | 200/404 | 200/404 |
| `GET /api/productos/buscar` | 401 | 200/422 | 200/422 |
| `GET /api/categorias` | 401 | 200 | 200 |
| `GET /api/productos` | 401 | **403** | 200 |
| `GET /api/productos/ubicaciones` (lote) | 401 | **403** | 200 |
| `GET /api/usuarios`, `/api/usuarios/{id}` | 401 | **403** | 200/404 |
| `GET /api/depositos`, `/api/depositos/{id}`, `/{id}/ubicaciones` | 401 | **403** | 200/404 |
| `GET /api/ubicaciones/{id}` | 401 | **403** | 200/404 |
| `GET /api/productos/{id}/ubicaciones` | 401 | **403** | 200/404 |
| `POST/PUT` de usuarios (no existe `DELETE`) | 401 | **403** `sin_permisos` | 2xx/4xx de negocio |
| `POST/PUT` de categorías y productos | 401 | **403** `sin_permisos` | 2xx/4xx de negocio |
| `POST/PUT/DELETE` de depósitos y ubicaciones | 401 | **403** `sin_permisos` | 2xx/4xx de negocio |
| `PUT /api/productos/{id}/ubicaciones` | 401 | **403** `sin_permisos` | 204/4xx |

El 403 se evalúa antes de parsear el cuerpo o resolver el recurso: un OPERACION no puede
distinguir si el id existe.

## Efecto de auditoría

| Endpoint | Acción registrada |
|---|---|
| `POST /api/usuarios` | `CREAR` en `usuarios` |
| `PUT /api/usuarios/{id}` | `ACTUALIZAR` en `usuarios` |
| `POST /api/usuarios/{id}/desactivar` | `DESACTIVAR` en `usuarios` |
| `POST /api/usuarios/{id}/reactivar` | `REACTIVAR` en `usuarios` |
| `POST /api/usuarios/{id}/password-temporal` | `PASSWORD_TEMPORAL` en `usuarios` |
| `POST /api/productos` | `CREAR` en `productos` |
| `PUT /api/productos/{id}` | `ACTUALIZAR` en `productos` |
| `POST /api/productos/{id}/desactivar` | `DESACTIVAR` en `productos` |
| `POST /api/productos/{id}/reactivar` | `REACTIVAR` en `productos` |
| `PUT /api/productos/{id}/ubicaciones` | `ASIGNAR_UBICACIONES` en `productos` |
| `POST /api/categorias` | `CREAR` en `categorias` |
| `PUT /api/categorias/{id}` | `ACTUALIZAR` en `categorias` |
| `POST /api/categorias/{id}/desactivar` | `DESACTIVAR` en `categorias` |
| `POST /api/categorias/{id}/reactivar` | `REACTIVAR` en `categorias` |
| `POST /api/depositos` | `CREAR` en `depositos` |
| `PUT /api/depositos/{id}` | `ACTUALIZAR` en `depositos` |
| `DELETE /api/depositos/{id}` | `ELIMINAR` en `depositos` |
| `POST /api/depositos/{id}/desactivar` | `DESACTIVAR` en `depositos` |
| `POST /api/depositos/{id}/reactivar` | `REACTIVAR` en `depositos` |
| `POST /api/ubicaciones` | `CREAR` en `ubicaciones` |
| `PUT /api/ubicaciones/{id}` | `ACTUALIZAR` en `ubicaciones` |
| `DELETE /api/ubicaciones/{id}` | `ELIMINAR` en `ubicaciones` |
| `POST /api/ubicaciones/{id}/desactivar` | `DESACTIVAR` en `ubicaciones` |
| `POST /api/ubicaciones/{id}/reactivar` | `REACTIVAR` en `ubicaciones` |

Reglas:

- La fila se escribe en la misma transacción; un fallo de negocio no deja auditoría.
- Un 401/403/409/422 no deja auditoría.
- `ELIMINAR` (borrado físico permitido de depósitos/ubicaciones) se audita en la misma transacción;
  la fila sobrevive porque `registro_id` no tiene FK.
- `ASIGNAR_UBICACIONES` se registra sobre el producto, incluso cuando el conjunto queda vacío.
- `usuario_id` es el autor de la sesión; `cuando` lo pone la base.
- No hay endpoint para consultar la auditoría (fuera de alcance).

## Errores relevantes

| Código | Status | Cuándo |
|---|---|---|
| `sin_permisos` | 403 | sesión OPERACION en superficie GESTION |
| `no_autenticado` | 401 | sin cookie o sesión inválida |
| `cambio_password_requerido` | 403 | contraseña temporal vigente |

## Fuera de este contrato

- Log rico, diffs, consultas y reportes.
- Diferenciación entre ADMIN y GERENTE.
- Auditoría de depósitos/ubicaciones.
