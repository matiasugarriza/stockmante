# Plan de implementación: Movimientos de stock (007)

**Spec**: `spec.md` · **Rama histórica**: `feat/modulo-2-stock` (mergeada a `main`) · **Migración**: `0009_stock_movimientos.sql`

**Revisado**: 2026-11-07. La implementación original (T001–T019) sigue vigente; la sección
«Ampliación 2026-11-07» agrega validaciones y mensajes surgidos de `/speckit analyze` y de las dos
rondas de `/speckit clarify`. Sin migración nueva.

## Contexto técnico

- `productos` existe desde `0002` y `depositos`/`ubicaciones`/`producto_ubicacion_permitida`
  desde `0003`; las tablas de stock (`stock_por_deposito`, `movimientos`, `movimiento_lineas`)
  y `productos.stock_actual` las crea `0009` (implementada). 0010, 0013 y 0014 agregan a
  `movimientos` las FK de transferencias, compras y ventas.
- Patrón de capas vigente: `models` (reglas puras) → `services` (SQLx y transacciones) →
  `handlers` (HTTP, DTOs, autorización por extractor) → `app.rs` (router).
- La auditoría se escribe con `services::auditoria::registrar` dentro de la misma transacción.
- Los errores de dominio viven en `AppError` con `status()` y `codigo()` estables.

## Constitution Check

| Principio | Cumplimiento |
|---|---|
| I. Simplicidad y anti-sobreingeniería | una tabla puente, un servicio de movimientos y uno de stock; sin abstracciones nuevas |
| II. Mantenibilidad | modelos/servicios/handlers acotados; SQL transaccional centralizado; nombres de negocio en español |
| III. Claridad en mensajes | todo 409/422 del alta dice qué pasó y qué hacer (literales de la spec); stock resultante por línea |
| IV. Testabilidad primero | unitarios de validación, integración HTTP de escenarios 7.1–7.11 y tests de cliente; RED antes del código en la ampliación, salvo lo cubierto por W-005 (T011, T012, T025, T033), que debe estar aprobado y registrado |
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
8. **Coherencia tipo ↔ motivo solo en el alta manual** (ampliación 2026-11-07). La regla vive en
   `models/movimiento.rs` como función pura y la usa `services/movimientos.rs::crear`. Los flujos
   de 008, 009, 012, 013 y 016 insertan sus movimientos con su propio SQL y no pasan por ella, así
   que siguen usando `TRANSFERENCIA`, `INVENTARIO`, `IMPORTACION`, `COMPRA`, `VENTA` y
   `DEVOLUCION`. No se agrega `CHECK` en la base: la combinación depende del origen.
9. **`AJUSTE` negativo** reutiliza la guarda de no negatividad: 409 `stock_insuficiente`.
10. **Mensajes con siguiente paso** (Principio III): se reescriben todos los 409/422 del alta que
    hoy no dicen qué hacer: depósito y producto inexistentes (nombrando el id), depósito inactivo,
    ubicaciones, cantidad resultante fuera de rango, `producto_inactivo` y `stock_insuficiente`.
    Los que enumeran valores válidos o un límite ya cumplen y no cambian.
11. **Orden de evaluación** (gana el primer error): (0) JSON mal formado (extractor) y tipo
    inválido, porque sin ellos no se sabe si es un `AJUSTE`; (1) 403 si `AJUSTE` sin `GESTION`;
    (2) modelo puro: motivo, coherencia tipo ↔ motivo, observación, líneas, cantidades enteras y
    signo; (3) servicio, dentro de la transacción: depósito, productos (existencia y luego 409
    `producto_inactivo`), ubicaciones y stock (409 `stock_insuficiente`). **Cambio de código**: hoy
    `services/movimientos.rs::crear` corre `validar_motivo`, `validar_observacion` y
    `validar_lineas` antes del 403; la comprobación de permiso pasa a ir justo después de
    `validar_tipo`. El escenario 7.10 lo fija con tests.
12. **Cantidad no entera**. Hoy `cantidad_delta: i32` hace que serde rechace `2.5` y el extractor
    devuelve el genérico `El cuerpo de la solicitud no es un JSON válido`. El DTO pasa a recibir
    `serde_json::Number` (`#[schema(value_type = i32)]` en OpenAPI) y una función pura
    `cantidad_entera(&Number) -> Result<i32, AppError>` en `models/movimiento.rs` la convierte:
    no entera → mensaje de 7.11; fuera de `i32` → `La cantidad de cada línea supera el máximo
    admitido. Revisá el valor.`. Un tipo no numérico (texto) sigue en el genérico del extractor.
