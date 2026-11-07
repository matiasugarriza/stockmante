# Especificación de Funcionalidad: Proveedores y compras

**Spec**: `012-proveedores-compras` | **Caso de uso**: US12 | **Prioridad**: P2

**Creado**: 2026-10-26 | **Estado**: Implementada (T001-T023) y aclarada el 2026-11-07; quedan
tareas de cierre de hallazgos de análisis en `tasks.md` (fase 6) | **Módulo**: 3 — Abastecimiento y
ventas

**Origen**: el abastecimiento es la fuente natural de las entradas de stock; sin compras, toda
reposición depende de un movimiento manual sin proveedor ni costo asociado.

**Depende de**: 007 (movimientos de stock), 003 (productos y categorías), 005 (depósitos).

## Clarifications

### Session 2026-11-07

- Q: ¿Qué representa el número de compra? → A: El número interno y secuencial es `compras.id`;
  `compras.numero` es el número de factura o remito del proveedor, opcional y cargado por el usuario.
- Q: ¿Qué responde la API al anular una compra en `BORRADOR` o ya `ANULADA`? → A: 409
  `compra_no_anulable`, con un mensaje específico según el estado.
- Q: ¿Qué responde la API si se crea, edita o confirma una compra con un proveedor dado de baja? →
  A: 409 `proveedor_inactivo`, coherente con `producto_inactivo`.
- Q: ¿Qué hace `GET /api/compras` ante una fecha mal formada o un estado inexistente? → A: 422
  `validacion`, con un mensaje que nombra el parámetro y el formato esperado.
- Q: ¿De dónde sale el resultado concreto (depósito, producto, stock resultante) al confirmar o
  anular? → A: La respuesta de `POST /confirmar` y `POST /anular` incluye el movimiento generado con
  sus líneas y `stock_resultante`; la UI lo muestra.

## Escenarios de Usuario y Pruebas *(obligatorio)*

### User Story 12 - Comprar y recibir mercadería con proveedor (Prioridad: P2)

Como **Gerente**, quiero registrar proveedores y compras, y confirmar la recepción para que el
stock entre automáticamente con su costo, para saber a quién le compré, a qué precio y qué quedó
en depósito.

**Por qué esta prioridad**: completa el ciclo de entrada del inventario y deja el costo del
producto actualizado para la valorización y los reportes.

**Escenario 12.1 — Alta de proveedor**

- **Dado** un gerente,
- **Cuando** crea un proveedor con nombre, CUIT, email y teléfono,
- **Entonces** responde **201**; un nombre repetido responde **409**
  `proveedor_duplicado` con `"Ya existe un proveedor con ese nombre."`, y un CUIT ya usado por otro
  proveedor (en alta o en edición) responde **409** `cuit_duplicado`.

**Escenario 12.2 — Compra en borrador**

- **Dado** un proveedor activo y dos productos,
- **Cuando** se crea una compra con líneas (cantidad y costo unitario) y, opcionalmente, el número
  de factura o remito del proveedor,
- **Entonces** la compra queda `BORRADOR` con su número interno (`id`), el total es la suma de los
  subtotales y el stock **no** cambia.
- **Dado** un proveedor dado de baja,
- **Cuando** se crea, edita o confirma una compra con él,
- **Entonces** responde **409** `proveedor_inactivo` con un mensaje que nombra al proveedor y
  sugiere reactivarlo o elegir otro, y nada cambia.

**Escenario 12.3 — Confirmar recepción**

- **Dado** una compra en borrador,
- **Cuando** se confirma,
- **Entonces** en una sola transacción se genera un movimiento `ENTRADA` (motivo `COMPRA`, con
  referencia a la compra), sube el stock del depósito elegido, se actualiza el costo del producto y
  la compra pasa a `CONFIRMADA`; la respuesta incluye el movimiento con depósito, productos y
  `stock_resultante` por línea, y la interfaz muestra ese resultado.

**Escenario 12.4 — Confirmación duplicada**

- **Dado** una compra ya confirmada,
- **Cuando** se intenta confirmar de nuevo,
- **Entonces** responde **409** `compra_ya_confirmada` con
  `"La compra ya fue confirmada; no se puede confirmar dos veces."` y el stock no se duplica.

**Escenario 12.5 — Anular una compra confirmada**

- **Dado** una compra confirmada con stock suficiente para revertir,
- **Cuando** se anula,
- **Entonces** se genera un movimiento `SALIDA` (motivo `DEVOLUCION`) por las mismas cantidades, la
  compra pasa a `ANULADA` y la respuesta incluye el movimiento con su `stock_resultante` por línea.
- **Dado** que el stock ya no alcanza para revertir,
- **Cuando** se anula,
- **Entonces** responde **409** `stock_insuficiente` y la compra sigue `CONFIRMADA`.
- **Dado** que otra operación descuenta stock del mismo producto y depósito al mismo tiempo,
- **Cuando** se anula,
- **Entonces** la anulación espera a la otra operación y responde éxito o **409**
  `stock_insuficiente` según el saldo resultante; nunca **500** y el stock nunca queda negativo.
- **Dado** una compra en `BORRADOR` o ya `ANULADA`,
- **Cuando** se intenta anular,
- **Entonces** responde **409** `compra_no_anulable` con
  `"La compra está en borrador; no hay stock que revertir. Si no la vas a usar, eliminala."` o
  `"La compra ya fue anulada; no se puede anular dos veces."`, y nada cambia.

**Escenario 12.6 — Compra en borrador editable**

