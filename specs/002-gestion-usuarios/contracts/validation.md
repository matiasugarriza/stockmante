# Validation Schemas — Gestión de usuarios

**Backend**: `serde` + validación en servicio → `AppError::Validacion` (422). **Frontend**: Zod en
`frontend/src/lib/schemas/usuarios.ts`. El backend es la autoridad final; el frontend duplica por UX.

> La política de contraseñas y el contrato base están en
> [001/contracts/validation.md](../../001-autenticacion-sesion/contracts/validation.md).

## Constantes

| Regla | Valor |
|-------|--------|
| Nombre de usuario | 1–120, `trim`, no vacío |
| Email | ≤255, exactamente una `@`, partes no vacías, sin whitespace, sin puntos consecutivos ni en bordes de usuario/dominio; se normaliza a minúsculas |
| Rol | `ADMIN`, `GERENTE`, `OPERARIO`, `CAJERO` |
| Familia | derivada del rol: `ADMIN|GERENTE → GESTION`; `OPERARIO|CAJERO → OPERACION` |
| Paginación | `pagina >= 1`; `por_pagina` 1–100 (default 20) |
| Password temporal | 12 caracteres, CSPRNG (propiedad de 001) |

## Zod (referencia frontend)

```ts
// frontend/src/lib/schemas/usuarios.ts
export const RolSchema = z.enum(['ADMIN', 'GERENTE', 'OPERARIO', 'CAJERO']);

export const UsuarioCreateSchema = z.object({
  nombre: z.string().trim().min(1).max(120),
  email: z.string().trim().toLowerCase().email().max(255),
  rol: RolSchema,
});

export const UsuarioUpdateSchema = UsuarioCreateSchema.partial();
```

## Casos clave a probar (test design)

| ID | Capa | Caso | Esperado |
|----|------|------|----------|
| T-AUTH-04 | servicio | desactivar usuario con sesión | siguientes requests 401 |
| T-USER-01 | servicio | segundo usuario mismo email | 409 `EmailDuplicado` |
| T-USER-01b | handler | email con múltiples `@` u otros formatos inválidos en POST/PUT | 422 y nada persistido |
| T-USER-02 | servicio | desactivar/degradar último `GESTION` | 409 `UltimoAdministrador` |
| T-USER-03 | handler | Operario `POST /usuarios` | 403 `sin_permisos` |
| T-USER-04 | handler | gestión con temporal pendiente | 403 `cambio_password_requerido`, sin cambios persistidos |
| T-USER-05 | handler | paginación extrema / overflow de offset | 422 |
| T-FE-USER-01 | vitest | Zod de alta inválido | errores por campo |

## Reglas de testing (constitución)

- RED→GREEN en reglas de dominio.
- Emails únicos por test (UUID) para permitir ejecución paralela.
- `cargo sqlx prepare --check` en verde al cerrar cada tarea que toque queries.
