# Plan de implementación: Inventario físico por conteo (009)

**Spec**: `spec.md` · **Rama**: `feat/modulo-2-stock` · **Migración**: `0011_inventario_fisico.sql`

## Contexto técnico

- `stock_por_deposito`, `movimientos` y `movimiento_lineas` existen desde `0009` (007).
- `depositos` y `ubicaciones` existen desde `0003`.
- Patrón de capas vigente: `models` → `services` (SQLx + transacciones) → `handlers` (HTTP, DTOs, autorización) → `app.rs`.
- La auditoría se escribe con `services::auditoria::registrar` dentro de la misma transacción.
- Los errores de dominio viven en `AppError` con `status()` y `codigo()` estables.
- `AJUSTE` requiere familia `GESTION` (FR-027, 007).

## Decisiones de diseño

1. **Tabla `conteos` + `conteo_lineas`**. Un conteo es cabecera con estado; las líneas capturan
   `cantidad_sistema` (snapshot al abrir), `cantidad_contada` (nullable hasta cargar) y la
   diferencia calculada al cerrar. La combinación (conteo, producto, ubicación) es única.
2. **Una transacción para el cierre** (Principio V). El servicio bloquea el conteo, recalcula
   `cantidad_sistema` contra el stock vigente, genera un único movimiento `AJUSTE` (motivo
   `INVENTARIO`) con una línea por diferencia distinta de cero, aplica el delta a
   `stock_por_deposito`, recalcula `productos.stock_actual` y audita. Todo en `pool.begin()`.
3. **Servicio dedicado** `services::conteos.rs`: el cierre necesita múltiples líneas con lógica
   de diff y un solo movimiento; no se reutiliza `movimientos::crear` por línea (rompería
   atomicidad y generaría N movimientos).
4. **Estado del conteo**: `ABIERTO`, `CERRADO`, `CANCELADO`. Transiciones:
   `ABIERTO → CERRADO` (cierra y ajusta), `ABIERTO → CANCELADO` (sin ajuste). No reabrible.
5. **Idempotencia del cierre**: el primer cierre muta; un reintento devuelve 409 `conteo_cerrado`
   sin re-aplicar el ajuste.
6. **Recálculo de `cantidad_sistema` al cerrar**: se lee el stock vigente por línea, no el
   snapshot del apertura, para no ajustar contra un valor viejo.
7. **Permisos**: toda la superficie (crear, cargar, cerrar, cancelar, consultar) reservada a
   `GESTION`; `OPERACION` recibe 403 `sin_permisos`.
8. **Cancelación**: no toca stock, pasa a `CANCELADO`.

## Superficie de API

| Método | Ruta | Permiso | Respuesta |
|---|---|---|---|
| POST | `/api/conteos` | `GESTION` | 201 conteo abierto con líneas |
| GET | `/api/conteos` | `GESTION` | 200 página |
| GET | `/api/conteos/{id}` | `GESTION` | 200 detalle con diferencias |
| PATCH | `/api/conteos/{id}/lineas` | `GESTION` | 200 conteo con diferencias actualizadas |
| POST | `/api/conteos/{id}/cerrar` | `GESTION` | 200 conteo cerrado |
| POST | `/api/conteos/{id}/cancelar` | `GESTION` | 200 conteo cancelado |

**Errores nuevos**

| Código | Status | Cuándo |
|---|---|---|
| `conteo_cerrado` | 409 | El conteo ya está cerrado o cancelado. |
| `conteo_cancelado` | 409 | El conteo está cancelado. |
| `conteo_no_encontrado` | 404 | No existe el conteo. |
| `lineas_sin_contar` | 422 | Hay líneas con `cantidad_contada` nula al cerrar. |
| `deposito_inactivo` | 422 | El depósito está inactivo. |

## Archivos

| Archivo | Cambio |
|---|---|
| `migrations/0011_inventario_fisico.sql` | tablas `conteos`, `conteo_lineas` |
| `backend/src/models/conteo.rs` | reglas puras de validación |
| `backend/src/models/stock.rs` | structs de respuesta (`ConteoDetalle`, `ListaConteos`, `ConteoLinea`) |
| `backend/src/error.rs` | `ConteoCerrado`, `ConteoCancelado`, `ConteoNoEncontrado` |
| `backend/src/services/conteos.rs` | alta, carga, cierre transaccional, cancelación, listado/detalle |
| `backend/src/services/auditoria.rs` | `TABLA_CONTEOS` |
| `backend/src/handlers/conteos.rs` | HTTP + DTOs |
| `backend/src/handlers/mod.rs`, `app.rs`, `docs.rs` | router y OpenAPI |
| `backend/tests/unit/test_conteo_validacion.rs` | reglas puras |
| `backend/tests/integration/test_conteos_http.rs` | contrato HTTP y atomicidad |
| `backend/tests/unit/test_openapi.rs` | rutas nuevas |
| `frontend/src/lib/schemas/conteos.ts` | Zod schema |
| `frontend/src/lib/api/conteos.ts` | cliente tipado |
| `frontend/src/routes/conteos/+page.svelte` | apertura, carga, cierre, cancelación |
| `frontend/src/routes/+layout.svelte` | enlace de navegación |
| `docs/api.md` | tabla de rutas |

## Estrategia de pruebas

1. **Unitarias** (`test_conteo_validacion.rs`): estado inválido, cantidad_contada negativa,
   depósito inválido, líneas vacías.
2. **Integración** (`test_conteos_http.rs`), sobre `stockmante_test`:
   - abrir conteo 201 con líneas del stock actual;
   - cargar cantidades 200 y diferencias calculadas;
   - cerrar con diferencias → un AJUSTE con líneas por diff ≠ 0, stock = contado (SC-017);
   - cerrar sin diferencias → sin movimiento;
   - cerrar conteo cerrado → 409 `conteo_cerrado`;
   - cancelar → sin ajuste, estado CANCELADO;
   - OPERACION → 403 en toda la superficie;
   - líneas sin contar al cerrar → 422 `lineas_sin_contar`.
3. **Frontend**: tests de schemas y cliente (`conteos.test.ts`).

## Orden de ejecución

1. Migración y aplicación en `stockmante` y `stockmante_test`.
2. Modelos, errores y servicio.
3. Handlers, router y OpenAPI.
4. Tests de backend en verde.
5. Cliente y pantalla de frontend.
6. `cargo test`, `npm test`, `npm run check`, `npm run build`, `cargo sqlx prepare --check`.

## Riesgos

- **Alto, datos**: el cierre genera un AJUSTE que muta stock en todas las líneas con diff;
  toda la escritura va en transacción.
- **Concurrencia**: bloqueo `FOR UPDATE` sobre el conteo evita doble cierre.
- **Migración**: `conteos` y `conteo_lineas` arrancan vacías.