13. **Mensajes compartidos**. `AppError::StockInsuficiente` y `AppError::ProductoInactivo` los
    usan también transferencias, compras y ventas. Se cambia el texto en `error.rs` (sin cambiar
    la forma de las variantes ni el código de error), así que el siguiente paso es genérico
    ("quitalo de la operación", "registrá antes una entrada en ese depósito").
14. **Waiver W-005**. Alcance T011, T012, T025 y T033 (las dos últimas verifican comportamiento
    ya implementado: bloqueo por producto y filtro `categoria_id`). T020 lo propone con `/speckit constitution`
    (constitución 1.3.1 → 1.3.2, PATCH; registrado el 2026-11-07) y la ampliación no se cierra (T030) sin W-005 registrado.

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
| `stock_insuficiente` | 409 | `Stock insuficiente en Principal: 3 disponibles, 5 solicitadas. Reducí la cantidad o registrá antes una entrada en ese depósito.` |
| `producto_inactivo` | 409 | `El producto está inactivo y no admite movimientos. Reactivalo desde Productos o quitalo de la operación.` |

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
| `AJUSTE` sin observación | `Un ajuste requiere una justificación: indicá la observación.` |
| Observación larga | `La observación no puede superar los 500 caracteres.` |
| Línea vacía o exceso | `El movimiento debe tener al menos una línea.` / `El movimiento no puede superar las 200 líneas.` |
| Delta cero o signo incorrecto | `La cantidad de cada línea debe ser distinta de cero.` / `En una ENTRADA la cantidad debe ser mayor que cero.` / `En una {tipo} la cantidad debe ser negativa.` |
| Cantidad no entera o fuera de rango | `La cantidad de cada línea debe ser un número entero distinto de cero.` / `La cantidad de cada línea supera el máximo admitido. Revisá el valor.` |
| Producto obligatorio | `El producto de cada línea es obligatorio. Elegí un producto o quitá la línea.` |
| Producto inexistente | `El producto {id} no existe. Elegí otro o quitá la línea.` |
| Línea repetida | `No podés repetir el mismo producto y ubicación en dos líneas.` |
| Depósito inexistente | `El depósito {id} no existe. Elegí uno de la lista de Depósitos.` |
| Depósito inactivo | `El depósito está inactivo. Reactivalo desde Depósitos o elegí otro.` |
| Cantidad resultante fuera de rango | `La cantidad resultante supera el máximo admitido. Dividí el movimiento o revisá la cantidad.` |
| Ubicación inexistente | `La ubicación indicada no existe. Elegí una del depósito o dejala vacía.` |
| Ubicación inválida | `La ubicación no pertenece al depósito del movimiento. Elegí una de ese depósito o dejala vacía.` / `La ubicación está inactiva. Elegí otra o dejala vacía para imputar al depósito.` / `La ubicación no está permitida para {producto}. Asignala desde Productos o elegí una permitida.` |
| Motivo incoherente | `El motivo {motivo} no corresponde a una {tipo}. Motivos válidos: {lista}.` |
| Motivo reservado | `El motivo TRANSFERENCIA lo registra el flujo de transferencias.` / `El motivo INVENTARIO lo registra el cierre de conteos.` / `El motivo IMPORTACION lo registra la importación masiva.` |

**Matriz de motivos del alta manual (ampliación 2026-11-07)**

| Tipo | Motivos admitidos |
|---|---|
| `ENTRADA` | `COMPRA`, `DEVOLUCION`, `OTRO` |
| `SALIDA` | `VENTA`, `DEVOLUCION`, `OTRO` |
| `MERMA` | `MERMA`, `OTRO` |
| `AJUSTE` | `AJUSTE`, `OTRO` |

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

## Ampliación 2026-11-07

**Impacto fuera de 007**

- Frontend `routes/movimientos/+page.svelte`: el selector de motivo se filtra por tipo con la
  misma matriz (expuesta desde `lib/schemas/movimientos.ts`) y no ofrece los reservados.
- Asistente (017): `proponer_movimiento` ya usa motivos por defecto coherentes; si el usuario pide
  uno fuera de la matriz, la API responde 422 y el asistente lo informa. Se alinea su `enum` con
  la matriz para no proponer combinaciones que la API rechazará.
