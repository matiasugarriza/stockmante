# Validation Schemas — Catálogo de productos y categorías

**Backend**: validación explícita en `services/productos.rs` y `services/categorias.rs` →
`AppError::Validacion` (422). **Frontend**: Zod en `frontend/src/lib/schemas/`. El backend es la
autoridad final; el frontend duplica por UX.

> Password, email y paginación base están en
> [001/contracts/validation.md](../../001-autenticacion-sesion/contracts/validation.md).

## Constantes

| Regla | Valor |
|-------|-------|
| SKU | `^[A-Z0-9][A-Z0-9-]{2,29}$` **tras** `trim` + mayúsculas; 3–30 caracteres |
| Código de barras | 8–64, `[0-9A-Za-z]`, opcional; vacío/blanco → `NULL` |
| Nombre producto | 1–160 (trim) |
| Nombre categoría | 1–80 (trim) |
| Unidad medida | `UNIDAD`, `KG`, `CAJA` |
| Costo / precio | ≥ 0, máximo 2 decimales, < `10^10` |
| Stock mínimo | entero ≥ 0 |
| `categoria_id` | entero > 0 y existente |
| Paginación | `pagina >= 1`; `por_pagina` 1–100 (default 20) |

## Zod (referencia frontend)

```ts
// frontend/src/lib/schemas/productos.ts
export const SkuSchema = z.string().trim().toUpperCase()
  .regex(/^[A-Z0-9][A-Z0-9-]{2,29}$/);

export const ProductoCreateSchema = z.object({
  nombre: z.string().trim().min(1).max(160),
  sku: SkuSchema,
  codigo_barras: z.string().trim().regex(/^[0-9A-Za-z]{8,64}$/).nullish(),
  categoria_id: z.number().int().positive(),
  unidad_medida: z.enum(['UNIDAD', 'KG', 'CAJA']),
  costo: z.number().nonnegative().multipleOf(0.01),
  precio_venta: z.number().nonnegative().multipleOf(0.01),
  stock_minimo: z.number().int().nonnegative(),
});
```

```ts
// frontend/src/lib/schemas/categorias.ts
export const CategoriaCreateSchema = z.object({
  nombre: z.string().trim().min(1).max(80),
});
export const CategoriaUpdateSchema = CategoriaCreateSchema.partial().extend({
  activo: z.boolean().optional(),
});
```

## Rust (referencia backend — misma semántica)

La validación es explícita, no con `validator`, para controlar el mensaje en español y el orden
de los chequeos:

```rust
pub fn normalizar_sku(sku: &str) -> String { sku.trim().to_uppercase() }

pub fn validar_sku(sku: &str) -> Result<String, AppError> {
    // 3..=30; primer char [A-Z0-9]; resto [A-Z0-9-]
}
```

`validar_codigo_barras(None | "")` → `Ok(None)`; `validar_monto` rechaza negativos,
`scale() > 2` y valores ≥ `10_000_000_000`.

## Casos clave a probar (test design)

| ID | Capa | Caso | Esperado |
|----|------|------|----------|
| T-PROD-01 | servicio | SKU `sku-001` y `SKU-001` | normaliza; 409 duplicado |
| T-PROD-02 | BD | código de barras repetido | 409 / unique violation → mapeo 409 |
| T-PROD-03 | servicio | SKU mal formado (`bad!`, largo fuera de 3–30) | 422 `Validacion` |
| T-PROD-04 | handler | DELETE producto con asignación | 409 `ProductoConReferencias` |
| T-PROD-05 | búsqueda | SKU exacto inactivo | no aparece en caja; sí en gestión (004) |
| T-PROD-06 | BD | varios productos con `codigo_barras = NULL` | permitido (regresión de 0004) |
| T-PROD-07 | servicio | `codigo_barras: ""` en PUT | limpia el código (`NULL`) |
| T-CAT-01 | servicio | categoría duplicada | 409 `CategoriaDuplicada` |
| T-CAT-02 | servicio | DELETE categoría con productos | 409 `CategoriaConProductos` |
| T-FE-01 | vitest | Zod `ProductoCreate` inválido | errores por campo |
| T-FE-02 | vitest | cliente api 409 | lanza `ApiError` con `codigo` |

> **`bad` no es un SKU inválido**: normaliza a `BAD`, que cumple el regex. El caso inválido es
> `bad!`. El texto original de T032 decía `bad`; fue un error de la tarea, no un requisito.

## Reglas de testing (constitución)

- RED→GREEN en reglas de dominio (T-PROD-01..03, T-CAT-*).
- Claves únicas derivadas de UUID para permitir tests en paralelo.
- `cargo sqlx prepare --check` en verde al cerrar cada tarea que toque queries.
