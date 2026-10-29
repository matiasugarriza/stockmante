# Especificación de Funcionalidad: Proveedores y compras

**Spec**: `012-proveedores-compras` | **Caso de uso**: US12 | **Prioridad**: P2

**Creado**: 2026-10-26 | **Estado**: Especificado (sin `plan.md` ni `tasks.md`; se generan al
implementar) | **Módulo**: 3 — Abastecimiento y ventas

**Origen**: el abastecimiento es la fuente natural de las entradas de stock; sin compras, toda
reposición depende de un movimiento manual sin proveedor ni costo asociado.

**Depende de**: 007 (movimientos de stock), 003 (productos y categorías).

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
  `proveedor_duplicado` con `"Ya existe un proveedor con ese nombre."`.

**Escenario 12.2 — Compra en borrador**

- **Dado** un proveedor activo y dos productos,
- **Cuando** se crea una compra con líneas (cantidad y costo unitario),
- **Entonces** la compra queda `BORRADOR`, el total es la suma de los subtotales y el stock **no**
  cambia.

**Escenario 12.3 — Confirmar recepción**

- **Dado** una compra en borrador,
- **Cuando** se confirma,
- **Entonces** en una sola transacción se genera un movimiento `ENTRADA` (motivo `COMPRA`, con
  referencia a la compra), sube el stock del depósito elegido, se actualiza el costo del producto y
  la compra pasa a `CONFIRMADA`.

**Escenario 12.4 — Confirmación duplicada**

- **Dado** una compra ya confirmada,
- **Cuando** se intenta confirmar de nuevo,
- **Entonces** responde **409** `compra_ya_confirmada` con
  `"La compra ya fue confirmada; no se puede confirmar dos veces."` y el stock no se duplica.

**Escenario 12.5 — Anular una compra confirmada**

- **Dado** una compra confirmada con stock suficiente para revertir,
- **Cuando** se anula,
- **Entonces** se genera un movimiento `SALIDA` (motivo `DEVOLUCION`) por las mismas cantidades y
  la compra pasa a `ANULADA`.
- **Dado** que el stock ya no alcanza para revertir,
- **Cuando** se anula,
- **Entonces** responde **409** `stock_insuficiente` y la compra sigue `CONFIRMADA`.

**Escenario 12.6 — Compra en borrador editable**

- **Dado** una compra `BORRADOR`,
- **Cuando** se edita o se elimina,
- **Entonces** los cambios se permiten y no afectan stock; una compra `CONFIRMADA` o `ANULADA` no
  se edita (409).

**Escenario 12.7 — Operario**

- **Dado** una sesión `OPERACION`,
- **Cuando** accede a proveedores o compras,
- **Entonces** responde **403** `sin_permisos`.

## Requisitos Funcionales

- **FR-044 CRUD de proveedores.** El sistema DEBE permitir crear, listar, consultar, editar, dar de
  baja lógica y reactivar proveedores con nombre único, CUIT opcional (11 dígitos, único si está
  presente), email, teléfono y dirección. La baja lógica DEBE impedir usarlo en compras nuevas.
- **FR-045 Compras en borrador.** El sistema DEBE permitir crear una compra con proveedor, depósito
  de destino y una o más líneas (producto, cantidad entera positiva, costo unitario), calculando el
  total. Una compra en `BORRADOR` no DEBE afectar stock ni costo.
- **FR-046 Confirmación.** Confirmar una compra DEBE ejecutar en una única transacción: validar
  productos y depósito activos, escribir un movimiento `ENTRADA` con motivo `COMPRA` y referencia a
  la compra, actualizar `productos.costo` con el último costo recibido, marcar la compra como
  `CONFIRMADA` y auditar la acción. Una compra confirmada no se edita ni se confirma dos veces.
- **FR-047 Anulación.** Anular una compra confirmada DEBE generar un movimiento `SALIDA` con motivo
  `DEVOLUCION` por las mismas cantidades; si el stock no alcanza, DEBE responder 409
  `stock_insuficiente` sin cambiar la compra.
- **FR-048 Consultas y permisos.** El sistema DEBE exponer `GET /api/proveedores`,
  `GET /api/proveedores/{id}`, `GET /api/compras` (filtros por proveedor, estado y fechas,
  paginado) y `GET /api/compras/{id}` con sus líneas. Todas las rutas DEBEN exigir `GESTION`.

## Criterios de Éxito Medibles

- **SC-020** Cero recepciones sin movimiento: toda compra `CONFIRMADA` tiene exactamente un
  movimiento `ENTRADA` asociado y el stock aumentó exactamente las cantidades compradas.

## Supuestos

- El costo del producto se actualiza al último costo de compra (no hay costo promedio ponderado).
- La compra tiene un solo depósito de destino y no maneja impuestos, percepciones ni moneda
  extranjera.
- No hay facturación electrónica ni integración contable; el comprobante del proveedor es externo.
- No hay órdenes de compra con aprobación previa: el borrador y la confirmación son el flujo.
- El número de compra es interno y secuencial; no reemplaza al número de factura del proveedor.

## Fuera de Alcance

- Órdenes de compra con aprobación, licitaciones o comparación de proveedores.
- Facturación electrónica, IVA, retenciones y cuentas por pagar.
- Recepción parcial o en tránsito.
- Costo promedio ponderado o valuación FIFO.

## Dependencias y orden de implementación

1. 007 implementada (entradas de stock).
2. Migración reservada: `0013_proveedores_compras.sql` (`proveedores`, `compras`, `compra_lineas`).
3. Alimenta la valorización y rotación (014) y la auditoría ampliada (015).
