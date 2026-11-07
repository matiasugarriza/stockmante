# Plan de implementación: Movimientos de stock (007)

**Spec**: `spec.md` · **Rama histórica**: `feat/modulo-2-stock` (mergeada a `main`) · **Migración**: `0009_stock_movimientos.sql`

## Contexto técnico

- `productos` existe desde `0002` y `depositos`/`ubicaciones`/`producto_ubicacion_permitida`
  desde `0003`; no hay ninguna tabla de stock todavía.
- Patrón de capas vigente: `models` (reglas puras) → `services` (SQLx y transacciones) →
  `handlers` (HTTP, DTOs, autorización por extractor) → `app.rs` (router).
- La auditoría se escribe con `services::auditoria::registrar` dentro de la misma transacción.
- Los errores de dominio viven en `AppError` con `status()` y `codigo()` estables.

## Constitution Check

| Principio | Cumplimiento |
|---|---|
| I. Simplicidad y anti-sobreingeniería | una tabla puente, un servicio de movimientos y uno de stock; sin abstracciones nuevas |
| II. Mantenibilidad | modelos/servicios/handlers acotados; SQL transaccional centralizado; nombres de negocio en español |
| III. Claridad en mensajes | 409/422 con mensajes literales de la spec y stock resultante por línea |
| IV. Testabilidad primero | unitarios de validación, integración HTTP de escenarios 7.1–7.8 y tests de cliente |
| V. Integridad de datos | transacción única, `stock_por_deposito` como fuente de verdad, `CHECK` en BD y cache recalculado |

## Decisiones de diseño

1. **`stock_por_deposito` es la fuente de verdad** (constitución, Principio V). Tiene
   `producto_id`, `deposito_id`, `ubicacion_id` opcional y `cantidad >= 0`, con
   `UNIQUE NULLS NOT DISTINCT (producto_id, deposito_id, ubicacion_id)` para que dos filas
   "sin ubicación" no se dupliquen.
2. **`productos.stock_actual` es cache**. Se recalcula como `SUM(stock_por_deposito.cantidad)`
   dentro de la misma transacción, después de aplicar las líneas. Recalcular (en vez de sumar
   el delta) elimina por construcción la deriva del cache (SC-014).
3. **Signo en la línea**. La API recibe siempre `cantidad_delta` con signo:
   `ENTRADA > 0`, `SALIDA` y `MERMA < 0`, `AJUSTE ≠ 0`. Es lo que ya dice FR-024 y evita dos
   campos para lo mismo.
4. **Serialización por producto**. Antes de tocar stock se bloquean las filas de `productos`
   afectadas con `SELECT … ORDER BY id FOR UPDATE`. Sin ese bloqueo, dos movimientos en
   ubicaciones distintas del mismo producto podrían recalcular el cache con una lectura
   incompleta del otro y perder unidades.
5. **Ubicaciones permitidas**. Si la línea trae `ubicacion_id`, debe existir, pertenecer al
   depósito del movimiento y estar en `producto_ubicacion_permitida` (005). Sin ubicación, el
   stock se imputa al depósito. Consecuencia esperada: un producto sin asignaciones solo admite
   movimientos a nivel depósito hasta que se le asignen ubicaciones.
6. **Producto inactivo → 409 `producto_inactivo`** (FR-022 de 003); depósito inactivo → 422
   `validacion`. La verificación ocurre dentro de la transacción.
7. **Sin idempotencia**: un reintento manual duplica el movimiento (supuesto de la spec).

## Superficie de API

| Método | Ruta | Permiso | Respuesta |
|---|---|---|---|
| POST | `/api/movimientos` | `OPERACION` (ENTRADA/SALIDA/MERMA) y `GESTION` (todos) | 201 detalle |
| GET | `/api/movimientos` | sesión vigente | 200 página |
| GET | `/api/movimientos/{id}` | sesión vigente | 200 detalle |
| GET | `/api/stock` | sesión vigente | 200 página |
| GET | `/api/stock/{producto_id}` | sesión vigente | 200 desglose |

**Cuerpo de `POST /api/movimientos`**

```json
{
  "tipo": "ENTRADA",
  "deposito_id": 1,
  "motivo": "COMPRA",
  "observacion": null,
  "lineas": [{ "producto_id": 1, "ubicacion_id": null, "cantidad_delta": 10 }]
}
```

**Errores nuevos**

| Código | Status | Cuándo |
|---|---|---|
| `stock_insuficiente` | 409 | `Stock insuficiente en Principal: 3 disponibles, 5 solicitadas.` |
| `producto_inactivo` | 409 | `El producto está inactivo y no admite movimientos.` |

**Respuesta y mensaje por línea**

`POST /api/movimientos` devuelve `MovimientoDetalle` en JSON: `deposito.nombre`,
`lineas[].producto_nombre` y `lineas[].stock_resultante`. El texto
`"Depósito Principal / Yerba 1 kg: 10 unidades"` es copy de la UI; la API no devuelve un campo
`mensaje`.

