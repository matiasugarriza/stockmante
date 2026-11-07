# Especificación de Funcionalidad: Movimientos de stock

**Spec**: `007-movimientos-stock` | **Caso de uso**: US7 | **Prioridad**: P1

**Creado**: 2026-10-26 | **Revisado**: 2026-11-07 (remediación de `/speckit analyze`) |
**Estado**: Implementado; ampliación de validaciones y mensajes pendiente (tasks T020+) | **Módulo**: 2 — Operación de stock

**Origen**: el README y `specs/README.md` declaran "movimientos de stock" fuera del alcance del
Módulo 1. La constitución (Principio V) ya fija el modelo: `stock_por_deposito` como fuente de
verdad y `productos.stock_actual` como cache denormalizado.

**Depende de**: 001 (sesión y RBAC), 003 (productos), 005 (depósitos y ubicaciones), 006 (auditoría).

## Clarifications

### Session 2026-11-07

Decisiones tomadas por el agente con la opción recomendada, a pedido del usuario.

- Q: ¿El alta manual (`POST /api/movimientos`) debe exigir coherencia entre tipo y motivo, y qué
  pasa con los motivos que generan otros flujos? → A: Sí. `ENTRADA` admite `COMPRA`,
  `DEVOLUCION` u `OTRO`; `SALIDA` admite `VENTA`, `DEVOLUCION` u `OTRO`; `MERMA` admite `MERMA` u
  `OTRO`; `AJUSTE` admite `AJUSTE` u `OTRO`. `TRANSFERENCIA`, `INVENTARIO` e `IMPORTACION` quedan
  reservados a 008, 009 y 016 y el alta manual los rechaza con 422. Los flujos de 008, 009, 012,
  013 y 016 no pasan por esta validación.
- Q: ¿Qué responde un `AJUSTE` negativo mayor que el stock disponible? → A: 409
  `stock_insuficiente` con el mismo mensaje que una salida, y el stock no cambia.
- Q: ¿Una ubicación inactiva se rechaza? → A: Sí, 422 `validacion`, igual que una ubicación de
  otro depósito o no permitida.
- Q: ¿Los mensajes de validación deben indicar el siguiente paso? → A: Sí (Principio III): cada
  mensaje de depósito y ubicación agrega qué hacer (por ejemplo, reactivar, elegir otra o dejar la
  ubicación vacía).
- Q: ¿Cómo se cierra la falta de evidencia de RED de 007? → A: No se reconstruye un RED
  artificial: se registra en `progress.md` y se propone un waiver acotado con
  `/speckit constitution`. Toda tarea nueva de 007 (T020+) observa RED antes del código.

Segunda ronda (hallazgos de `/speckit analyze`), respondida por el usuario.

- Q: ¿A qué mensajes de error de `POST /api/movimientos` se les exige indicar qué hacer a
  continuación? → A: A todos los 409 y 422 del alta, incluidos depósito y producto inexistentes
  (nombrando cuál), producto inactivo y stock insuficiente. Un mensaje que enumera los valores
  válidos o el límite admitido ya cumple. Como `stock_insuficiente` y `producto_inactivo` son
  variantes compartidas de `AppError`, el texto nuevo alcanza también a 008, 012 y 013.
- Q: ¿En qué orden se evalúan los errores cuando un pedido tiene varios a la vez? → A: Permiso,
  luego cuerpo, luego base: 403 por `AJUSTE` sin `GESTION` (una vez reconocido el tipo; un JSON
  mal formado o un tipo inválido siguen respondiendo 422 antes); después los 422 de validación pura
  (tipo, motivo, coherencia tipo ↔ motivo, observación, líneas y cantidades); después, dentro de la
  transacción, depósito, productos (409 `producto_inactivo`), ubicaciones y stock (409
  `stock_insuficiente`). Gana el primer error encontrado.
- Q: ¿Qué responde una cantidad no entera (por ejemplo `2.5`)? → A: 422 `validacion` con
  `"La cantidad de cada línea debe ser un número entero distinto de cero."`, no el mensaje
  genérico de JSON inválido.
