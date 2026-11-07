# Plan de implementación: Transferencias entre depósitos (008)

**Spec**: `spec.md` · **Rama histórica**: `feat/modulo-2-stock` (implementado en `928aa8a`) ·
**Migración**: `0010_transferencias.sql`

**Revisado**: 2026-11-07. La implementación original (T001–T024) sigue vigente salvo lo que
corrige la sección «Ampliación 2026-11-07», que remedia los hallazgos de `/speckit analyze`
(tests de integración ausentes, `expect()` en producción, semántica de stock por ubicación,
detalle de movimientos, auditoría sin test). Sin migración nueva.

## Contexto técnico

- `stock_por_deposito`, `movimientos` y `movimiento_lineas` existen desde `0009` (007).
- `depositos`, `ubicaciones` y `producto_ubicacion_permitida` existen desde `0003`.
- Las migraciones viven en `migrations/` (raíz del repo), no en `backend/migrations/`.
- Las consultas usan `sqlx::query*` sin macro, igual que 007 (ver decisión 13).
- Patrón de capas vigente: `models` (reglas puras) → `services` (SQLx y transacciones) →
  `handlers` (HTTP, DTOs, autorización por extractor) → `app.rs` (router).
- La auditoría se escribe con `services::auditoria::registrar` dentro de la misma transacción.
- Los errores de dominio viven en `AppError` con `status()` y `codigo()` estables.

## Constitution Check

| Principio | Cumplimiento |
|---|---|
| I. Simplicidad | una tabla cabecera y un servicio dedicado; sin abstracciones nuevas; el detalle no embebe movimientos completos (decisión 11) |
| II. Mantenibilidad | **Violación detectada**: `expect()` en `services/transferencias.rs:93,96`. Se corrige en T029 |
| III. Claridad en mensajes | mensajes literales de la spec; stock resultante en origen y destino; el 404 se alinea al contrato y agrega qué hacer (T033) |
| IV. Testabilidad primero | **Violación detectada**: faltan los tests de integración (T007–T009) y no hay evidencia de RED. Se escriben en la Fase 5 y se propone el waiver W-004 (T025) |
| V. Integridad de datos | transacción única, `CHECK (cantidad >= 0)`, cache recalculado; se agrega suma con `checked_add` en destino (T029) y SC-016 en fallos (T031) |

Gate: la feature **no se considera cerrada** hasta que T025–T036 estén completas y los quality
gates en verde.

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

## Ampliación 2026-11-07

Remediación de `/speckit analyze` (hallazgos C1–C3, H1–H4, M1–M6, L1–L3). Las decisiones 9 y 11
fijan semántica que la spec dejaba abierta; `/speckit clarify` las confirmó sin cambios el
2026-11-07 (spec.md, Clarifications; T034).

**Decisiones**

9. **Stock por fila de ubicación, igual que 007** (H1). "Stock en origen" es la fila
   `stock_por_deposito (producto, depósito, ubicación)` indicada; sin `ubicacion_origen_id` es la
   fila sin ubicación (`ubicacion_id IS NULL`), no el total del depósito. Así se comportan las
   salidas de 007, y una transferencia es una salida más una entrada. `stock_origen_resultante` y
   `stock_destino_resultante` son el saldo de esas filas, el mismo valor que guarda
   `movimiento_lineas.stock_resultante`. El mensaje de 409 se mantiene literal (8.2).
10. **Sin `expect()` ni suma sin control** (C2). Los depósitos bloqueados se toman con
    `ok_or_else(|| AppError::Validacion(...))?`, y el incremento en destino usa `checked_add`,
    como 007, para devolver 422 en vez de un desborde.
11. **El detalle referencia los movimientos por id** (M5). `GET /api/transferencias/{id}` expone
    `movimiento_salida_id` y `movimiento_entrada_id`; el tipo, el depósito y el motivo de cada uno se
    consultan con `GET /api/movimientos/{id}`. Embeber los movimientos duplicaría el shape de 007
    sin un consumidor que lo necesite (Principio I).
12. **Orden de bloqueo real** (M1). Primero los depósitos `FOR UPDATE` por id ascendente; esa
    fila serializa cualquier operación sobre el mismo par de depósitos, así que el bloqueo de las
    filas de stock (origen y luego destino) no puede generar deadlock. Se documenta así en
    `data-model.md`, en lugar de cambiar el código.
