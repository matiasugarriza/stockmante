# Research: Ventas de caja y clientes (013)

Decisiones que cierran los hallazgos del análisis del 2026-11-07 (G1, C2–C4, I1–I8, U1–U4,
A1–A5, D1). Las cinco preguntas de negocio quedaron cerradas en `## Clarifications` de
`spec.md` (sesión 2026-11-07); acá se registra cómo se implementan y qué alternativas se
descartaron. El punto de partida es el código real: `0014_ventas.sql` aplicada, endpoints
`/api/ventas` y `/api/clientes` existentes, POS y pantallas admin **sin implementar**, sin tests
de ventas/clientes y con `tasks.md` marcando como completas tareas que no lo están.

## R1 — El precio unitario sale siempre del catálogo (A1, FR-050)

- **Decisión**: `services::ventas::crear` toma el precio de `productos.precio_venta`
  (`NUMERIC(12,2)`) y **rechaza** cualquier request que incluya `precio_unitario`, aun cuando
  coincida con el de lista, con `AppError::Validacion("El precio unitario se toma del catálogo;
  no se puede enviar en la venta.")` → 422 `validacion`. El campo sigue existiendo en
  `VentaLineaDto` como `Option<Decimal>` para poder detectar su presencia: `JsonValido` no usa
  `deny_unknown_fields`, así que un campo desconocido se ignoraría en silencio y no podría
  cumplirse el contrato.
- **Rationale**: la spec solo admite ventas al precio de lista (Supuestos) y el permiso de
  vender lo tienen `OPERACION` y `GESTION`; aceptar un precio del cliente habilita vender bajo
  costo sin auditoría. Congelar el precio en la línea al confirmar (FR-051) sigue igual.
- **Alternativas consideradas**: override solo `GESTION` (descartada en la clarificación: agrega
  un permiso y un caso de prueba que la spec no pide); precio libre (descartada por integridad).

## R2 — Líneas repetidas: se agregan por producto antes de validar (U1, SC-021, FR-050/FR-051)

- **Decisión**: `services::ventas::crear` normaliza las líneas con una función pura
  `agregar_lineas(&[NuevaVentaLinea]) -> Vec<(producto_id, cantidad)>` que suma cantidades por
  `producto_id` conservando el orden de primera aparición. La validación de stock, el bloqueo, el
  `movimiento_lineas` y la persistencia usan **una sola fila por producto**. Con 2 unidades y
  líneas `2 + 1` responde 409 `stock_insuficiente` con `"2 disponibles, 3 solicitadas."`; con
  líneas `1 + 1` responde 201, deja el stock en 0 y persiste una línea de cantidad 2.
- **Rationale**: hoy la validación recorre líneas contra el mismo saldo inicial: `2 + 1` pasa dos
  veces la comparación y la segunda escritura viola `CHECK (cantidad >= 0)` y
  `CHECK (stock_resultante >= 0)`, devolviendo 500 en lugar de `StockInsuficiente` 409 (violación
  del Principio V). Agregar es además el comportamiento esperado de un carrito.
- **Alternativas consideradas**: rechazar duplicados con 422 (descartada en la clarificación: el
  POS tendría que deduplicar y el error sería evitable); validar contra la suma pero persistir
  líneas repetidas (descartada: ticket y subtotales ambiguos y más filas para el mismo hecho).

## R3 — Autorización explícita por endpoint (G1, FR-055)

- **Decisión**: `handlers::ventas::anular_venta` y las escrituras de
  `handlers::clientes` (POST, PUT, activar, desactivar) reciben `RequiereGestion` en lugar de
  `AuthUsuario`; las lecturas de clientes y de ventas quedan con `AuthUsuario` (solo sesión
  vigente). Se documenta la matriz en el `//!` de cada handler y se agregan respuestas 403 a
  `utoipa`.
- **Rationale**: hoy todos los endpoints usan `AuthUsuario`, así que un `OPERACION`/`CAJERO`
  puede anular ventas y administrar clientes incumpliendo FR-055. `RequiereGestion` ya existe y
  es el patrón del resto del backend (productos, depósitos, compras).
- **Alternativas consideradas**: middleware por router (descartada: `/api/clientes` mezcla
  lectura abierta y escritura GESTION y obligaría a dos routers); chequeo manual repetido en cada
  handler (descartada: duplica lógica que el extractor ya encapsula).

## R4 — Documento de cliente duplicado: 409 `documento_duplicado` (A2, FR-049)

- **Decisión**: se agrega `AppError::DocumentoDuplicado` (mensaje `"Ya existe un cliente con ese
  documento."`) mapeada a 409 `documento_duplicado`; `services::clientes::{crear, actualizar}`
  la devuelven en lugar de `Validacion`. El `UNIQUE (documento)` de la migración se mantiene y
  cubre también a los clientes inactivos.
- **Rationale**: el resto del backend responde 409 para claves duplicadas (SKU, CUIT, email); la
  clarificación fijó ese contrato y el Principio V pide una variante específica por error de
  dominio.
- **Alternativas consideradas**: mantener 422 `validacion` (descartada en la clarificación);
  unicidad solo entre activos con índice parcial (descartada: un documento dado de baja quedaría
  reutilizable y confundiría el historial).

