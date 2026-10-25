# Quickstart — Gestión de usuarios (002)

**Propósito**: validar el caso de uso US2 sobre la base ya instalada. El setup, las variables de
entorno y el bootstrap están en [001/quickstart.md](../001-autenticacion-sesion/quickstart.md).

## Prerrequisitos

- Base y US1 funcionando (001).
- Backend y frontend levantados según 001.
- Sesión con un usuario de familia GESTION.

## V3 — Alta de usuario y rol

1. Autenticarse como Administrador/Gerente y abrir `/admin/usuarios`.
2. Crear un usuario con nombre, correo nuevo y rol `OPERARIO`.
3. Esperado: 201/alta visible; la contraseña temporal llega por correo (en DEV se imprime en la
   consola del backend).
4. Cerrar sesión e iniciar sesión con ese usuario: se exige el cambio de contraseña temporal.
5. Al completarlo, el Operario no ve el menú de gestión y `POST /api/usuarios` responde 403.

## V5 — Último administrador

1. Con un único usuario de familia GESTION activo, intentar desactivarlo o degradarlo a
   `OPERARIO`.
2. Esperado: 409 `ultimo_administrador`; el usuario sigue activo y con su rol.
3. Crear un segundo gestor y repetir: ahora la operación se permite.

## Verificaciones puntuales

| Caso | Esperado |
|---|---|
| Alta con un correo ya registrado | 409 `email_duplicado` |
| Alta con `a@b@c.com`, `a@@b.com`, `a@b..com`, `.a@b.com` | 422 `validacion`, nada persistido |
| Desactivar un usuario con sesión abierta | Su siguiente request responde 401 |
| Reactivar un usuario desactivado | Puede volver a iniciar sesión |
| `POST /usuarios/{id}/password-temporal` | 204; la anterior deja de servir y las sesiones se revocan |
| Gestión con contraseña temporal pendiente | 403 `cambio_password_requerido` |

## V8 — Regresión de calidad

```bash
cd backend && cargo test && cargo sqlx prepare --check
cd frontend && npm test && npm run check && npm run build
```

Todos en verde; sin `unwrap`/`panic`/`expect` en rutas de producción.
