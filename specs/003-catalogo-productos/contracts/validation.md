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
| Nombre producto | 1–160 (trim, no vacío) |
| Nombre categoría | 1–80 (trim, no vacío); único global sobre `lower(btrim(nombre))` |
| Descripción categoría | 0–255; vacío/blanco → `NULL` |
| Unidad medida | `UNIDAD`, `KG`, `METRO`, `CAJA` |
| Costo / precio | ≥ 0, máximo 2 decimales, < `10^10` |
| Stock mínimo | entero ≥ 0 |
| `categoria_id` | entero > 0, existente y **activo** |
| Filtro `q` | 0–160 tras `trim`; vacío = sin filtro |
| Filtro `categoria_id` | entero > 0 |
| Filtro `activo` | `true` \| `false` \| `all` (default `true` en catálogo) |
| Paginación productos | `pagina >= 1`; `por_pagina` 1–100 (default 20) |
| `activo` en `PUT` | prohibido → 422 |

## Reglas de normalización

| Campo | Normalización | Efecto |
|---|---|---|
| `sku` | trim + mayúsculas | `" yerba-1k "` → `YERBA-1K`; la unicidad se evalúa ya normalizado |
| `codigo_barras` | trim | `""` o solo espacios → `NULL`; `PUT` con `""` limpia |
| `nombre` (categoría) | trim | la unicidad compara sin mayúsculas ni espacios de extremo |
| `descripcion` | trim | `""` o solo espacios → `NULL` |
| `nombre` (producto) | trim | se persiste recortado |

La comparación de nombres de categoría **no** pliega acentos: «Almacén» y «Almacen» son distintos.
El espejo Zod no recorta `codigo_barras`: el frontend rechaza un valor con espacios de extremo y el
backend, que es la autoridad, lo recorta y lo valida.

## Zod (referencia frontend)

```ts
// frontend/src/lib/schemas/productos.ts (espejo real; el backend es la autoridad)
export const UnidadMedidaSchema = z.enum(['UNIDAD', 'KG', 'METRO', 'CAJA']);

const MontoSchema = z.number().nonnegative().multipleOf(0.01).lt(10_000_000_000);

const CodigoBarrasSchema = z
  .union([z.literal(''), z.string().regex(/^[0-9A-Za-z]{8,64}$/)])
  .nullish();

export const CrearProductoSchema = z.object({
  nombre: z.string().trim().min(1).max(160),
  sku: z.string().trim().toUpperCase().regex(/^[A-Z0-9][A-Z0-9-]{2,29}$/),
  codigo_barras: CodigoBarrasSchema,
  categoria_id: z.number().int().positive(),
  unidad_medida: UnidadMedidaSchema,
  costo: MontoSchema,
  precio_venta: MontoSchema,
  stock_minimo: z.number().int().nonnegative(),
});

export const ActualizarProductoSchema = CrearProductoSchema.partial()
  .refine((v) => Object.keys(v).length > 0, 'Indicá al menos un campo para actualizar.');
```

```ts
// frontend/src/lib/schemas/categorias.ts
export const CategoriaCreateSchema = z.object({
  nombre: z.string().trim().min(1).max(80),
  descripcion: z.string().trim().max(255).nullish(),
});

export const CategoriaUpdateSchema = CategoriaCreateSchema.partial()
  .refine((v) => Object.keys(v).length > 0, 'Indicá al menos un campo para actualizar.');
```

Ningún schema de actualización incluye `activo`: el estado viaja por `/desactivar` y `/reactivar`.

## Rust (referencia backend — misma semántica)

La validación es explícita, no con `validator`, para controlar el mensaje en español y el orden de
los chequeos:

```rust
pub fn normalizar_sku(sku: &str) -> String { sku.trim().to_uppercase() }

pub fn validar_sku(sku: &str) -> Result<String, AppError> {
    // 3..=30; primer char [A-Z0-9]; resto [A-Z0-9-]
}

pub fn validar_codigo_barras(valor: Option<&str>) -> Result<Option<String>, AppError> {
    // trim; vacío/blanco → None; 8..=64 alfanumérico → Some
}

pub fn normalizar_descripcion(valor: Option<&str>) -> Option<String> {
    // trim; vacío → None
}
```

`validar_monto` rechaza negativos, `scale() > 2` y valores ≥ `10_000_000_000`.
`validar_unidad` compara contra la lista cerrada y devuelve 422 con las opciones válidas.

## Casos clave a probar (test design)

| ID | Capa | Caso | Esperado |
|----|------|------|----------|
| T-PROD-01 | servicio | SKU `sku-001` y `SKU-001` | normaliza; 409 `sku_duplicado` |
| T-PROD-02 | BD | código de barras repetido | 409 `codigo_barras_duplicado` |
| T-PROD-03 | servicio | SKU mal formado (`bad!`, 2 o 31 caracteres, guion inicial) | 422 `validacion` |
| T-PROD-04 | handler | `DELETE /productos/{id}` | 405; la fila y su `activo` no cambian |
| T-PROD-05 | servicio | `codigo_barras: ""` en `PUT` | limpia el código (`NULL`) |
| T-PROD-06 | BD | varios productos con `codigo_barras = NULL` | permitido |
| T-PROD-07 | servicio | `PUT` con `activo` o con `{}` | 422; fila sin cambios |
| T-PROD-08 | servicio | `PUT` hacia categoría inactiva o inexistente | 422 |
| T-PROD-09 | handler | `POST /{id}/desactivar` repetido | 204 las dos veces |
| T-PROD-10 | handler | `GET /productos` sin `activo` | solo activos |
| T-PROD-11 | handler | `q` + `categoria_id` + `activo` combinados | AND, orden estable |
| T-CAT-01 | servicio | `"  BEBIDAS "` existiendo `"Bebidas"` | 409 `categoria_duplicada` |
| T-CAT-02 | handler | `DELETE /categorias/{id}` | 405; la fila no cambia |
| T-CAT-03 | handler | desactivar categoría con productos | 204; los productos conservan `categoria_id` |
| T-CAT-04 | servicio | `PUT` con `descripcion: ""` | queda `NULL`; `nombre` intacto |
| T-CAT-05 | handler | `GET /categorias` sin `activo` | solo activas |
| T-REG-01 | handler | `GET /api/usuarios` sin `activo` tras centralizar `parsear_activo` | devuelve activos e inactivos |
| T-FE-01 | vitest | Zod `ProductoCreate` inválido | errores por campo |
| T-FE-02 | vitest | cliente api 409 | lanza `ApiError` con `codigo` |

> **`bad` no es un SKU inválido**: normaliza a `BAD`, que cumple el regex. El caso inválido es
> `bad!`.

## Reglas de testing (constitución)

- RED→GREEN en reglas de dominio (T-PROD-01..03, T-PROD-07..08, T-CAT-01, T-CAT-04). Desde 003 no
  hay waiver preventivo (W-001 está cerrado).
- Claves únicas derivadas de UUID para permitir tests en paralelo.
- `cargo sqlx prepare --check` en verde al cerrar cada tarea que toque queries.
- Toda baja lógica se prueba por su efecto observable: el listado por defecto deja de mostrarla y
  el detalle sigue devolviéndola.
