# Especificación de Funcionalidad: Ventas de caja y clientes

**Spec**: `013-ventas-caja` | **Caso de uso**: US13 | **Prioridad**: P1

**Creado**: 2026-10-26 | **Estado**: Especificado (sin `plan.md` ni `tasks.md`; se generan al
implementar) | **Módulo**: 3 — Abastecimiento y ventas

**Origen**: "facturación" figura fuera del alcance del Módulo 1; la pantalla `/caja` de 004 ya
existe como búsqueda, pero no registra la salida de stock que produce una venta.

**Depende de**: 003 (productos y precios), 004 (búsqueda de caja), 005 (depósitos), 007
(movimientos de stock).

## Escenarios de Usuario y Pruebas *(obligatorio)*

### User Story 13 - Vender en la caja y descontar stock (Prioridad: P1)

Como **Cajero**, quiero registrar una venta con varios productos, cobrarla y que el stock baje en
el acto, para que el inventario y la caja reflejen la misma realidad sin planillas paralelas.

**Por qué esta prioridad**: es la razón de ser del inventario; sin ventas el sistema solo acumula
stock y no cierra el ciclo operativo del negocio.

**Escenario 13.1 — Venta completa**

- **Dado** un cajero con sesión, el depósito "Principal" y dos productos con stock y precio,
- **Cuando** registra una venta con cantidades y medio de pago `EFECTIVO`,
- **Entonces** responde **201** con número de venta, total y stock resultante por producto, y el
  stock del depósito baja exactamente las cantidades vendidas.

**Escenario 13.2 — Stock insuficiente**

- **Dado** un producto con 2 unidades en el depósito de venta,
- **Cuando** se venden 3,
- **Entonces** responde **409** `stock_insuficiente` con
  `"Stock insuficiente en Principal: 2 disponibles, 3 solicitadas."` y no se registra la venta.

**Escenario 13.3 — Venta multi-línea atómica**

- **Dado** una venta con tres líneas donde la última no tiene stock,
- **Cuando** se envía,
- **Entonces** responde **409** y no queda venta, líneas, movimiento ni cambio de stock.

**Escenario 13.4 — Producto inactivo**

- **Dado** un producto inactivo,
- **Cuando** se intenta vender,
- **Entonces** responde **422** `validacion` con
  `"El producto no está disponible para la venta."`.

**Escenario 13.5 — Precio congelado**

- **Dado** una venta confirmada y un cambio posterior del precio del producto,
- **Cuando** se consulta la venta,
- **Entonces** muestra el precio vigente al momento de la venta, no el actual.

**Escenario 13.6 — Anulación**

- **Dado** una venta confirmada,
- **Cuando** un gerente la anula,
- **Entonces** se genera un movimiento `ENTRADA` (motivo `DEVOLUCION`) que repone el stock y la
  venta pasa a `ANULADA`.
- **Dado** una venta ya anulada,
- **Cuando** se intenta anular de nuevo,
- **Entonces** responde **409** `venta_ya_anulada` con
  `"La venta ya está anulada; no se puede anular dos veces."`.

**Escenario 13.7 — Descuento y total**

- **Dado** una venta con subtotal 10.000 y descuento 1.000,
- **Cuando** se confirma,
- **Entonces** el total es 9.000 y el descuento no puede superar el subtotal (422 si lo hace).

**Escenario 13.8 — Cliente opcional**

- **Dado** una venta con cliente registrado,
- **Cuando** se consulta,
- **Entonces** muestra el cliente; una venta sin cliente es válida (consumidor final).

## Requisitos Funcionales

- **FR-049 CRUD de clientes.** El sistema DEBE permitir crear, listar, consultar, editar, dar de
  baja lógica y reactivar clientes con nombre, documento opcional (único si está presente), email,
  teléfono y dirección. La baja lógica no DEBE impedir consultar ventas históricas.
- **FR-050 Venta y numeración.** El sistema DEBE registrar ventas con número secuencial único,
  depósito, cliente opcional, medio de pago (`EFECTIVO`, `TARJETA`, `TRANSFERENCIA`), líneas
  (producto, cantidad entera positiva, precio unitario) y descuento; el total DEBE ser
  `subtotal - descuento` y el descuento no puede superarlo.
- **FR-051 Descuento de stock.** Confirmar una venta DEBE ejecutar en una única transacción: validar
  stock suficiente en el depósito de venta, escribir un movimiento `SALIDA` (motivo `VENTA` con
  referencia a la venta) y actualizar el stock y el cache. El precio unitario DEBE congelarse en la
  línea de venta al confirmar.
- **FR-052 Anulación.** Anular una venta DEBE generar un movimiento `ENTRADA` (motivo `DEVOLUCION`)
  por las mismas cantidades, marcar la venta como `ANULADA` y auditar; una venta anulada no se
  reanula.
- **FR-053 Medios de pago.** El medio de pago DEBE pertenecer al catálogo `EFECTIVO`, `TARJETA` o
  `TRANSFERENCIA`; no se registran pagos divididos ni cuentas corrientes.
- **FR-054 Consultas y POS.** El sistema DEBE exponer `GET /api/ventas` (filtros por fecha,
  depósito, cliente y estado, paginado) y `GET /api/ventas/{id}` con líneas y totales; el frontend
  DEBE convertir `/caja` en el punto de venta (carrito, descuento, medio de pago y confirmación).
- **FR-055 Permisos.** `OPERACION` y `GESTION` DEBEN poder vender; anular ventas y administrar
  clientes DEBE estar reservado a `GESTION`; la consulta de ventas DEBE exigir sesión vigente.

## Criterios de Éxito Medibles

- **SC-021** Cero ventas confirmadas con stock negativo y descuento exacto: para toda venta, el
  stock del depósito baja exactamente la cantidad vendida y la suma de líneas coincide con el
  subtotal.

## Supuestos

- La venta se imputa a un depósito (el punto de venta); no se busca stock en otros depósitos
  automáticamente.
- No hay impuestos, percepciones, moneda extranjera ni redondeos por línea.
- El ticket es una vista imprimible del navegador; no hay impresora fiscal ni integración AFIP.
- No hay devoluciones parciales ni cambios: la anulación revierte la venta completa.
- Los precios salen de `productos.precio_venta` y se congelan en la línea al confirmar.

## Fuera de Alcance

- Facturación electrónica, IVA y libros contables.
- Cuentas corrientes, pagos divididos y arqueo de caja.
- Devoluciones parciales, cambios y notas de crédito.
- Integración con balanzas, lectores o impresoras fiscales.

## Dependencias y orden de implementación

1. 007 implementada; 004 disponible como base de búsqueda.
2. Migración reservada: `0014_ventas.sql` (`clientes`, `ventas`, `venta_lineas`, secuencia de
   número).
3. Alimenta reportes de ventas y rotación (014) y la auditoría ampliada (015).
