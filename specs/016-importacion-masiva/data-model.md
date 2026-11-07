# Data Model — Importación masiva de productos y stock (016)

**Spec**: [spec.md](./spec.md) | **Plan**: [plan.md](./plan.md) | **Esquema verificado**:
`migrations/0002_categorias_productos.sql`, `migrations/0009_stock_movimientos.sql`.

No hay migración nueva. La importación escribe en entidades existentes y expone DTOs de entrada
(CSV) y salida (resumen/errores).

## Entidades reutilizadas

### `productos` (destino de cada fila válida)

| Columna | Tipo | Regla relevante |
|---------|------|-----------------|
| `id` | INT identity | Generado por la BD |
| `nombre` | VARCHAR(160) NOT NULL | `validar_nombre_producto`: 1–160 tras `trim` |
| `sku` | VARCHAR(30) NOT NULL UNIQUE | `validar_sku`: 3–30, mayúsculas/dígitos/guiones, normaliza a mayúsculas |
| `codigo_barras` | VARCHAR(64) NULL UNIQUE NULLS NOT DISTINCT | `validar_codigo_barras`: 8–64 alfanuméricos; vacío → NULL |
| `categoria_id` | INT NOT NULL → `categorias.id` | Debe existir y estar activa; el CSV trae el nombre |
| `unidad_medida` | VARCHAR(20) NOT NULL | `validar_unidad`: UNIDAD, KG, METRO, CAJA |
| `costo` | NUMERIC(12,2) NOT NULL CHECK ≥ 0 | `validar_monto` |
| `precio_venta` | NUMERIC(12,2) NOT NULL CHECK ≥ 0 | `validar_monto` |
| `stock_minimo` | INT NOT NULL DEFAULT 0 CHECK ≥ 0 | `validar_stock_minimo` |
| `activo` | BOOLEAN NOT NULL DEFAULT TRUE | Alta como activo |
| `stock_actual` | INT NOT NULL DEFAULT 0 CHECK ≥ 0 (0009) | Cache; se actualiza en la misma transacción |
| `creado_en` / `actualizado_en` | TIMESTAMPTZ | `now()` |

### `stock_por_deposito` (fuente de verdad del stock)

| Columna | Tipo | Regla |
|---------|------|-------|
| `producto_id` | INT NOT NULL | Producto recién creado |
| `deposito_id` | INT NOT NULL | Depósito indicado o "Principal" |
| `ubicacion_id` | INT NULL | `NULL` (la importación no reparte por ubicación) |
| `cantidad` | INT NOT NULL CHECK ≥ 0 | Se inserta/actualiza con el stock inicial |
| `actualizado_en` | TIMESTAMPTZ | `now()` |

Restricción `uq_stock_producto_deposito_ubicacion` (NULLS NOT DISTINCT) guía el `ON CONFLICT`.

### `movimientos` + `movimiento_lineas` (stock inicial como entrada)

| Tabla | Campos usados | Regla |
|-------|---------------|-------|
| `movimientos` | `tipo='ENTRADA'`, `deposito_id`, `motivo='IMPORTACION'`, `observacion`, `usuario_id` | `usuario_id` sale de la sesión, nunca del CSV |
| `movimiento_lineas` | `producto_id`, `ubicacion_id=NULL`, `cantidad_delta=+stock_inicial`, `stock_resultante` | `cantidad_delta <> 0` (solo se crea si stock > 0); `stock_resultante ≥ 0` |

### Catálogos de apoyo (solo lectura)

- `categorias(id, nombre, activo)`: se resuelve por `nombre` (`trim`, sin distinguir mayúsculas).
- `depositos(id, nombre, activo)`: se resuelve por `nombre`; el default es "Principal".

## DTOs

### `ModoImportacion` (entrada, query param)

```text
validar | aplicar   (default: validar)
```

### Fila CSV canónica (`FilaCsv`, uso interno)

| Columna | Obligatoria | Tipo destino |
|---------|-------------|--------------|
| `nombre` | Sí | String |
| `sku` | Sí | String normalizado |
| `categoria` | Sí | Nombre de categoría existente |
| `unidad_medida` | Sí | UNIDAD / KG / METRO / CAJA |
| `costo` | Sí | Decimal ≥ 0, ≤ 2 decimales |
| `precio_venta` | Sí | Decimal ≥ 0, ≤ 2 decimales |
| `stock_minimo` | Sí | Int ≥ 0 |
| `codigo_barras` | No | 8–64 alfanumérico |
| `stock_inicial` | No (vacío = 0) | Int ≥ 0 |
| `deposito` | No (vacío = "Principal") | Nombre de depósito existente |

### `DetalleImportacion` (error por celda)

