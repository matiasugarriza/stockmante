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
| Password temporal automática | 14 caracteres (`TEMP_LEN`), CSPRNG, con ≥1 mayúscula y ≥1 dígito (genera el backend) |
| Password manual (alta/regenerar) | ausente ⇒ automática; presente ⇒ política 001: ≥8, ≥1 mayúscula, ≥1 dígito; fuera de política ⇒ 422 |

## Zod (referencia frontend)

```ts
// frontend/src/lib/schemas/usuarios.ts
export const RolSchema = z.enum(['ADMIN', 'GERENTE', 'OPERARIO', 'CAJERO']);

// Política 001 aplicada a la contraseña manual (opcional: ausente ⇒ automática)
const PoliticaPasswordSchema = z
  .string()
  .min(8, 'Mínimo 8 caracteres')
  .regex(/[A-Z]/, 'Debe tener una mayúscula')
  .regex(/[0-9]/, 'Debe tener un dígito');

export const CrearUsuarioSchema = z.object({
  nombre: z.string().trim().min(1).max(120),
  email: z.string().trim().toLowerCase().email().max(255),
  rol: RolSchema,
  password: PoliticaPasswordSchema.optional(),
});

// PUT nunca envía password (exclusivo de la regeneración de temporal)
export const ActualizarUsuarioSchema = CrearUsuarioSchema
  .omit({ password: true })
  .partial();

// Respuestas 201/200 con credenciales para la modal (FR-008b)
export const UsuarioCreadoSchema = z.object({
  usuario: UsuarioSchema,
  password_temporal: z.string(),
});
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
| T-USER-06 | service/handler | `POST /usuarios` con `password` manual fuera de política (`pass1`) | 422 y nada persistido |
| T-USER-07 | service/handler | `POST /usuarios` sin `password` (automática) | 201 con `password_temporal` de 14 chars, ≥1 mayúscula, ≥1 dígito |
| T-USER-08 | handler | `POST /usuarios/{id}/password-temporal` con `{ password: "Manual99" }` y sin body | 200 con `password_temporal`; la clave anterior deja de servir |
| T-FE-USER-01 | vitest | Zod de alta inválido | errores por campo |
| T-FE-USER-02 | vitest | Zod con `password` débil (`pass1`) | error por campo; sin `password` válido (automática) |

## Reglas de testing (constitución)

- RED→GREEN en reglas de dominio.
- Emails únicos por test (UUID) para permitir ejecución paralela.
- `cargo sqlx prepare --check` en verde al cerrar cada tarea que toque queries.
