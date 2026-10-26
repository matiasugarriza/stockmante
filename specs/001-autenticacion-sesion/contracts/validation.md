# Validation Schemas — Base y autenticación

**Backend**: `serde` + `validator` (o `garde`) en DTOs → `AppError::Validacion` (422). **Frontend**: Zod en `frontend/src/lib/schemas/*.ts`, importado por forms y `lib/api`. El backend es la autoridad final; el frontend duplica por UX.

> Reglas transversales (sobre de error, status, Zod por capa) y las de cada caso de uso viven
> en el `contracts/validation.md` de su spec.

## Constantes de autenticación

| Regla | Valor |
|-------|--------|
| Password | ≥8, ≥1 mayúscula, ≥1 dígito; no se compara con lista de peores |
| Password temporal | 14 caracteres (mínimo 12), generada con CSPRNG del sistema; ≥1 mayúscula y ≥1 dígito |

## Zod (referencia frontend)

```ts
// frontend/src/lib/schemas/auth.ts
// SvelteKit 3 / Zod 4: usar z.email() (z.string().email() quedó deprecado).
export const LoginSchema = z.object({
  email: z.email().max(255),
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
| T-AUTH-04 | servicio/integración | desactivar usuario con sesión | siguientes requests 401 (verificado en 001: `test_sesion_unica.rs::desactivar_usuario_deja_sesiones_invalidas`; la baja lógica es de 002) |
| T-AUTH-05 | servicio | cambiar password temporal | password_temporal=false; vieja no sirve |
| T-AUTH-06 | extractor | temporal pendiente en endpoint autenticado | 403 `cambio_password_requerido`, salvo `/auth/sesion` y `/auth/cambiar-password` (T028) |
| T-AUTH-07 | logging | temporal en logs | el stub por defecto y los logs de producción nunca incluyen la credencial (T013) |
| T-AUTH-08 | servicio | expiración absoluta | a las 8 h del login la sesión deja de valer aunque haya actividad; no se renueva (T026) |
| T-AUTH-09 | integración | dos logins concurrentes | queda una sola sesión vigente; la cookie perdedora responde 401 (T027) |

> T-AUTH-04 se mantiene también en la validación de 002 para la operación de baja;
> la **verificación de invalidación** corresponde a 001 (SC-004).

## Reglas de testing (constitución)

- RED→GREEN en reglas de dominio.
- Sin dependencias ocultas de reloj: inyectar `TraitReloj` para expiración de sesión en tests.
- `cargo sqlx prepare --check` en verde al cerrar cada tarea que toque queries.
