# Plan de implementación: Transferencias entre depósitos (008)

**Spec**: `spec.md` · **Rama**: `feat/modulo-2-stock` · **Migración**: `0010_transferencias.sql`

## Contexto técnico

- `stock_por_deposito`, `movimientos` y `movimiento_lineas` existen desde `0009` (007).
- `depositos` y `ubicaciones` existen desde `0003`; `producto_ubicacion_permitida` desde `0005`.
- Patrón de capas vigente: `models` (reglas puras) → `services` (SQLx y transacciones) →
  `handlers` (HTTP, DTOs, autorización por extractor) → `app.rs` (router).
- La auditoría se escribe con `services::auditoria::registrar` dentro de la misma transacción.
- Los errores de dominio viven en `AppError` con `status()` y `codigo()` estables.

## Decisiones de diseño

1. **Tabla `transferencias` como cabecera de la operación**. Cada transferencia es una fila
   que enlaza dos movimientos (`SALIDA` en origen y `ENTRADA` en destino) mediante
   `movimientos.transferencia_id`. Esto permite consultar la transferencia como unidad y
   trazar ambos movimientos sin reconstruirlos.
2. **Una transacción para todo** (Principio V). El servicio bloquea depósitos, productos y
   stock en el mismo orden que 007 (`depositos` → `productos` → `stock_por_deposito`),
   descuenta origen, incrementa destino, escribe los dos movimientos, recalcula
   `productos.stock_actual` (que no cambia porque la suma global es constante) y audita.
3. **Reutilización de `movimientos::crear` NO aplicable**. La transferencia necesita dos
   movimientos en depósitos distintos con la misma transacción; el servicio actual crea un
   movimiento por llamada. Se escribe un servicio dedicado que replica la lógica de bloqueo
   y validación pero aplicando ambas direcciones en un solo `pool.begin()`.
4. **Motivo fijo `TRANSFERENCIA`**. Los dos movimientos generados siempre llevan
   `motivo = 'TRANSFERENCIA'` y `tipo` `SALIDA`/`ENTRADA` respectivamente. No se expone
   al usuario como parámetro.
5. **Ubicación origen y destino opcionales**. Se permite indicar `ubicacion_id` en origen
   (de dónde sale) y en destino (a dónde entra). Si se omite, el stock se imputa al depósito
   completo. Las reglas de `producto_ubicacion_permitida` (005) aplican igual que en 007.
6. **Producto inactivo → 409 `producto_inactivo`** (consistente con 007); depósito inactivo
   (origen o destino) → 422 `validacion`.
7. **Sin idempotencia**: un reintento manual duplica la transferencia (supuesto de la spec).
8. **Consulta con filtros**. `GET /api/transferencias` admite filtros por `producto_id`,
   `deposito_origen_id`, `deposito_destino_id`, `desde`, `hasta` y paginación.

## Superficie de API

| Método | Ruta | Permiso | Respuesta |
|---|---|---|---|
| POST | `/api/transferencias` | `OPERACION` y `GESTION` | 201 detalle |
| GET | `/api/transferencias` | sesión vigente | 200 página |
| GET | `/api/transferencias/{id}` | sesión vigente | 200 detalle |

**Cuerpo de `POST /api/transferencias`**

```json
{
  "producto_id": 1,
  "deposito_origen_id": 1,
  "deposito_destino_id": 2,
  "cantidad": 8,
  "ubicacion_origen_id": null,
  "ubicacion_destino_id": null,
  "observacion": "Abastecer sucursal"
}
```

**Respuesta 201** (incluye stock resultante en ambos depósitos — Principio III):

```json
{
  "id": 1,
  "producto_id": 1,
  "producto_nombre": "Café",
  "sku": "CAF-001",
  "deposito_origen": { "id": 1, "nombre": "Principal" },
  "deposito_destino": { "id": 2, "nombre": "Sucursal" },
  "cantidad": 8,
  "stock_origen_resultante": 12,
  "stock_destino_resultante": 8,
  "usuario": { "id": 3, "nombre": "Juan" },
  "creado_en": "2026-10-29T12:00:00Z",
  "movimiento_salida_id": 10,
  "movimiento_entrada_id": 11,
  "observacion": "Abastecer sucursal"
}
```