- Q: ¿Qué cubre el waiver W-005? → A: T011 y T012 (sin RED histórico) y T025 (test de
  concurrencia que verifica un bloqueo ya implementado), y T033 (test del filtro `categoria_id`
  de `GET /api/stock`, comportamiento ya implementado), que el usuario agregó después. Debe quedar
  aprobado y registrado en la constitución antes de cerrar la ampliación; no basta con
  proponerlo.

## Escenarios de Usuario y Pruebas *(obligatorio)*

### User Story 7 - Registrar entradas, salidas, mermas y ajustes (Prioridad: P1)

Como **Operario o Gerente**, quiero registrar cada entrada, salida, merma o ajuste contra un
depósito y una ubicación, para que el stock refleje la realidad y cada cambio quede trazado con
autor, motivo y fecha.

**Por qué esta prioridad**: sin movimientos el inventario es solo un catálogo; es la base del
Módulo 2 y de todo lo que viene después (transferencias, conteos, compras y ventas).

**Escenario 7.1 — Entrada de mercadería (US7)**

- **Dado** un operario con sesión vigente, un producto activo y el depósito "Principal" activo,
- **Cuando** registra `POST /api/movimientos` con tipo `ENTRADA`, motivo `COMPRA` y una línea de
  10 unidades,
- **Entonces** responde **201** con el movimiento, el autor y el stock resultante por línea
  (`"Depósito Principal / Yerba 1 kg: 10 unidades"`), y el stock queda actualizado en la misma
  transacción.

**Escenario 7.2 — Salida sin stock suficiente**

- **Dado** un producto con 3 unidades en "Principal",
- **Cuando** se registra una `SALIDA` de 5 unidades,
- **Entonces** responde **409** `stock_insuficiente` con el mensaje
  `"Stock insuficiente en Principal: 3 disponibles, 5 solicitadas. Reducí la cantidad o registrá
  antes una entrada en ese depósito."` y el stock no cambia.

**Escenario 7.3 — Merma parcial**

- **Dado** un producto con 20 unidades,
- **Cuando** se registra una `MERMA` de 2 unidades con motivo `MERMA` (o `OTRO` con la
  observación "Vencimiento de lote"),
- **Entonces** el stock baja a 18, la respuesta lo informa y la acción queda auditada.

**Escenario 7.4 — Ajuste reservado a gestión**

- **Dado** una sesión `OPERACION`,
- **Cuando** intenta registrar un `AJUSTE`,
- **Entonces** responde **403** `sin_permisos`; con sesión `GESTION` el ajuste procede y puede ser
  positivo o negativo.
- **Y** un `AJUSTE` negativo mayor que el stock disponible responde **409** `stock_insuficiente`
  con el mismo mensaje que una salida y el stock no cambia.

**Escenario 7.5 — Movimiento multi-línea atómico**

- **Dado** un movimiento con tres líneas donde la tercera no tiene stock suficiente,
- **Cuando** se envía el request,
- **Entonces** responde **409** y **no** queda ninguna línea escrita ni ningún stock modificado.

**Escenario 7.6 — Ubicación inválida**

- **Dado** una ubicación inexistente, inactiva, de otro depósito o no asignada al producto (005),
- **Cuando** se registra el movimiento con esa ubicación,
- **Entonces** responde **422** `validacion` con un mensaje que dice qué pasó y qué hacer (elegir
  otra ubicación, asignarla al producto o dejarla vacía para imputar al depósito).

**Escenario 7.7 — Movimiento sobre entidades inactivas**

- **Dado** un producto inactivo,
- **Cuando** se intenta un movimiento,
- **Entonces** responde **409** `producto_inactivo` con
  `"El producto está inactivo y no admite movimientos. Reactivalo desde Productos o quitalo de la
  operación."` y no se escribe nada (FR-022 de 003).
- **Dado** un producto o un depósito inexistente,
- **Cuando** se intenta un movimiento,
- **Entonces** responde **422** `validacion` con un mensaje que nombra el id que no existe y qué
  hacer (elegir otro o quitar la línea), y no se escribe nada.
- **Dado** un depósito inactivo,
- **Cuando** se intenta un movimiento,
- **Entonces** responde **422** `validacion` con `"El depósito está inactivo. Reactivalo desde
  Depósitos o elegí otro."` y no se escribe nada.

