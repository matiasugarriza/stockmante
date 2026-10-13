# Validation Schemas — Módulo 1

**Backend**: `serde` + `validator` (o `garde`) en DTOs → `AppError::Validacion` (422). **Frontend**: Zod en `frontend/src/lib/schemas/*.ts`, importado por forms y `$lib/api`. El backend es la autoridad final; el frontend duplica por UX.

## Constantes compartidas (lógica de negocio)

| Regla | Valor |
|-------|--------|
| Email | formato RFC básico (no se exige TLD extraño) |
| Password | ≥8, ≥1 mayúscula, ≥1 dígito; no se compara con lista de peores |
| Password temporal | 12 caracteres, generada con CSPRNG del sistema |
| SKU | `^[A-Z0-9][A-Z0-9-]{2,29}$` tras normalizar a mayúsculas y sin espacios |
| Código de barras | 8–64, `[0-9A-Za-z]`, opcional |
| Nombre producto | 1–160 |
| Unidad medida | catálogo: `UNIDAD`, `KG`, `CAJA` (extensible) |
| Costo / precio | ≥0, máx 2 decimales |
| Stock mínimo | entero ≥0 |
| Búsqueda nombre | mínimo 2 caracteres |

## Zod (referencia frontend)

```ts
// frontend/src/lib/schemas/producto.ts
export const SkuSchema = z.string().trim().toUpperCase()
  .regex(/^[A-Z0-9][A-Z0-9-]{2,29}$/);

export const ProductoCreateSchema = z.object({
  nombre: z.string().trim().min(1).max(160),
  sku: SkuSchema,
  codigo_barras: z.string().trim().regex(/^[0-9A-Za-z]{8,64}$/).nullish(),
  categoria_id: z.number().int().positive(),
  unidad_medida: z.enum(['UNIDAD', 'KG', 'CAJA']),
  costo: z.number().nonnegative(),
  precio_venta: z.number().nonnegative(),
  stock_minimo: z.number().int().nonnegative(),
});
```

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

```ts
// frontend/src/lib/schemas/ubicacion.ts
export const UbicacionCreateSchema = z.object({
  deposito_id: z.number().int().positive(),
  padre_id: z.number().int().positive().nullish(),
  nombre: z.string().trim().min(1).max(80),
  tipo: z.enum(['PASILLO', 'ESTANTERIA', 'NIVEL', 'POSICION']),
});
```

## Rust (referencia backend — misma semántica)

```rust
#[derive(Debug, Deserialize, Validate)]
pub struct ProductoCreate {
    #[validate(length(min = 1, max = 160))]
    pub nombre: String,
    #[validate(regex = "path::SKU_RE")]
    pub sku: String, // servicio normaliza a mayúsculas antes de validar
    #[validate(regex = "path::CODIGO_BARRAS_RE")]
    pub codigo_barras: Option<String>,
    pub categoria_id: i32,
    pub unidad_medida: UnidadMedida,
    #[validate(range(min = 0))]
    pub costo: rust_decimal::Decimal,
    #[validate(range(min = 0))]
    pub precio_venta: rust_decimal::Decimal,
    #[validate(range(min = 0))]
    pub stock_minimo: i32,
}
```

## Casos clave a probar (test design)

| ID | Capa | Caso | Esperado |
|----|------|------|----------|
| T-AUTH-01 | servicio | login password incorrecta | Err CredencialesInvalidas, mismo mensaje que email inexistente |
| T-AUTH-02 | servicio | login cuenta inactiva | mismo mensaje genérico 401 |
| T-AUTH-03 | servicio | segundo login | sesión anterior revocada; request con cookie vieja → 401 |
| T-AUTH-04 | servicio | desactivar usuario con sesión | siguientes requests 401 |
| T-AUTH-05 | servicio | cambiar password temporal | password_temporal=false; vieja no sirve |
| T-USER-01 | servicio | segundo usuario mismo email | 409 EmailDuplicado |
| T-USER-02 | servicio | desactivar último GESTION | 409 UltimoAdministrador |
| T-USER-03 | handler | Operario POST /usuarios | 403 |
| T-PROD-01 | servicio | SKU `sku-001` y `SKU-001` | normaliza; 409 duplicado |
| T-PROD-02 | BD | código de barras repetido | 409 / unique violation → mapeo 409 |
| T-PROD-03 | servicio | SKU mal formado | 422 Validacion |
| T-PROD-04 | handler | DELETE producto con asignación | 409 ProductoConReferencias |
| T-PROD-05 | búsqueda | SKU exacto inactivo | no aparece en caja; sí en gestión |
| T-UBI-01 | servicio | ubicacion_huerfana deposito_id=999 | 422 |
| T-UBI-02 | servicio | hija con deposito ≠ padre.deposito | 422 |
| T-FE-01 | vitest | Zod ProductoCreate inválido | errors en campo |
| T-FE-02 | vitest | cliente api 409 | lanza error tipado con codigo |

## Reglas de testing (constitución)

- RED→GREEN en reglas de dominio (T-AUTH-*, T-USER-02, T-PROD-01..03, T-UBI-*).
- Sin dependencias ocultas de reloj: inyectar `TraitReloj` para expiración de sesión en tests.
- `cargo sqlx prepare --check` en verde al cerrar cada tarea que toque queries.