- Tests existentes que afirman los mensajes anteriores se actualizan: `test_movimientos_http.rs`
  y `test_asistente_http.rs` (literal de `stock_insuficiente`). Los fixtures del frontend
  (`movimientos.test.ts`, `transferencias.test.ts`, `asistente.test.ts`) no dependen del texto
  real, pero se alinean para no documentar un literal viejo.
- Specs 008 (8.2), 013 (escenarios con stock insuficiente) y 017 (propagación del error) citan el
  literal anterior de `stock_insuficiente`; se alinean en la misma unidad de trabajo.

**Archivos**

| Archivo | Cambio |
|---|---|
| `backend/src/models/movimiento.rs` | `validar_motivo_para_tipo` (matriz + reservados), `cantidad_entera` |
| `backend/src/error.rs` | textos con siguiente paso de `StockInsuficiente` y `ProductoInactivo` |
| `backend/src/handlers/movimientos.rs` | `cantidad_delta: serde_json::Number` con `value_type = i32` |
| `backend/src/services/movimientos.rs` | usa las reglas nuevas y los mensajes reescritos (id inexistente incluido) |
| `backend/tests/unit/test_movimiento_validacion.rs` | matriz, reservados y `cantidad_entera` |
| `backend/tests/integration/test_movimientos_http.rs` | 7.4 ajuste negativo, 7.6 ubicación inactiva, 7.7 inexistentes, 7.9, 7.10 precedencia, 7.11 no entera, filtro por categoría, concurrencia, mensajes |
| `backend/tests/integration/test_asistente_http.rs` | literal nuevo de `stock_insuficiente` |
| `specs/008-…/spec.md`, `specs/013-…/spec.md`, `specs/017-…/spec.md` (+ contratos que lo citen) | literal nuevo |
| `frontend/src/lib/schemas/movimientos.ts` (+ test) | `MOTIVOS_POR_TIPO` |
| `frontend/src/routes/movimientos/+page.svelte` | selector de motivo filtrado |
| `asistente/src/tools.ts` (+ test) | motivos de la propuesta según la matriz |
| `specs/007-movimientos-stock/progress.md` | evidencia de RED (histórica y nueva) |

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
4. **Ampliación 2026-11-07** (RED obligatorio antes del código):
   - unitarios de la matriz tipo ↔ motivo y de los motivos reservados;
   - integración: 7.9 (incoherente y reservado → 422 sin escritura), `AJUSTE` negativo mayor que
     el stock → 409 y stock intacto, ubicación inactiva → 422;
   - concurrencia (SC-014): dos altas simultáneas del mismo producto en ubicaciones distintas
     terminan con `stock_actual` igual a la suma por depósito;
   - mensajes nuevos literales (depósito, producto, ubicación, `producto_inactivo`,
     `stock_insuficiente`);
   - 7.10: `OPERACION` + `AJUSTE` + motivo `VENTA` + producto inactivo → 403; `GESTION` +
     `ENTRADA` + `VENTA` + producto inactivo → 422 por el motivo;
   - 7.11: `cantidad_delta: 2.5` → 422 con el mensaje específico, y unitarios de
     `cantidad_entera` (entero, decimal, fuera de `i32`);
   - FR-029: `GET /api/stock?categoria_id=` devuelve solo esa categoría (los filtros `q` y
     `bajo_minimo` ya los cubre `consulta_de_historial_y_stock`).

## Orden de ejecución

Implementación original (cerrada):

1. Migración y aplicación en `stockmante` y `stockmante_test`.
2. Modelos, errores y servicio.
3. Handlers, router y OpenAPI.
4. Tests de backend en verde.
5. Cliente y pantalla de frontend.
6. `cargo test`, `npm test`, `npm run check`, `npm run build`, `cargo sqlx prepare --check`.

Ampliación 2026-11-07:

1. `progress.md` y propuesta de W-005 (`/speckit constitution`).
2. Tests en RED (unitarios y de integración), salvo T025 y T033 (cubiertos por W-005).
3. Modelo, error, handler y servicio hasta verde.
4. Frontend, asistente, docs y literales de 008/013/017.
5. Quality gates, con W-005 registrado.

## Riesgos

- **Alto, datos**: cambia el esquema y escribe stock; toda escritura va en transacción y con
  `CHECK` en la base como red de seguridad.
- **Concurrencia**: el orden de bloqueo (`productos` por id ascendente) evita deadlocks.
- **Migración**: `productos.stock_actual` arranca en 0; no hay stock previo que migrar.