**Escenario 7.8 — Consulta de stock y de movimientos**

- **Dado** una sesión autenticada,
- **Cuando** consulta `GET /api/stock/{producto_id}` o `GET /api/movimientos`,
- **Entonces** obtiene el desglose por depósito y el historial paginado con sus líneas.

**Escenario 7.9 — Tipo y motivo incoherentes**

- **Dado** cualquier sesión,
- **Cuando** registra una `ENTRADA` con motivo `VENTA`,
- **Entonces** responde **422** `validacion` con `"El motivo VENTA no corresponde a una ENTRADA.
  Motivos válidos: COMPRA, DEVOLUCION u OTRO."` y no se escribe nada.
- **Cuando** registra un movimiento con motivo `TRANSFERENCIA`, `INVENTARIO` o `IMPORTACION`,
- **Entonces** responde **422** `validacion` indicando qué flujo lo registra (transferencias,
  conteos o importación) y no se escribe nada.

**Escenario 7.10 — Varios errores a la vez**

- **Dado** una sesión `OPERACION`,
- **Cuando** registra un `AJUSTE` con motivo `VENTA` sobre un producto inactivo,
- **Entonces** responde **403** `sin_permisos` (el permiso se evalúa primero) y no se escribe nada.
- **Dado** una sesión `GESTION`,
- **Cuando** registra una `ENTRADA` con motivo `VENTA` sobre un producto inactivo,
- **Entonces** responde **422** `validacion` por el motivo (el cuerpo se valida antes que la base).

**Escenario 7.11 — Cantidad no entera**

- **Dado** cualquier sesión,
- **Cuando** envía una línea con `cantidad_delta: 2.5`,
- **Entonces** responde **422** `validacion` con
  `"La cantidad de cada línea debe ser un número entero distinto de cero."` y no se escribe nada.

## Requisitos Funcionales

- **FR-024 Registro de movimientos.** El sistema DEBE registrar movimientos de tipo `ENTRADA`,
  `SALIDA`, `MERMA` y `AJUSTE`, con una cabecera (tipo, depósito, motivo, observación opcional) y
  entre 1 y 200 líneas (producto, ubicación opcional, `cantidad_delta` entera distinta de cero). El
  motivo DEBE pertenecer al catálogo `COMPRA`, `VENTA`, `MERMA`, `AJUSTE`, `DEVOLUCION`,
  `TRANSFERENCIA`, `INVENTARIO`, `IMPORTACION` u `OTRO`; con `OTRO` la observación es obligatoria.
  Todo `AJUSTE` exige además una justificación en la observación (aunque el motivo sea `AJUSTE`).
  La observación admite hasta 500 caracteres (422 si los supera).
  En el alta manual el motivo DEBE ser coherente con el tipo (`ENTRADA`: `COMPRA`, `DEVOLUCION`,
  `OTRO`; `SALIDA`: `VENTA`, `DEVOLUCION`, `OTRO`; `MERMA`: `MERMA`, `OTRO`; `AJUSTE`: `AJUSTE`,
  `OTRO`); `TRANSFERENCIA`, `INVENTARIO` e `IMPORTACION` quedan reservados a 008, 009 y 016.
- **FR-025 Atomicidad y no negatividad.** Todo movimiento DEBE ejecutarse en una única transacción
  SQLx que actualice `stock_por_deposito` y `productos.stock_actual`; el stock NUNCA puede quedar
  negativo ni en la tabla puente ni en el cache. La base DEBE imponer `CHECK (cantidad >= 0)` y
  `CHECK (stock_actual >= 0)`.
- **FR-026 Autoría y auditoría.** El autor DEBE ser el usuario de la sesión validada, nunca un dato
  del cuerpo; cada movimiento confirmado DEBE registrar una fila de auditoría
  (`tabla_ref = 'movimientos'`, acción `CREAR`).
- **FR-027 Permisos por tipo.** `ENTRADA`, `SALIDA` y `MERMA` DEBEN estar disponibles para
  `OPERACION` y `GESTION`; `AJUSTE` DEBE estar reservado a `GESTION`. La consulta de stock y de
  movimientos DEBE estar disponible para cualquier sesión vigente.
