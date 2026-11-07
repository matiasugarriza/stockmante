# Contratos de API — Importación masiva de productos y stock (016)

Base: `/api`. Todas las rutas exigen cookie de sesión (`stockmante_sesion`). Permisos según FR-065 y
la matriz de 006:

| Operación | `OPERACION` (CAJERO/OPERARIO) | `GESTION` (ADMIN/GERENTE) |
|---|---|---|
| `POST /api/importaciones/productos` | ❌ 403 `sin_permisos` | ✅ |

Formato de error: `{ "error": { "codigo": "<codigo>", "mensaje": "<texto>", "detalles": [...] } }`.
`detalles` solo aparece en `importacion_invalida`; en el resto de errores se mantiene el sobre
existente sin ese campo.

## POST `/api/importaciones/productos?modo=validar|aplicar`

- **modo** (query, opcional): `validar` (default) o `aplicar`. Otro valor → 422 `validacion`.
- **Body**: `multipart/form-data` con el campo `archivo` (obligatorio). Se ignoran campos extra.
- **Archivo**: CSV UTF-8, hasta 5 MB y 5.000 filas de datos. Se tolera BOM UTF-8.
- **Nombre del archivo**: irrelevante; no se valida la extensión.
- **Límite de cuerpo**: la ruta admite hasta 16 MB para poder responder 422 con el mensaje de
  negocio cuando el archivo supera los 5 MB.

### Permisos

- Sin sesión → 401 `no_autenticado`.
- `OPERACION` → 403 `sin_permisos` con `"No tenés permiso para realizar esta acción"`.

### Formato CSV

La primera línea es el encabezado. Nombres de columna sin distinguir mayúsculas y con `trim`;
columnas desconocidas se ignoran.

| Columna | Obligatoria | Regla |
|---------|-------------|-------|
| `nombre` | Sí | 1–160 caracteres |
| `sku` | Sí | 3–30; mayúsculas, dígitos y guiones; se normaliza a mayúsculas |
| `categoria` | Sí | Nombre de una categoría existente y activa |
| `unidad_medida` | Sí | `UNIDAD`, `KG`, `METRO` o `CAJA` |
| `costo` | Sí | Monto ≥ 0, hasta 2 decimales |
| `precio_venta` | Sí | Monto ≥ 0, hasta 2 decimales |
| `stock_minimo` | Sí | Entero ≥ 0 |
| `codigo_barras` | No | 8–64 alfanuméricos; vacío = sin código |
| `stock_inicial` | No | Entero ≥ 0; vacío = 0 |
| `deposito` | No | Nombre de depósito existente; vacío = "Principal" |

**Dialecto**: el delimitador se detecta por el encabezado (`;` si aparece más que `,`; si no, `,`).
El decimal se interpreta como `,` cuando el delimitador es `;`, y como `.` cuando es `,`. Comillas
dobles RFC 4180. No se admiten separadores de miles.

### 200 — `modo=validar`

No escribe nada. Devuelve el resumen y los errores por celda.

```json
{
  "filas_leidas": 3,
  "validas": 2,
  "con_error": 1,
  "errores": [
    { "fila": 4, "columna": "costo", "mensaje": "Costo inválido." }
  ],
  "creados": 0,
  "aplicado": false
}
```

### 201 — `modo=aplicar` sin errores

Crea productos, stock inicial y movimientos en una única transacción.

```json
{
  "filas_leidas": 3,
  "validas": 3,
  "con_error": 0,
  "errores": [],
  "creados": 3,
  "aplicado": true
}
```

### 422 — `modo=aplicar` con errores

No escribe ningún producto. `detalles` es obligatorio y trae una entrada por celda inválida.

```json
{
  "error": {
    "codigo": "importacion_invalida",
    "mensaje": "La importación tiene 1 fila con errores.",
    "detalles": [
      {
        "fila": 4,
        "columna": "sku",
        "mensaje": "El SKU 'YERBA-1K' ya existe."
      }
    ]
  }
}
```

Casos globales (también 422 `importacion_invalida`, con `fila` y `columna` indicativas):

| Caso | `fila` | `columna` | `mensaje` |
|------|--------|-----------|-----------|
| Archivo > 5 MB o > 5.000 filas | 0 | `archivo` | `El archivo supera el límite de 5.000 filas o 5 MB.` |
| Archivo sin filas de datos | 0 | `archivo` | `El archivo no contiene filas para importar.` |
| Encabezado sin una columna obligatoria | 1 | `<columna>` | `Falta la columna 'sku'.` |
| CSV ilegible / comillas mal cerradas | 0 | `archivo` | `No se pudo leer el CSV. Revisá el formato del archivo.` |
| Fila con más celdas que el encabezado | N | `archivo` | `La fila tiene más columnas que el encabezado.` |
| Falta el campo `archivo` | 0 | `archivo` | `Falta el archivo.` |

`modo` inválido sigue el contrato general: 422 `validacion` con
`"modo debe ser validar o aplicar."`.

### Errores de fila esperados

| Condición | Columna | Mensaje |
|-----------|---------|---------|
| SKU ya existente | `sku` | `El SKU 'YERBA-1K' ya existe.` |
| SKU repetido en el archivo | `sku` | `El SKU 'YERBA-1K' está repetido en el archivo (filas 3 y 8).` |
| Categoría inexistente | `categoria` | `La categoría 'Almacén' no existe.` |
| Depósito inexistente (stock > 0) | `deposito` | `El depósito 'Depósito 2' no existe.` |
| Sin "Principal" y sin depósito (stock > 0) | `deposito` | `No se encontró el depósito Principal.` |
| Código de barras duplicado | `codigo_barras` | `El código de barras '7791234567890' ya existe.` |

### Notas de contrato

- Los importes viajan como número JSON (`serde-float`), no como string.
- `errores` puede traer más de una entrada por fila (una por columna inválida).
- El dry-run devuelve 200 aunque haya errores; solo `aplicar` usa 422.
- La importación nunca crea categorías ni depósitos ni actualiza productos existentes.
