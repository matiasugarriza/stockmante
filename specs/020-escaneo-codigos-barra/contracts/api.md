# Contrato de API: códigos de barras (020)

Base: `/api`. Autenticación por cookie de sesión (001). Errores con el sobre estándar
`{ "error": { "codigo": string, "mensaje": string } }` (P-III).

## Endpoints existentes reutilizados (sin cambios de contrato)

### `GET /api/productos/buscar` (004)

Usado por `resolverLectura` con coincidencia exacta:

1. `?q=<valor>&campo=codigo&incluir_inactivos=true&limite=2`
2. si no hay ítems: `?q=<valor>&campo=sku&incluir_inactivos=true&limite=2`

Cualquier rol autenticado (operación y gestión). `q` > 160 caracteres → 422 (el cliente no lo envía
y responde `no_encontrado`).

### `PUT /api/productos/{id}` (003) — cambio de comportamiento acotado

Contrato idéntico. Cambia solo la validación de `codigo_barras` (también en `POST /api/productos` y
en la importación CSV de 016): 13 dígitos numéricos con verificador incorrecto → **422**
`validacion` con el dígito esperado en el mensaje.

## Endpoint nuevo

### `POST /api/productos/{id}/codigo-barras`

Asigna o reemplaza el código de barras del producto con un EAN-13. **Solo gestión** (extractor
`RequiereGestion`). No escribe movimientos de stock.

**Request**

```json
{ "modo": "automatico", "reemplazar": false }
```

```json
{ "modo": "manual", "valor": "779123456789", "reemplazar": true }
```

| Campo | Tipo | Regla |
|---|---|---|
| `modo` | `"automatico" \| "manual"` | requerido |
| `valor` | `string` | requerido en `manual`: 12 dígitos (se calcula el verificador) o 13 (se valida); se ignora en `automatico` |
| `reemplazar` | `boolean` | default `false`; DEBE ser `true` si el producto ya tiene código |

**Respuestas**

| Código | `error.codigo` | Cuándo | Efecto |
|---|---|---|---|
| 200 | — | asignado o reemplazado; body = `Producto` (003) con el nuevo `codigo_barras` | 1 fila de auditoría |
| 401 | `no_autenticado` | sin sesión | ninguno |
| 403 | `sin_permisos` | rol de operación | ninguno |
| 404 | `recurso_no_encontrado` | el producto no existe | ninguno |
| 409 | `codigo_barras_existente` | ya tiene código y `reemplazar` ≠ `true`. Mensaje: «El producto ya tiene el código 7791234567898. Confirmá el reemplazo para asignar uno nuevo.» | ninguno |
| 409 | `codigo_barras_duplicado` | el EAN-13 ya pertenece a otro producto. Mensaje: «Ya existe un producto con ese código de barras. Generá otro o ingresá 12 dígitos distintos.» | ninguno |
| 422 | `validacion` | `modo` inválido, `valor` ausente, no numérico, largo ≠ 12/13, o verificador incorrecto | ninguno |

**Semántica**

- Transacción: `SELECT … FROM productos WHERE id = $1 FOR UPDATE` → reglas → `UPDATE` →
  `auditoria::registrar(ASIGNAR_CODIGO_BARRAS | REEMPLAZAR_CODIGO_BARRAS)` → `commit`.
- Duplicado detectado por la violación `23505` de `productos_codigo_barras_key` (sin pre-chequeo
  TOCTOU), como en 003.
- `automatico` → `"20" + format!("{:010}", id) + verificador`.
- Asignar el mismo código que ya tiene el producto con `reemplazar: true` → 200 sin cambios y sin
  auditoría (idempotente).
- Se permite sobre productos inactivos (la unicidad incluye inactivos).

**OpenAPI**: registrar el path en `backend/src/docs.rs` con el tag `productos`.

**Cliente tipado** (`frontend/src/lib/api/productos.ts`):

```ts
asignarCodigoBarras(id: number, datos: { modo: 'automatico' | 'manual'; valor?: string; reemplazar: boolean }): Promise<Producto>
resolverLectura(valor: string): Promise<ResultadoLectura>
```
