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
| POST | `/usuarios` | `{ nombre, email, rol, password? }` — sin `password` ⇒ automática (14 chars); con `password` ⇒ manual (política 422); envía el correo configurado | 201 + `{ usuario: {...}, password_temporal: "..." }` | 409 email_duplicado; 422 validacion; 403 |
| PUT | `/usuarios/{id}` | `{ nombre?, email?, rol? }` | 200 usuario | 409 email_duplicado / ultimo_administrador; 422; 404 |
| POST | `/usuarios/{id}/desactivar` | — | 204; revoca sesiones | 409 ultimo_administrador; 404 |
| POST | `/usuarios/{id}/reactivar` | — | 204 | 404 |
| POST | `/usuarios/{id}/password-temporal` | `{ password? }` opcional — misma semántica manual/automática; envía correo | 200 `{ password_temporal: "..." }`; revoca sesiones | 404; 422 password inválida |

> Toda fila hereda además 401 (`no_autenticado`/`sesion_expirada`) y 403 (`sin_permisos` o
> `cambio_password_requerido`) de la tabla de guards; la columna «Errores clave» solo lista los
> específicos del endpoint.

**Item usuario**: `{ id, nombre, email, rol, familia_rol, activo, password_temporal, creado_en }`
— nunca incluye `password_hash`.

**Envoltura de credenciales** (solo `POST /usuarios` y `POST /usuarios/{id}/password-temporal`):
`{ usuario: <item usuario>, password_temporal: "<texto plano>" }`. La clave en claro existe **solo**
en estas respuestas 201/200 para la modal de entrega inmediata (FR-008b): no se persiste en claro,
no se registra en logs y no viaja por ninguna otra ruta.

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
- El alta devuelve 201 con el usuario público **más la contraseña temporal en claro** para la
  modal en pantalla (FR-008b); además se envía por el transporte configurado (o consola DEV).
  *(Cambio 2026-11: antes la clave solo viajaba por correo.)*
- `POST /usuarios/{id}/password-temporal` cambió de **204 a 200** con body `{ password_temporal }`
  para la misma modal de regreso (escenario 5 de aceptación); acepta `{ password? }` opcional con
  la misma semántica manual/automática del alta.
- `PUT /usuarios/{id}` no acepta contraseñas: cambiar la clave de un tercero es exclusivo de la
  regeneración de temporal.
- `GET /usuarios` requiere GESTION; el filtro `activo=all` es válido.
- Todas las rutas exigen `RequiereGestion` y quedan bloqueadas por `DebeCambiarPassword`
  mientras la temporal esté vigente.