13. **Consultas sin macro** (M6). Se mantiene `sqlx::query*` sin macro, por coherencia con 007.
    La verificación contra el esquema la dan los tests de integración sobre `stockmante_test`, que
    ejecutan todas las consultas del servicio. Migrar a `query!` es una decisión de todo el
    proyecto, fuera de 008.
14. **Sin idempotencia** (L1). Ya figuraba en la decisión 7; se agrega a los supuestos de la spec.
15. **Permiso** (L2). `OPERACION` y `GESTION` son las únicas familias de rol, así que el handler
    exige solo sesión vigente. Si aparece una familia nueva, esta regla se revisa.

**Archivos**

| Archivo | Cambio |
|---|---|
| `backend/src/services/transferencias.rs` | quitar `expect()`; `checked_add` en destino; mensaje del 404 |
| `backend/Cargo.toml` | `[[test]]` para `test_transferencia_validacion` y `test_transferencias_http` |
| `backend/tests/unit/common/mod.rs` | helper `crear_transferencia_de_prueba` |
| `backend/tests/unit/test_transferencia_validacion.rs` | reglas puras vía la API pública del crate |
| `backend/tests/integration/test_transferencias_http.rs` | 8.1–8.5, ubicaciones, SC-016 en fallo, auditoría, 404 |
| `backend/tests/unit/test_openapi.rs` | las tres rutas y los dos schemas |
| `specs/008-transferencias-depositos/progress.md` | evidencia de RED y waiver W-004 propuesto |
| `specs/008-transferencias-depositos/data-model.md` | relaciones 1:1 y orden de bloqueo real |
| `specs/008-transferencias-depositos/contracts/api.md` | semántica por ubicación y referencia a movimientos |
| `specs/008-transferencias-depositos/spec.md`, `specs/README.md` | estado, supuestos y aclaraciones |

**Pruebas nuevas** (además de las de la estrategia original, que pasan a existir de verdad)

- Transferencia fallida (stock insuficiente, depósito inactivo, mismo depósito): la suma global
  de `stock_por_deposito` y `productos.stock_actual` no cambian, y no hay filas nuevas en
  `transferencias`, `movimientos` ni `auditoria` (SC-016).
- Una transferencia exitosa deja una fila de auditoría `tabla_ref = 'transferencias'`,
  `accion = 'CREAR'` y `registro_id` igual al id de la transferencia (FR-033).
- Con 20 unidades solo en la ubicación A-01 de origen, transferir sin `ubicacion_origen_id`
  responde 409 `stock_insuficiente` con "0 disponibles" (decisión 9).
- `GET /api/movimientos/{movimiento_salida_id}` y `{movimiento_entrada_id}` devuelven `SALIDA` en
  origen y `ENTRADA` en destino, ambos con motivo `TRANSFERENCIA` (8.5, decisión 11).
- `GET /api/transferencias/999999` → 404 `recurso_no_encontrado` con el mensaje del contrato.

**Orden**: progress.md → helper y `Cargo.toml` → tests (registrar RED o verde inmediato) →
correcciones del servicio → docs de spec → quality gates.

## Riesgos

- **Alto, datos**: escribe stock en dos depósitos; toda escritura va en transacción y con
  `CHECK (cantidad >= 0)` en la base como red de seguridad.
- **Concurrencia**: el orden de bloqueo (`depositos` por id, `productos` por id ascendente)
  evita deadlocks con movimientos concurrentes de 007.
- **Migración**: `transferencias` arranca vacía; `movimientos.transferencia_id` es nullable
  y las filas existentes quedan en NULL.

## Implementación de la remediación (2026-11-07)

16. **Saldos derivados, sin migración**. `0010_transferencias.sql` no tiene columnas de saldo; el
    servicio las leía y fallaba con 500. Se derivan con `JOIN movimiento_lineas` sobre
    `movimiento_salida_id`/`movimiento_entrada_id`: es exactamente el dato de la decisión 9 y no se
    duplica. Se corrigió además el alias `do` (reservado en PostgreSQL). Evidencia en
    [progress.md](./progress.md); waiver W-004 registrado en la constitución 1.3.4.
