# API Contracts — Gestión de usuarios

**Base**: `/api` | **Formato**: JSON (UTF-8) | **Auth**: cookie de sesión `HttpOnly` | **Creado**: 2026-10-25

El sobre de error, la cookie de sesión y los guards son el **contrato base** de
[001/contracts/api.md](../../001-autenticacion-sesion/contracts/api.md). Acá solo se documentan
los endpoints de US2.

## Usuarios (familia GESTION)

| Method | Path | Body / Query | Éxito | Errores clave |
|--------|------|--------------|-------|----------------|
| GET | `/usuarios` | `?activo=true\|false\|all` + `pagina`, `por_pagina` | 200 `{ items, total, pagina, por_pagina }` | 403 sin_permisos; 422 validacion |
| GET | `/usuarios/{id}` | — | 200 usuario | 404; 403 |
| POST | `/usuarios` | `{ nombre, email, rol }` — genera contraseña temporal y la envía por correo | 201 + `{ id, nombre, email, rol, familia_rol, activo, password_temporal: true, creado_en }` | 409 email_duplicado; 422 validacion; 403 |
| PUT | `/usuarios/{id}` | `{ nombre?, email?, rol? }` | 200 usuario | 409 email_duplicado / ultimo_administrador; 422; 404 |
| POST | `/usuarios/{id}/desactivar` | — | 204; revoca sesiones | 409 ultimo_administrador; 404 |
| POST | `/usuarios/{id}/reactivar` | — | 204 | 404 |
| POST | `/usuarios/{id}/password-temporal` | — | 204; nueva temporal enviada; revoca sesiones | 404 |

> Toda fila hereda además 401 (`no_autenticado`/`sesion_expirada`) y 403 (`sin_permisos` o
> `cambio_password_requerido`) de la tabla de guards; la columna «Errores clave» solo lista los
> específicos del endpoint.

**Item usuario**: `{ id, nombre, email, rol, familia_rol, activo, password_temporal, creado_en }`
— nunca incluye `password_hash` ni la contraseña temporal.

## Guards y errores

| Situación | Status | `codigo` |
|-----------|--------|----------|
| Sin sesión o sesión revocada/expirada | 401 | `no_autenticado` / `sesion_expirada` |
| Familia OPERACION | 403 | `sin_permisos` |
| Gestión con contraseña temporal pendiente | 403 | `cambio_password_requerido` |
| Usuario inexistente | 404 | `no_encontrado` |
| Email repetido | 409 | `email_duplicado` |
| Desactivar/degradar al último gestor | 409 | `ultimo_administrador` |
| Payload inválido (incluye extractores) | 422 | `validacion` |

## Notas de contrato

- No existe `DELETE /usuarios/{id}`: la baja es lógica (`POST …/desactivar`).
- El alta devuelve 201 con el usuario público; la temporal solo viaja por correo (o consola DEV).
- `GET /usuarios` requiere GESTION; el filtro `activo=all` es válido.
- Todas las rutas exigen `RequiereGestion` y quedan bloqueadas por `DebeCambiarPassword`
  mientras la temporal esté vigente.