- **FR-028 Validaciones de contexto.** El sistema DEBE rechazar con **409** `producto_inactivo`
  todo movimiento sobre un producto inactivo, verificándolo dentro de la misma transacción
  (FR-022 de 003), y con **422** un depósito inactivo, una cantidad no entera, cero o con signo
  incorrecto para el tipo (`ENTRADA > 0`, `SALIDA` y `MERMA < 0`, `AJUSTE ≠ 0`), una línea
  repetida para el mismo producto y ubicación, un motivo incoherente con el tipo o reservado, o una
  ubicación inexistente, inactiva, de otro depósito o no permitida para el producto. Un depósito o
  producto inexistente DEBE rechazarse con **422** nombrando el id. Una cantidad no entera DEBE
  responder 422 con `"La cantidad de cada línea debe ser un número entero distinto de cero."`.
  Todo mensaje 409 y 422 del alta DEBE indicar qué hacer a continuación (Principio III); enumerar
  los valores válidos o el límite admitido cuenta como indicación.
  **Orden de evaluación** (gana el primer error): 422 si el cuerpo no es JSON válido o el tipo es
  inválido (sin eso no se sabe si es un `AJUSTE`); 403 por permiso; 422 del resto de la
  validación pura del cuerpo; dentro de la transacción, depósito, productos (409
  `producto_inactivo`), ubicaciones y stock (409 `stock_insuficiente`).
- **FR-029 Consulta.** El sistema DEBE exponer `GET /api/stock` (búsqueda `q` por nombre o SKU, filtros por depósito,
  categoría y `bajo_minimo`, paginado), `GET /api/stock/{producto_id}` (desglose por depósito),
  `GET /api/movimientos` (filtros por tipo, depósito, producto y rango de fechas, paginado) y
  `GET /api/movimientos/{id}`.
- **FR-030 Resultado concreto.** La respuesta de un movimiento DEBE incluir, por línea, el stock
  resultante en el depósito afectado (Principio III: confirmación con resultado concreto).

## Criterios de Éxito Medibles

- **SC-013** Cero registros de stock negativos: toda salida, merma o ajuste negativo sin stock
  suficiente se rechaza con 409 y el valor no cambia (escenarios 7.2, 7.4 y 7.5).
- **SC-014** Cero divergencias entre `productos.stock_actual` y la suma de `stock_por_deposito`
  después de cualquier secuencia de movimientos, incluida una fallida a mitad de camino y dos
  altas concurrentes sobre el mismo producto en ubicaciones distintas.
- **SC-015** Cero movimientos parciales: un request con una línea inválida no deja cabecera,
  líneas, auditoría ni cambios de stock.

## Supuestos

- Una ubicación por línea es opcional: sin ubicación, el stock se imputa al depósito.
- Un movimiento pertenece a un solo depósito; las transferencias entre depósitos son la spec 008.
- No hay reservas de stock ni movimientos pendientes de confirmación: todo movimiento confirmado
  aplica en el acto.
- La idempotencia por clave de request queda fuera de alcance; un reintento manual duplica el
  movimiento y es responsabilidad del operador.
- El costo del producto no cambia por movimientos; lo actualiza la recepción de compras (012).
- No hay lotes, vencimientos ni números de serie.

## Fuera de Alcance

- Transferencias entre depósitos (008) e inventario físico (009).
- Compras (012) y ventas (013), que generan movimientos con sus propios motivos.
- Reservas, pedidos en tránsito, serialización y trazabilidad por lote.
- Importación masiva (016) y asistente conversacional (017).

## Dependencias y orden de implementación

1. 001, 003, 005 y 006 cerradas.
2. Migración reservada: `0009_stock_movimientos.sql` (`stock_por_deposito`, `movimientos`,
   `movimiento_lineas`, `productos.stock_actual` e índices de consulta).
3. Es la base de 008, 009, 010, 011, 012, 013, 014, 016 y 017.
4. Los textos nuevos de `stock_insuficiente` y `producto_inactivo` (ampliación 2026-11-07) cambian
   también los literales citados en 008, 013 y 017, que deben alinearse.
