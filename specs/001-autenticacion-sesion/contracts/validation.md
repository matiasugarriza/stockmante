# Validation Schemas — Base y autenticación

**Backend**: `serde` + `validator` (o `garde`) en DTOs → `AppError::Validacion` (422). **Frontend**: Zod en `frontend/src/lib/schemas/*.ts`, importado por forms y `lib/api`. El backend es la autoridad final; el frontend duplica por UX.

> Reglas transversales (sobre de error, status, Zod por capa) y las de cada caso de uso viven
> en el `contracts/validation.md` de su spec.

## Constantes de autenticación

| Regla | Valor |
|-------|--------|
| Password | ≥8, ≥1 mayúscula, ≥1 dígito; no se compara con lista de peores |
| Password temporal | 12 caracteres, generada con CSPRNG del sistema |

## Zod (referencia frontend)

```ts
// frontend/src/lib/schemas/auth.ts
export const LoginSchema = z.object({
  email: z.string().email().max(255),
  password: z.string().min(1),
});
export const CambioPasswordSchema = z.object({
  password_actual: z.string().min(1),
  password_nueva: z.string().min(8).regex(/[A-Z]/).regex(/[0-9]/),
});
```

## Casos clave a probar (test design)

| ID | Capa | Caso | Esperado |
|----|------|------|----------|
| T-AUTH-01 | servicio | login password incorrecta | Err CredencialesInvalidas, mismo mensaje que email inexistente |
| T-AUTH-02 | servicio | login cuenta inactiva | mismo mensaje genérico 401 |
| T-AUTH-03 | servicio | segundo login | sesión anterior revocada; request con cookie vieja → 401 |
| T-AUTH-05 | servicio | cambiar password temporal | password_temporal=false; vieja no sirve |

> T-AUTH-04 (desactivar usuario con sesión) pertenece a
> [002](../../002-gestion-usuarios/contracts/validation.md).

## Reglas de testing (constitución)

- RED→GREEN en reglas de dominio.
- Sin dependencias ocultas de reloj: inyectar `TraitReloj` para expiración de sesión en tests.
- `cargo sqlx prepare --check` en verde al cerrar cada tarea que toque queries.
