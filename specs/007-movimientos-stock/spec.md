# Especificación de Funcionalidad: Movimientos de stock

**Spec**: `007-movimientos-stock` | **Caso de uso**: US7 | **Prioridad**: P1

**Creado**: 2026-10-26 | **Estado**: Implementado | **Módulo**: 2 — Operación de stock

**Origen**: el README y `specs/README.md` declaran "movimientos de stock" fuera del alcance del
Módulo 1. La constitución (Principio V) ya fija el modelo: `stock_por_deposito` como fuente de
verdad y `productos.stock_actual` como cache denormalizado.

**Depende de**: 001 (sesión y RBAC), 003 (productos), 005 (depósitos y ubicaciones), 006 (auditoría).

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
  `"Stock insuficiente en Principal: 3 disponibles, 5 solicitadas."` y el stock no cambia.

**Escenario 7.3 — Merma parcial**

- **Dado** un producto con 20 unidades,
- **Cuando** se registra una `MERMA` de 2 unidades con motivo `VENCIMIENTO`,
- **Entonces** el stock baja a 18, la respuesta lo informa y la acción queda auditada.

**Escenario 7.4 — Ajuste reservado a gestión**

- **Dado** una sesión `OPERACION`,
- **Cuando** intenta registrar un `AJUSTE`,
- **Entonces** responde **403** `sin_permisos`; con sesión `GESTION` el ajuste procede y puede ser
  positivo o negativo.

**Escenario 7.5 — Movimiento multi-línea atómico**

- **Dado** un movimiento con tres líneas donde la tercera no tiene stock suficiente,
- **Cuando** se envía el request,
- **Entonces** responde **409** y **no** queda ninguna línea escrita ni ningún stock modificado.

**Escenario 7.6 — Ubicación inválida**

- **Dado** una ubicación de otro depósito o no asignada al producto (005),
- **Cuando** se registra el movimiento con esa ubicación,
- **Entonces** responde **422** `validacion` indicando que la ubicación no pertenece al depósito o
  no está permitida para el producto.

**Escenario 7.7 — Movimiento sobre entidades inactivas**

- **Dado** un producto inactivo,
- **Cuando** se intenta un movimiento,
- **Entonces** responde **409** `producto_inactivo` con
  `"El producto está inactivo y no admite movimientos."` y no se escribe nada (FR-022 de 003).
- **Dado** un depósito inactivo,
- **Cuando** se intenta un movimiento,
- **Entonces** responde **422** `validacion` y no se escribe nada.

**Escenario 7.8 — Consulta de stock y de movimientos**

- **Dado** una sesión autenticada,
- **Cuando** consulta `GET /api/stock/{producto_id}` o `GET /api/movimientos`,
- **Entonces** obtiene el desglose por depósito y el historial paginado con sus líneas.

## Requisitos Funcionales

- **FR-024 Registro de movimientos.** El sistema DEBE registrar movimientos de tipo `ENTRADA`,
  `SALIDA`, `MERMA` y `AJUSTE`, con una cabecera (tipo, depósito, motivo, observación opcional) y
  una o más líneas (producto, ubicación opcional, `cantidad_delta` entera distinta de cero). El
  motivo DEBE pertenecer al catálogo `COMPRA`, `VENTA`, `MERMA`, `AJUSTE`, `DEVOLUCION`,
  `TRANSFERENCIA`, `INVENTARIO`, `IMPORTACION` u `OTRO`; con `OTRO` la observación es obligatoria.
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
  (FR-022 de 003), y con **422** un depósito inactivo, una cantidad no entera o menor o igual a
  cero, una línea repetida para el mismo producto y ubicación, o una ubicación inexistente, de otro
  depósito o no permitida para el producto.
- **FR-029 Consulta.** El sistema DEBE exponer `GET /api/stock` (filtros por producto, depósito,
  categoría y `bajo_minimo`, paginado), `GET /api/stock/{producto_id}` (desglose por depósito),
  `GET /api/movimientos` (filtros por tipo, depósito, producto y rango de fechas, paginado) y
  `GET /api/movimientos/{id}`.
- **FR-030 Resultado concreto.** La respuesta de un movimiento DEBE incluir, por línea, el stock
  resultante en el depósito afectado (Principio III: confirmación con resultado concreto).

## Criterios de Éxito Medibles

- **SC-013** Cero registros de stock negativos: toda salida o merma sin stock suficiente se rechaza
  con 409 y el valor no cambia (verificado con 100% de los casos de la suite).
- **SC-014** Cero divergencias entre `productos.stock_actual` y la suma de `stock_por_deposito`
  después de cualquier secuencia de movimientos, incluida una fallida a mitad de camino.
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
