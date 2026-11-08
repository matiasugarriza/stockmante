# Data Model: Escaneo y generación de códigos de barras/SKU

**Feature**: [spec.md](./spec.md) · **Research**: [research.md](./research.md)

**Delta de esquema**: ninguno. No hay migración nueva (ver
[research.md § Verificación de esquema](./research.md#verificación-de-esquema-p-v)).

## Entidades persistidas (sin cambios de columnas)

### `productos` (003)

| Campo | Tipo | Regla en 020 |
|---|---|---|
| `id` | `INT` | base del EAN-13 automático: `"20"` + `id` con ceros a 10 dígitos + verificador |
| `sku` | `VARCHAR(30)` | se codifica en Code 128/QR cuando no hay `codigo_barras` (FR-159) |
| `codigo_barras` | `VARCHAR(64) NULL UNIQUE` | 8–64 alfanuméricos (003) **y**, si son 13 dígitos numéricos, dígito verificador EAN-13 correcto (FR-143) |
| `activo` | `BOOLEAN` | una lectura de un inactivo se informa como «dado de baja» (FR-141) |

**Reglas de validación de `codigo_barras`** (en `validar_codigo_barras`, aplica a alta, ficha,
endpoint nuevo e importación CSV):

1. `trim`; vacío → `NULL`.
2. 8–64 caracteres ASCII alfanuméricos (sin cambios).
3. **Nuevo**: 13 caracteres, todos dígitos → el 13.º DEBE ser el dígito verificador EAN-13 de los
   12 primeros. Si no, 422: «El código 7791234567890 no es un EAN-13 válido: el dígito verificador
   debería ser 8. Corregí el último dígito o ingresá solo los 12 primeros para calcularlo.»
4. Unicidad entre todos los productos (activos e inactivos) por `productos_codigo_barras_key`.

**Dígito verificador EAN-13**: con los 12 primeros dígitos `d1…d12`, `s = Σ dᵢ · (1 si i impar,
3 si i par)`; verificador = `(10 − s mod 10) mod 10`.

### `auditoria` (006)

Sin cambios de columnas. Valores nuevos de `accion` con `tabla_ref = 'productos'`:

| `accion` | Cuándo |
|---|---|
| `ASIGNAR_CODIGO_BARRAS` | el producto no tenía código y se le asignó uno por `POST …/codigo-barras` |
| `REEMPLAZAR_CODIGO_BARRAS` | el producto tenía código y se reemplazó con `reemplazar: true` |

Los cambios de código hechos desde la ficha (`PUT /api/productos/{id}`) siguen auditándose como
`ACTUALIZAR` (003/006).

## Entidades transitorias (solo frontend, no se persisten)

### Lectura

| Campo | Tipo | Notas |
|---|---|---|
| `valor` | `string` | ya pasado por `normalizarLectura` (sin espacios ni caracteres de control) |
| `formato` | `'ean_13' \| 'ean_8' \| 'upc_a' \| 'code_128' \| 'code_39' \| 'qr_code' \| 'teclado'` | `teclado` = lector físico o entrada manual |
| `resultado` | `'encontrado' \| 'inactivo' \| 'no_encontrado' \| 'error_red'` | de `resolverLectura` |
| `producto` | `ProductoBusqueda?` | presente en `encontrado` e `inactivo` |

### Estado del escáner

```text
inactivo ──abrir──▶ solicitando ──permiso ok──▶ activo ──detección──▶ leido ──▶ inactivo
    ▲                    │                        │
    │                    └──error──▶ error(tipo) ◀┘ (cámara perdida)
    └──────── cancelar / salir de la pantalla / reintentar ────────────┘
```

`tipo` ∈ `inseguro`, `sin_soporte`, `permiso_denegado`, `sin_camara`, `ocupada`, `desconocido`.
En `leido`, `error` y al cancelar, las pistas de video se detienen (FR-153).

### Etiqueta

| Campo | Tipo | Regla |
|---|---|---|
| `formato` | `'ean_13' \| 'code_128' \| 'qr'` | `ean_13` solo si `es_ean13_valido(codigo_barras)` (FR-158) |
| `valorCodificado` | `string` | `codigo_barras` si existe; si no, `sku` (solo Code 128/QR) (FR-159) |
| `nombre` | `string` | recortado a 2 líneas en 50 mm |
| `sku` | `string` | siempre visible en texto |
| `tamano` | `50 × 30 mm` | fijo; vista previa a tamaño real (FR-154) |

**Opciones de formato según el producto**:

| Producto | EAN-13 | Code 128 | QR |
|---|---|---|---|
| Código EAN-13 válido | ✓ | ✓ (código) | ✓ (código) |
| Código interno alfanumérico | ✗ «no es EAN-13» | ✓ (código) | ✓ (código) |
| Sin código | ✗ «generá el código primero» | ✓ (SKU) | ✓ (SKU) |