- **Dado** una compra `BORRADOR`,
- **Cuando** se edita o se elimina,
- **Entonces** los cambios se permiten y no afectan stock; una compra `CONFIRMADA` o `ANULADA` no
  se edita ni se elimina (409 `compra_no_editable`).

**Escenario 12.7 — Operario**

- **Dado** una sesión `OPERACION`,
- **Cuando** accede a proveedores o compras,
- **Entonces** responde **403** `sin_permisos`.

**Escenario 12.8 — Filtros inválidos en el listado**

- **Dado** un gerente,
- **Cuando** lista compras con `desde`/`hasta` que no son `AAAA-MM-DD` ni RFC 3339, o con un
  `estado` distinto de `BORRADOR`, `CONFIRMADA` o `ANULADA`,
- **Entonces** responde **422** `validacion` con un mensaje que nombra el parámetro y el formato o
  los valores aceptados.

## Requisitos Funcionales

- **FR-044 CRUD de proveedores.** El sistema DEBE permitir crear, listar, consultar, editar, dar de
  baja lógica y reactivar proveedores con nombre único, CUIT opcional (11 dígitos, único si está
  presente, en alta y en edición: 409 `cuit_duplicado`), email, teléfono y dirección. La baja
  lógica DEBE impedir usarlo en compras: crear, editar o confirmar una compra con un proveedor
  inactivo responde 409 `proveedor_inactivo`.
- **FR-045 Compras en borrador.** El sistema DEBE permitir crear una compra con proveedor, depósito
  de destino, número de factura o remito del proveedor opcional (`numero`, texto libre de hasta 40
  caracteres) y una o más líneas (producto, cantidad entera positiva, costo unitario entre 0 y
  9.999.999.999,99 —el rango de `productos.costo`—, sin productos repetidos), calculando el total.
  Un `numero` o un costo fuera de rango DEBE responder 422 `validacion` nombrando el campo, nunca
  500. El número interno de la compra es su `id`. Una
  compra en `BORRADOR` no DEBE afectar stock ni costo.
- **FR-046 Confirmación.** Confirmar una compra DEBE ejecutar en una única transacción: validar
  proveedor, productos y depósito activos, escribir un movimiento `ENTRADA` con motivo `COMPRA` y
  referencia a la compra, imputando el stock al depósito sin ubicación (como 007), actualizar
  `productos.costo` con el último costo recibido, marcar la compra como `CONFIRMADA` y auditar la
  acción. La respuesta DEBE incluir la compra y el movimiento generado con sus líneas
  (`producto`, `cantidad_delta`, `stock_resultante`) y su depósito. Una compra confirmada no se
  edita ni se confirma dos veces.
- **FR-047 Anulación.** Anular una compra confirmada DEBE generar, en una única transacción, un
  movimiento `SALIDA` con motivo `DEVOLUCION` por las mismas cantidades, con la misma forma de
  respuesta que FR-046. La validación de stock DEBE hacerse sobre filas bloqueadas y el descuento
  con guarda condicional (`cantidad >= n`); si el stock no alcanza, DEBE responder 409
  `stock_insuficiente` sin cambiar la compra. Anular una compra en `BORRADOR` o ya `ANULADA` DEBE
  responder 409 `compra_no_anulable` con un mensaje específico según el estado.
- **FR-048 Consultas y permisos.** El sistema DEBE exponer `GET /api/proveedores`,
  `GET /api/proveedores/{id}`, `GET /api/compras` (filtros por proveedor, estado y fechas,
  paginado) y `GET /api/compras/{id}` con sus líneas. Un filtro `desde`/`hasta` mal formado o un
  `estado` inexistente DEBE responder 422 `validacion` nombrando el parámetro. Todas las rutas DEBEN
  exigir `GESTION`.

## Criterios de Éxito Medibles

- **SC-020** Cero recepciones sin movimiento: toda compra `CONFIRMADA` tiene exactamente un
  movimiento `ENTRADA` asociado y el stock aumentó exactamente las cantidades compradas.
- **SC-021** Cero errores 500 por concurrencia: con anulaciones y salidas simultáneas sobre el mismo
  producto y depósito, cada operación termina en éxito o 409 y ningún saldo queda negativo.

## Supuestos

- El costo del producto se actualiza al último costo de compra (no hay costo promedio ponderado).
- La compra tiene un solo depósito de destino y no maneja impuestos, percepciones ni moneda
  extranjera.
- No hay facturación electrónica ni integración contable; el comprobante del proveedor es externo.
- No hay órdenes de compra con aprobación previa: el borrador y la confirmación son el flujo.
- El número interno de la compra es `compras.id` (secuencial); `compras.numero` guarda el número de
  factura o remito del proveedor y no tiene unicidad.
- La recepción no elige ubicación: el stock entra al depósito sin ubicación, igual que un movimiento
  de 007 sin ubicación.

## Fuera de Alcance

- Órdenes de compra con aprobación, licitaciones o comparación de proveedores.
- Facturación electrónica, IVA, retenciones y cuentas por pagar.
- Recepción parcial o en tránsito.
- Costo promedio ponderado o valuación FIFO.
- Asignar ubicación dentro del depósito al recibir.

## Dependencias y orden de implementación

1. 007 implementada (entradas de stock).
2. Migración: `migrations/0013_proveedores_compras.sql` (`proveedores`, `compras`, `compra_lineas`
   y `movimientos.compra_id`).
3. Alimenta la valorización y rotación (014) y la auditoría ampliada (015).