## R5 — Una sola migración y modelos separados (I3, I4)

- **Decisión**: la migración es `migrations/0014_ventas.sql` (la que existe y está aplicada); se
  corrigen plan y tasks que decían `0014_ventas_caja.sql` / `backend/migrations/`. `Cliente` y
  `ListaClientes` se mueven a `backend/src/models/cliente.rs` y se registran en
  `models/mod.rs`; `models/venta.rs` conserva `Venta`, `VentaLinea`, `VentaDetalle`, `ListaVentas`.
- **Rationale**: tres nombres para la misma migración rompen la verificación de esquema del
  Principio V; un archivo `venta.rs` que contiene clientes contradice el Principio II y el plan
  anterior ya prometía `models/cliente.rs`.
- **Alternativas consideradas**: dejar `Cliente` en `venta.rs` y corregir el plan (descartada: el
  nombre del archivo deja de prometer su contenido); renombrar la migración (descartada: ya está
  aplicada).

## R6 — Alcance de interfaz: POS y administración (I6, I7, FR-054)

- **Decisión**: `/caja` pasa a ser POS completo (carrito, descuento, medio de pago,
  confirmación); se crean `/admin/ventas` (listado, detalle, anular) y `/admin/clientes` (CRUD);
  se agregan los enlaces en `frontend/src/routes/+layout.svelte` (ventas y clientes dentro del
  bloque `GESTION`). Toda llamada pasa por `$lib/api/ventas.ts` y `$lib/api/clientes.ts` con
  schemas Zod en `$lib/schemas/`.
- **Rationale**: el plan anterior ya listaba las pantallas admin y FR-054 ahora las exige; sin
  clientes de API tipados el POS incumpliría el Principio II.
- **Alternativas consideradas**: solo API (descartada en la clarificación: FR-054 pide POS);
  POS + admin clientes sin admin ventas (descartada: la anulación quedaría inaccesible desde la
  UI).

## R7 — Tests RED antes de completar el comportamiento (C2, C3, P-IV)

- **Decisión**: antes de tocar los servicios se escriben y se ven fallar:
  `backend/tests/integration/test_ventas_http.rs` (Escenarios 13.1–13.9 y SC-021),
  `backend/tests/integration/test_clientes_http.rs` (FR-049, 409 `documento_duplicado`),
  casos de ventas/clientes en `backend/tests/integration/test_rbac_matriz.rs` (FR-055),
  `backend/tests/unit/test_venta_validacion.rs` (agregación y rechazo de precio) y
  `frontend/src/lib/api/ventas.test.ts` / `clientes.test.ts`. Gate final: `cargo test`,
  `cargo sqlx prepare --check`, `npm test`, `npm run check`, `npm run build`.
- **Rationale**: `tasks.md` marcaba tests inexistentes; el Principio IV exige RED→GREEN y las
  cinco tareas falsamente completas tienen que reabrirse.
- **Alternativas consideradas**: tests después de implementar (descartada por P-IV; si apareciera
  la necesidad de un waiver habría que registrarlo en la constitución).

## R8 — Trazabilidad y documentación (I1, I2, I3, U3)

- **Decisión**: cada tarea de `tasks.md` cita el FR/SC que cubre y una ruta de archivo concreta;
  `docs/api.md` documenta `/api/ventas` y `/api/clientes` con permisos y errores; la spec deja de
  decir "sin plan.md ni tasks.md" (ya corregido en la clarificación).
- **Rationale**: el análisis no pudo mapear tareas a requisitos y `docs/api.md` no tenía ninguna
  mención de ventas; sin trazabilidad, `/speckit-analyze` vuelve a marcar cobertura desconocida.
- **Alternativas consideradas**: no documentar y confiar en OpenAPI (descartada: `docs/api.md` es
  el índice que usan los módulos anteriores).

## R9 — El detalle de venta devuelve el stock resultante por línea (13.1, P-III)

- **Decisión**: `VentaLinea` gana `stock_resultante: Option<i32>`; `services::ventas::{crear,
  obtener, anular}` lo completan con un `LEFT JOIN LATERAL` a `movimiento_lineas` del movimiento
  más reciente de la venta (`SALIDA` al crear, `ENTRADA` si está anulada).
- **Rationale**: el Escenario 13.1 exige que el 201 devuelva "número de venta, total y stock
  resultante por producto", y el Principio III exige mostrar el resultado concreto de toda acción
  sobre inventario. Hoy `VentaDetalle` no incluye saldo alguno.
- **Alternativas consideradas**: devolver el stock actual del producto (descartada: no es el
  resultante de la operación y cambia con ventas posteriores); agregar la columna a
  `venta_lineas` (descartada: migración innecesaria cuando el dato ya está en
  `movimiento_lineas`).

## Orden de bloqueo y concurrencia (FR-051)

Se mantiene el orden de 007: `depositos FOR UPDATE` → `productos WHERE id = ANY(...) ORDER BY id
FOR UPDATE` → filas de `stock_por_deposito FOR UPDATE`. La agregación de R2 reduce a una sola
escritura por producto y evita el caso de dos actualizaciones sobre la misma fila.
