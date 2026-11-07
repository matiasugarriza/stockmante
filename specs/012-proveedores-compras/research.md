# Research: Proveedores y compras (012)

Decisiones que cierran los hallazgos del análisis del 2026-11-07 (C1–C4, I1–I5, U1–U5, A1). Las
cinco preguntas de negocio quedaron cerradas en `## Clarifications` de `spec.md` (sesión
2026-11-07); acá se registra cómo se implementan y qué alternativas se descartaron. El plan
anterior describía tablas (`ordenes_compra`, `orden_compra_lineas`, `recepciones`) y un endpoint
(`/recibir`) que nunca se implementaron: este documento parte del código real.

## R1 — Anulación concurrente: orden de bloqueo de 007 y descuento condicional (C3, SC-021)

- **Decisión**: `services::compras::anular` adopta el orden de bloqueo de
  `services::movimientos::crear`: `compras` (`FOR UPDATE`) → `depositos` (`FOR UPDATE`) →
  `productos WHERE id = ANY($1) ORDER BY id FOR UPDATE` → filas de `stock_por_deposito`. El
  descuento se hace con
  `UPDATE stock_por_deposito SET cantidad = cantidad - $n ... WHERE ... AND cantidad >= $n RETURNING cantidad`;
  si no devuelve fila, responde `StockInsuficiente` con el saldo leído bajo bloqueo.
- **Rationale**: hoy el chequeo lee sin bloqueo y el `UPDATE` asigna un valor absoluto. Una salida
  de 007 que corre en paralelo deja el saldo por debajo de lo previsto, el `CHECK (cantidad >= 0)`
  aborta la transacción y el cliente recibe un 500. Bloquear los productos en el mismo orden que 007
  serializa ambas operaciones sin riesgo de interbloqueo, y la guarda condicional es la que exige
  el Principio V aunque el bloqueo ya alcance.
- **Alternativas consideradas**: solo agregar `FOR UPDATE` al chequeo (descartada: no cubre una fila
  de stock inexistente y no cumple la guarda condicional); aislamiento `SERIALIZABLE` con reintento
  (descartada: agrega un bucle de reintentos que el resto del backend no usa).

## R2 — Respuesta de confirmar y anular con el movimiento (C4, clarificación 5)

- **Decisión**: `POST /confirmar` y `POST /anular` devuelven `ResultadoCompra { compra, lineas,
  movimiento }`, donde `movimiento` es el `MovimientoDetalle` de 007 (depósito, motivo y líneas con
  `cantidad_delta` y `stock_resultante`), obtenido con `services::movimientos::obtener` después del
  commit. `CompraDetalle` no cambia para `GET`, `POST` y `PUT`.
- **Rationale**: el Principio III pide mostrar el resultado concreto de toda acción sobre inventario.
  Reusar `MovimientoDetalle` evita un DTO paralelo y deja al frontend con un schema que ya existe en
  `$lib/schemas/movimientos.ts`. El cambio es aditivo: los clientes que leen `compra` y `lineas`
  siguen funcionando.
- **Alternativas consideradas**: agregar `movimiento: Option<...>` a `CompraDetalle` (descartada:
  ensucia el detalle de borradores con un campo siempre nulo); que la UI consulte el stock después
  (descartada en la clarificación: más llamadas y una carrera entre la confirmación y la lectura).

## R3 — Nuevas variantes de error (clarificaciones 2 y 3)

- **Decisión**: `AppError::CompraNoAnulable(String)` → 409 `compra_no_anulable`, con el mensaje que
  corresponde a `BORRADOR` o a `ANULADA`; `AppError::ProveedorInactivo(String)` → 409
  `proveedor_inactivo`, con el nombre del proveedor. `CompraNoEditable` queda solo para editar y
  eliminar.
- **Rationale**: el Principio V pide una variante específica por error de dominio y el III un
  mensaje que diga qué hacer. `proveedor_inactivo` no colisiona con `proveedor_no_disponible`, que
  pertenece al SSO.
- **Alternativas consideradas**: reusar `CompraNoEditable` y `Validacion` con mejor texto
  (descartada en la clarificación).

## R4 — Filtros del listado: 422 en lugar de ignorar (clarificación 4)

- **Decisión**: `parsear_fecha` pasa a devolver `Result<Option<DateTime<Utc>>, AppError>`; un valor
  no vacío que no sea `AAAA-MM-DD` ni RFC 3339 responde `Validacion("El parámetro 'desde' debe tener
  formato AAAA-MM-DD.")`. `estado` se valida contra `BORRADOR|CONFIRMADA|ANULADA`. La función queda
  pura y testeable sin HTTP.
- **Rationale**: un filtro ignorado en silencio muestra un listado más amplio que el pedido y el
  usuario no se entera.
- **Alternativas consideradas**: ignorar y documentar (descartada en la clarificación).

## R5 — Límites de `numero` y `costo_unitario` (hallazgo nuevo de la verificación de esquema)

- **Decisión**: validar en `models::proveedor` que `numero` tenga como máximo 40 caracteres y que
  `costo_unitario` esté entre 0 y 9.999.999.999,99; fuera de rango → 422 `validacion` nombrando el
  campo.
- **Rationale**: `compras.numero` es `VARCHAR(40)` y no se valida, así que un texto más largo hoy
  termina en 500. `compra_lineas.costo_unitario` es `NUMERIC(18,2)`, pero al confirmar se copia a
  `productos.costo NUMERIC(12,2)`: un costo de 11 dígitos enteros pasa el borrador y revienta en la
  confirmación. Validar en el borrador falla antes y con un mensaje claro.
- **Alternativas consideradas**: ampliar `productos.costo` a `NUMERIC(18,2)` (descartada: cambia el
  esquema de 003 sin un requisito que lo pida).

## R6 — Número de compra (clarificación 1, I3)

- **Decisión**: sin migración. El número interno es `compras.id` y la UI lo muestra como «Compra
  #id»; `numero` se rotula «Nº factura/remito del proveedor».
- **Rationale**: el esquema ya cumple; solo había un supuesto mal redactado.

## R7 — Waiver y verificación de esquema retroactivos (C1, C2)

- **Decisión**: T001–T023 se implementaron sin registrar el RED y sin una tarea de verificación de
  esquema. Se propone el waiver **W-003** (Principio IV, alcance T006/T020/T023 de 012), que debe
  aprobarse con `/speckit constitution`; esta feature no puede enmendar la constitución. La
  verificación de esquema se registra en `data-model.md` con fecha 2026-11-07. Desde la fase 6 de
  `tasks.md`, cada corrección empieza por un test que falla.
- **Rationale**: la constitución prohíbe ignorar una violación por costumbre y prohíbe reconstruir
  un RED artificial; el camino previsto es un waiver acotado.

## R8 — Ubicación de la recepción (U3)

- **Decisión**: la recepción imputa al depósito sin ubicación (`ubicacion_id NULL`), igual que un
  movimiento de 007 sin ubicación. Las ubicaciones permitidas de 003/005 no se validan.
- **Rationale**: 007 define la ubicación como opcional; elegirla al recibir es una funcionalidad que
  la spec deja fuera de alcance.
