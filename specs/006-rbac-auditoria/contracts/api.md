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
| `GET /api/usuarios`, `/api/usuarios/{id}` | 401 | **403** | 200/404 |
| `GET /api/depositos`, `/api/depositos/{id}`, `/{id}/ubicaciones` | 401 | **403** | 200/404 |
| `GET /api/ubicaciones/{id}` | 401 | **403** | 200/404 |
| `GET /api/productos/{id}/ubicaciones` | 401 | **403** | 200/404 |
| `POST/PUT/DELETE` de usuarios | 401 | **403** `sin_permisos` | 2xx/4xx de negocio |
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

Reglas:

- La fila se escribe en la misma transacción; un fallo de negocio no deja auditoría.
- Un 401/403 no deja auditoría.
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
