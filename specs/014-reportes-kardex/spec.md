# Especificación de Funcionalidad: Reportes, kardex y valorización

**Spec**: `014-reportes-kardex` | **Caso de uso**: US14 | **Prioridad**: P2

**Creado**: 2026-10-26 | **Estado**: Especificado (sin `plan.md` ni `tasks.md`; se generan al
implementar) | **Módulo**: 3 — Abastecimiento y ventas

**Origen**: "reportes financieros" figura fuera del alcance del Módulo 1; con movimientos, compras y
ventas ya registrados, los reportes son la salida natural de esos datos.

**Depende de**: 007 (movimientos), 012 (compras y costos), 013 (ventas).

## Escenarios de Usuario y Pruebas *(obligatorio)*

### User Story 14 - Entender qué pasó con el inventario (Prioridad: P2)

Como **Gerente**, quiero ver el kardex de un producto, el stock valorizado y la rotación por
período, y exportarlos, para decidir compras y detectar diferencias sin armar planillas.

**Por qué esta prioridad**: los datos ya existen; sin reportes el sistema registra pero no ayuda a
decidir. Es la base del control de gestión y de la futura conciliación contable.

**Escenario 14.1 — Kardex por producto**

- **Dado** un producto con entradas, salidas y ajustes en un rango de fechas,
- **Cuando** se consulta `GET /api/reportes/kardex?producto_id=1&desde=2026-10-01&hasta=2026-10-31`,
- **Entonces** obtiene las líneas ordenadas por fecha con tipo, motivo, cantidad con signo y saldo
  corrido; el saldo final coincide con `stock_actual`.

**Escenario 14.2 — Stock valorizado**

- **Dado** el stock actual y los costos,
- **Cuando** se consulta `GET /api/reportes/stock-valorizado`,
- **Entonces** obtiene cantidad, costo unitario, valor por producto y total general.

**Escenario 14.3 — Rotación**

- **Dado** un rango de fechas,
- **Cuando** se consulta `GET /api/reportes/rotacion?desde=&hasta=`,
- **Entonces** obtiene por producto las unidades salidas y el índice de rotación (salidas sobre
  stock promedio del período).

**Escenario 14.4 — Ventas por período**

- **Dado** un rango con ventas,
- **Cuando** se consulta `GET /api/reportes/ventas?desde=&hasta=`,
- **Entonces** obtiene total vendido, cantidad de ventas y desglose por día y por producto.

**Escenario 14.5 — Exportación**

- **Dado** cualquiera de los reportes,
- **Cuando** se agrega `formato=csv`,
- **Entonces** responde `text/csv; charset=utf-8` con encabezados en español y los mismos datos,
  sin paginar.

**Escenario 14.6 — Rango inválido**

- **Dado** `desde` posterior a `hasta`,
- **Cuando** se consulta,
- **Entonces** responde **422** `validacion` con
  `"La fecha desde no puede ser posterior a la fecha hasta."`.

**Escenario 14.7 — Operario**

- **Dado** una sesión `OPERACION`,
- **Cuando** consulta reportes,
- **Entonces** responde **403** `sin_permisos`.

## Requisitos Funcionales

- **FR-056 Kardex.** El sistema DEBE exponer el historial de un producto por rango de fechas con
  tipo, motivo, depósito, cantidad con signo y saldo corrido, ordenado cronológicamente.
- **FR-057 Stock valorizado.** El sistema DEBE exponer el stock actual valorizado a
  `productos.costo`, por producto y con total general.
- **FR-058 Rotación y ventas.** El sistema DEBE exponer la rotación por producto (unidades salidas
  sobre stock promedio del período) y el resumen de ventas por día y por producto.
- **FR-059 Exportación y permisos.** Cada reporte DEBE aceptar `formato=csv` (UTF-8, encabezados en
  español) y todas las rutas DEBEN exigir `GESTION`. Los rangos inválidos DEBEN responder 422 con
  el mensaje del escenario 14.6.

## Criterios de Éxito Medibles

- **SC-022** El kardex reconstruye el stock: para cualquier producto y rango, el saldo final del
  kardex más el saldo inicial coincide exactamente con `stock_actual` (verificado sobre datos de la
  suite).

## Supuestos

- La valorización usa `productos.costo` (último costo de compra), no costo promedio ponderado.
- El stock promedio del período se calcula como el promedio simple entre saldo inicial y final.
- Los reportes leen datos transaccionales; no hay tablas materializadas ni cache de reportes.
- El CSV se genera en memoria y está acotado por el rango de fechas consultado.
- No hay reportes contables, de impuestos ni de cuentas por cobrar/pagar.

## Fuera de Alcance

- Contabilidad, balance, IVA y libros.
- Reportes programados, envío por correo o dashboards con gráficos históricos.
- Comparación entre períodos o presupuestos.
- Exportación a Excel o PDF.

## Dependencias y orden de implementación

1. 007, 012 y 013 implementadas.
2. Sin migración propia; usa los índices creados en 0009 y 0014 (evaluar uno adicional solo si
   `EXPLAIN ANALYZE` lo justifica).
3. Alimenta el asistente (017) con consultas agregadas.