**Errores de validación (422 `validacion`)**

| Caso | Mensaje |
|---|---|
| Tipo inválido | `El tipo de movimiento debe ser ENTRADA, SALIDA, MERMA o AJUSTE.` |
| Motivo inválido | `El motivo debe ser COMPRA, VENTA, MERMA, AJUSTE, DEVOLUCION, TRANSFERENCIA, INVENTARIO, IMPORTACION u OTRO.` |
| `OTRO` sin observación | `Con motivo OTRO tenés que indicar una observación.` |
| Línea vacía o exceso | `El movimiento debe tener al menos una línea.` / `El movimiento no puede superar las 200 líneas.` |
| Delta cero o signo incorrecto | `La cantidad de cada línea debe ser distinta de cero.` / `En una ENTRADA la cantidad debe ser mayor que cero.` / `En una {tipo} la cantidad debe ser negativa.` |
| Producto/ubicación inválidos | `El producto de cada línea es obligatorio.` / `La ubicación indicada no es válida.` |
| Línea repetida | `No podés repetir el mismo producto y ubicación en dos líneas.` |
| Depósito inactivo | `El depósito está inactivo.` |
| Ubicación inválida | `La ubicación no pertenece al depósito del movimiento.` / `La ubicación está inactiva.` / `La ubicación no está permitida para {producto}.` |

## Archivos

| Archivo | Cambio |
|---|---|
| `migrations/0009_stock_movimientos.sql` | `stock_por_deposito`, `movimientos`, `movimiento_lineas`, `productos.stock_actual` |
| `backend/src/models/movimiento.rs` | tipos, motivos y validaciones puras |
| `backend/src/models/stock.rs` | structs de consulta de stock y movimientos |
| `backend/src/error.rs` | `StockInsuficiente`, `ProductoInactivo` |
| `backend/src/services/movimientos.rs` | alta transaccional, listado y detalle |
| `backend/src/services/stock.rs` | listado y desglose por producto |
| `backend/src/services/auditoria.rs` | `TABLA_MOVIMIENTOS` |
| `backend/src/handlers/movimientos.rs`, `handlers/stock.rs` | HTTP + DTOs |
| `backend/src/app.rs`, `handlers/mod.rs`, `docs.rs` | router y OpenAPI |
| `backend/tests/unit/test_movimiento_validacion.rs` | reglas puras |
| `backend/tests/integration/test_movimientos_http.rs` | contrato HTTP y atomicidad |
| `backend/tests/unit/test_openapi.rs` | rutas nuevas |
| `frontend/src/lib/schemas/movimientos.ts`, `lib/api/movimientos.ts`, `lib/api/stock.ts` | cliente tipado |
| `frontend/src/routes/movimientos/+page.svelte` | alta y consulta |
| `frontend/src/routes/+layout.svelte` | enlace de navegación |
| `docs/api.md` | tabla de rutas |

## Estrategia de pruebas

1. **Unitarias** (`test_movimiento_validacion.rs`): tipo inválido, motivo inválido, `OTRO` sin
   observación, delta con signo incorrecto por tipo, delta cero, lista vacía, línea duplicada.
2. **Integración** (`test_movimientos_http.rs`), sobre `stockmante_test`:
   - entrada 201 → `stock_por_deposito` y `stock_actual` actualizados;
   - salida sin stock → 409 `stock_insuficiente` con el mensaje literal y stock intacto;
   - merma 201 con stock resultante;
   - `AJUSTE` con `OPERACION` → 403; con `GESTION` → 201;
   - multi-línea con la última inválida → 409/422 y **ninguna** fila escrita;
   - producto inactivo → 409 `producto_inactivo`; depósito inactivo → 422;
   - ubicación de otro depósito y ubicación no asignada → 422;
   - `GET /api/movimientos` filtra por tipo y depósito; `GET /api/movimientos/{id}` trae líneas;
   - `GET /api/stock` y `GET /api/stock/{id}` devuelven cache y desglose coherentes;
   - SC-014: tras una secuencia con un fallo en el medio, `stock_actual` = suma por depósito.
3. **Frontend**: tests de schemas y del cliente (`movimientos.test.ts`), sin E2E.

## Orden de ejecución

1. Migración y aplicación en `stockmante` y `stockmante_test`.
2. Modelos, errores y servicio.
3. Handlers, router y OpenAPI.
4. Tests de backend en verde.
5. Cliente y pantalla de frontend.
6. `cargo test`, `npm test`, `npm run check`, `npm run build`, `cargo sqlx prepare --check`.

## Riesgos

- **Alto, datos**: cambia el esquema y escribe stock; toda escritura va en transacción y con
  `CHECK` en la base como red de seguridad.
- **Concurrencia**: el orden de bloqueo (`productos` por id ascendente) evita deadlocks.
- **Migración**: `productos.stock_actual` arranca en 0; no hay stock previo que migrar.