```json
{ "fila": 3, "columna": "sku", "mensaje": "El SKU 'YERBA-1K' ya existe." }
```

- `fila`: número de fila del archivo, 1 = encabezado; la primera fila de datos es 2.
- `columna`: nombre canónico de la columna (`sku`, `costo`, `deposito`, …) o `archivo`/`encabezado`
  para errores globales.
- `mensaje`: texto accionable, sin emojis (P-III).

### `ResumenImportacion` (200 en validar, 201 en aplicar)

| Campo | Tipo | Significado |
|-------|------|-------------|
| `filas_leidas` | usize | Filas de datos leídas |
| `validas` | usize | Filas sin errores |
| `con_error` | usize | Filas con al menos un error (distintas) |
| `errores` | DetalleImportacion[] | Detalle por celda |
| `creados` | usize | Productos creados (0 en validar) |
| `aplicado` | bool | `false` en validar, `true` en aplicar exitoso |

### `ErrorImportacion` (422)

Sobre estándar del proyecto ampliado:

```json
{
  "error": {
    "codigo": "importacion_invalida",
    "mensaje": "La importación tiene 2 filas con errores.",
    "detalles": [
      { "fila": 3, "columna": "sku", "mensaje": "El SKU 'YERBA-1K' ya existe." }
    ]
  }
}
```

## Reglas de validación por fila

| Condición | Columna | Mensaje |
|-----------|---------|---------|
| `nombre` vacío | `nombre` | `El nombre es obligatorio.` |
| `nombre` > 160 | `nombre` | `El nombre del producto no puede superar los 160 caracteres.` |
| `sku` vacío | `sku` | `El SKU es obligatorio.` |
| `sku` inválido | `sku` | `El SKU debe tener entre 3 y 30 caracteres en mayúsculas, dígitos y guiones, y empezar con letra o dígito.` |
| `sku` existente en BD | `sku` | `El SKU 'X' ya existe.` |
| `sku` repetido en el archivo | `sku` | `El SKU 'X' está repetido en el archivo (filas N y M).` |
| `categoria` vacía | `categoria` | `La categoría es obligatoria.` |
| `categoria` inexistente/inactiva | `categoria` | `La categoría 'X' no existe.` |
| `unidad_medida` fuera de la lista | `unidad_medida` | `La unidad de medida debe ser UNIDAD, KG, METRO o CAJA.` |
| `costo` vacío / no numérico / negativo | `costo` | `El costo es obligatorio.` / `Costo inválido.` / `El costo debe ser un monto mayor o igual a 0, con hasta 2 decimales y menor a 10.000.000.000.` |
| `precio_venta` vacío / no numérico / negativo | `precio_venta` | `El precio de venta es obligatorio.` / `Precio inválido.` / mensaje de `validar_monto` |
| `stock_minimo` no entero o negativo | `stock_minimo` | `stock_minimo inválido.` / `El stock mínimo debe ser un entero mayor o igual a 0.` |
| `stock_inicial` no entero o negativo | `stock_inicial` | `stock_inicial inválido.` / `El stock inicial debe ser un entero mayor o igual a 0.` |
| `codigo_barras` inválido | `codigo_barras` | `El código de barras debe tener entre 8 y 64 caracteres alfanuméricos.` |
| `codigo_barras` duplicado en BD o archivo | `codigo_barras` | `El código de barras 'X' ya existe.` / `El código de barras 'X' está repetido en el archivo (filas N y M).` |
| `deposito` indicado inexistente (solo si stock > 0) | `deposito` | `El depósito 'X' no existe.` |
| sin `deposito` y sin "Principal" (solo si stock > 0) | `deposito` | `No se encontró el depósito Principal.` |

Los errores de fila **no** abortan `validar`; sí impiden `aplicar`.

## Flujo de estados

La importación no persiste un estado propio: es una operación de un solo paso.

```text
archivo recibido
   │
   ├─ modo=validar → parsear + validar → 200 { resumen + errores }   (sin escrituras)
   │
   └─ modo=aplicar → parsear + validar
                        ├─ con errores → 422 importacion_invalida    (sin escrituras)
                        └─ sin errores → BEGIN → productos + stock + movimientos → COMMIT → 201
```

## Invariantes

1. `stock_por_deposito` es la fuente de verdad; `productos.stock_actual` se actualiza en la misma
   transacción (P-V).
2. Ningún stock puede quedar negativo (`CHECK` en BD y valor calculado ≥ 0).
3. Un SKU o código de barras existente nunca se actualiza: es error de fila.
4. Categorías y depósitos no se crean durante la importación.
5. `aplicar` es todo-o-nada: si una fila falla, no se confirma ninguna.