**Errores nuevos**

| Código | Status | Cuándo |
|---|---|---|
| `transferencia_mismo_deposito` | 422 | `"El depósito de origen y el de destino deben ser distintos."` |
| `stock_insuficiente` | 409 | Reutiliza el de 007 |
| `producto_inactivo` | 409 | Reutiliza el de 007 |

## Archivos

| Archivo | Cambio |
|---|---|
| `migrations/0010_transferencias.sql` | tabla `transferencias`, columna `movimientos.transferencia_id` |
| `backend/src/models/transferencia.rs` | reglas puras de validación |
| `backend/src/models/stock.rs` | structs de respuesta (`TransferenciaDetalle`, `ListaTransferencias`) |
| `backend/src/error.rs` | `TransferenciaMismoDeposito` |
| `backend/src/services/transferencias.rs` | alta transaccional, listado y detalle |
| `backend/src/services/auditoria.rs` | `TABLA_TRANSFERENCIAS` |
| `backend/src/handlers/transferencias.rs` | HTTP + DTOs |
| `backend/src/handlers/mod.rs`, `app.rs`, `docs.rs` | router y OpenAPI |
| `backend/tests/unit/test_transferencia_validacion.rs` | reglas puras |
| `backend/tests/integration/test_transferencias_http.rs` | contrato HTTP y atomicidad |
| `backend/tests/unit/test_openapi.rs` | rutas nuevas |
| `frontend/src/lib/schemas/transferencias.ts` | Zod schema |
| `frontend/src/lib/api/transferencias.ts` | cliente tipado |
| `frontend/src/routes/transferencias/+page.svelte` | alta y consulta |
| `frontend/src/routes/+layout.svelte` | enlace de navegación |
| `docs/api.md` | tabla de rutas |

## Estrategia de pruebas

1. **Unitarias** (`test_transferencia_validacion.rs`): origen = destino, cantidad ≤ 0,
   producto_id inválido, depósito origen/destino inválidos, observación demasiado larga.
2. **Integración** (`test_transferencias_http.rs`), sobre `stockmante_test`:
   - transferencia 201 → stock resultante en origen y destino correctos;
   - stock insuficiente en origen → 409 con el mensaje literal y ningún depósito cambia;
   - origen = destino → 422 `transferencia_mismo_deposito`;
   - depósito destino inactivo → 422 y nada escrito;
   - producto inactivo → 409 `producto_inactivo`;
   - con `ubicacion_origen_id` y `ubicacion_destino_id` → 201 y stock en ubicación;
   - ubicación de otro depósito → 422;
   - `GET /api/transferencias` filtra por producto y depósitos;
   - `GET /api/transferencias/{id}` trae los dos movimientos generados;
   - SC-016: suma global de stock idéntica antes y después de una transferencia exitosa.
3. **Frontend**: tests de schemas y del cliente (`transferencias.test.ts`), sin E2E.

## Orden de ejecución

1. Migración y aplicación en `stockmante` y `stockmante_test`.
2. Modelos, errores y servicio.
3. Handlers, router y OpenAPI.
4. Tests de backend en verde.
5. Cliente y pantalla de frontend.
6. `cargo test`, `npm test`, `npm run check`, `npm run build`, `cargo sqlx prepare --check`.

## Riesgos

- **Alto, datos**: escribe stock en dos depósitos; toda escritura va en transacción y con
  `CHECK (cantidad >= 0)` en la base como red de seguridad.
- **Concurrencia**: el orden de bloqueo (`depositos` por id, `productos` por id ascendente)
  evita deadlocks con movimientos concurrentes de 007.
- **Migración**: `transferencias` arranca vacía; `movimientos.transferencia_id` es nullable
  y las filas existentes quedan en NULL.
