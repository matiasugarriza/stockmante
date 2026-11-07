# Especificación de Funcionalidad: Reportes, kardex y valorización

**Spec**: `014-reportes-kardex` | **Caso de uso**: US14 | **Prioridad**: P2

**Creado**: 2026-10-26 | **Estado**: Implementado y verificado (2026-11-07) 
| **Módulo**: 3 — Abastecimiento y ventas

**Origen**: "reportes financieros" figura fuera del alcance del Módulo 1; con movimientos, compras y
ventas ya registrados, los reportes son la salida natural de esos datos.

**Depende de**: 007 (movimientos), 008 (transferencias), 012 (compras y costos), 013 (ventas).

## Clarifications

### Session 2026-11-07

- Q: ¿Quién puede consultar cada reporte? → A: Los cuatro reportes, en JSON y en CSV, exigen
  `GESTION`; `OPERACION` recibe 403 `sin_permisos` (opción A).
- Q: ¿Qué cuenta como "unidades salidas" para la rotación? → A: Movimientos tipo `SALIDA` excepto
  los de motivo `TRANSFERENCIA`; mermas y ajustes no son rotación (opción A).
- Q: ¿Cómo se resuelve un rango sin `desde`/`hasta`? → A: Rotación y ventas exigen ambos (422 si
  falta alguno); el kardex los acepta opcionales: sin `desde` = desde el primer movimiento, sin
  `hasta` = sin límite superior. Ningún reporte toma la hora del sistema (opción A).
- Q: ¿En qué zona horaria se interpretan las fechas y se agrupan las ventas por día? → A: Día local
  de Buenos Aires, offset fijo UTC-03:00 (igual que 011); `desde` y `hasta` en `YYYY-MM-DD` son
  inclusivos por día completo (opción A).

## Escenarios de Usuario y Pruebas *(obligatorio)*

### User Story 14 - Entender qué pasó con el inventario (Prioridad: P2)

Como **Gerente**, quiero ver el kardex de un producto, el stock valorizado y la rotación por
período, y exportarlos, para decidir compras y detectar diferencias sin armar planillas.

**Por qué esta prioridad**: los datos ya existen; sin reportes el sistema registra pero no ayuda a
decidir. Es la base del control de gestión y de la futura conciliación contable.

**Escenario 14.1 — Kardex por producto**

- **Dado** un producto con entradas, salidas y ajustes en un rango de fechas,
- **Cuando** se consulta `GET /api/reportes/kardex?producto_id=1&desde=2026-10-01&hasta=2026-10-31`,
- **Entonces** obtiene el saldo inicial (suma de movimientos anteriores al 2026-10-01 local), las
  líneas ordenadas por fecha con tipo, motivo, depósito, cantidad con signo y saldo corrido, y el
  saldo final; las líneas del 31 de octubre local están incluidas. Sin `hasta` y sin
  `deposito_id`, el saldo final coincide con `productos.stock_actual`.

**Escenario 14.2 — Stock valorizado**

- **Dado** el stock actual y los costos,
- **Cuando** se consulta `GET /api/reportes/stock-valorizado`,
- **Entonces** obtiene, por cada producto con stock mayor a cero, cantidad, costo unitario
  (`productos.costo`) y valor, más el total general.

**Escenario 14.3 — Rotación**

- **Dado** un rango de fechas,
- **Cuando** se consulta `GET /api/reportes/rotacion?desde=2026-10-01&hasta=2026-10-31`,
- **Entonces** obtiene por producto las unidades salidas (movimientos `SALIDA` sin motivo
  `TRANSFERENCIA`), el stock inicial, el final, el promedio y el índice de rotación (unidades
  salidas sobre stock promedio del período).

**Escenario 14.4 — Ventas por período**

- **Dado** un rango con ventas confirmadas y anuladas,
- **Cuando** se consulta `GET /api/reportes/ventas?desde=2026-10-01&hasta=2026-10-31`,
- **Entonces** obtiene total vendido, cantidad de ventas y desglose por día local y por producto,
  contando solo ventas `CONFIRMADA`.

**Escenario 14.5 — Exportación**

- **Dado** cualquiera de los cuatro reportes,
- **Cuando** se agrega `formato=csv`,
- **Entonces** responde `text/csv; charset=utf-8` con encabezados en español y los mismos datos,
  sin paginar; en ventas, `agrupar=dia` (predeterminado) o `agrupar=producto` elige el desglose.

**Escenario 14.6 — Rango inválido**

- **Dado** `desde` posterior a `hasta`,
- **Cuando** se consulta,
- **Entonces** responde **422** `validacion` con
  `"La fecha desde no puede ser posterior a la fecha hasta."`.

**Escenario 14.7 — Operario**

- **Dado** una sesión `OPERACION`,
- **Cuando** consulta cualquiera de los cuatro reportes, en JSON o en CSV,
- **Entonces** responde **403** `sin_permisos`.

### Casos borde

- Rotación o ventas sin `desde` o sin `hasta` → **422** `validacion` con
  `"Indicá las fechas desde y hasta del período (formato AAAA-MM-DD)."`.
- Fecha con formato inválido → **422** `validacion` con
  `"El parámetro desde debe estar en formato ISO 8601 (AAAA-MM-DD)."` (o `hasta`).
- `formato` distinto de `json` o `csv`, o `agrupar` distinto de `dia` o `producto` → **422**
  `validacion` que nombra los valores aceptados.
- Kardex de un producto inexistente → **404** `recurso_no_encontrado` con
  `"El producto no existe."`; un producto inactivo sí tiene kardex.
- Kardex con `deposito_id` → solo movimientos de ese depósito; sin `hasta`, el saldo final
  coincide con `stock_por_deposito` de ese producto en ese depósito.
- Kardex sin movimientos en el rango → líneas vacías y saldo final igual al inicial.
- Un rango sin datos → reporte vacío con totales en cero, nunca error.
- Las transferencias aparecen en el kardex (salida en origen, entrada en destino) y suman cero en
  el kardex global.
- Un nombre de producto con comas, comillas o saltos de línea se exporta entre comillas dobles
  (RFC 4180).

## Requisitos Funcionales

- **FR-056 Kardex.** El sistema DEBE exponer el historial de un producto, opcionalmente filtrado
  por depósito y por rango de fechas, con saldo inicial, tipo, motivo, depósito, cantidad con
  signo, saldo corrido y saldo final, ordenado cronológicamente (fecha, movimiento, línea).
- **FR-057 Stock valorizado.** El sistema DEBE exponer el stock actual (suma de
  `stock_por_deposito`) valorizado a `productos.costo`, por producto con stock mayor a cero y con
  total general.
- **FR-058 Rotación y ventas.** El sistema DEBE exponer, para un rango obligatorio, la rotación por
  producto (unidades salidas por movimientos `SALIDA` sin motivo `TRANSFERENCIA`, sobre el stock
  promedio del período) y el resumen de ventas `CONFIRMADA` por día local y por producto.
- **FR-059 Exportación, permisos y validación.** Cada reporte DEBE aceptar `formato=csv` (UTF-8,
  encabezados en español, RFC 4180). Todas las rutas, en JSON y CSV, DEBEN exigir `GESTION`. Las
  fechas se interpretan como días locales UTC-03:00 inclusivos; los rangos invertidos, las fechas
  mal formadas y los parámetros con valores no admitidos DEBEN responder 422 con los mensajes de
  14.6 y de Casos borde.

## Criterios de Éxito Medibles

- **SC-022** El kardex reconstruye el stock: para cualquier producto, `saldo_inicial + Σ cantidad
  de las líneas = saldo_final`, y sin `hasta` ni `deposito_id` el `saldo_final` es igual a
  `productos.stock_actual` (verificado en la suite de integración con entradas, salidas, mermas,
  ajustes y transferencias).
- **SC-031** Los cuatro reportes, en JSON y en CSV, responden 403 a `OPERACION` y 200 a `GESTION`
  (verificado en la matriz RBAC).

## Supuestos

- La valorización usa `productos.costo` (último costo de compra, que 012 actualiza al recibir), no
  costo promedio ponderado.
- El stock promedio del período se calcula como el promedio simple entre saldo inicial y final;
  con promedio cero el índice es cero.
- Los reportes leen datos transaccionales; no hay tablas materializadas ni cache de reportes.
- Ni el JSON ni el CSV se paginan; el volumen está acotado por el rango de fechas consultado y el
  CSV se genera en memoria.
- El total por producto de ventas suma los subtotales de línea (antes del descuento de la venta);
  el total general suma `ventas.total` (después del descuento).
- No hay reportes contables, de impuestos ni de cuentas por cobrar/pagar.

## Fuera de Alcance

- Contabilidad, balance, IVA y libros.
- Reportes programados, envío por correo o dashboards con gráficos históricos.
- Comparación entre períodos o presupuestos.
- Exportación a Excel o PDF.
- Zonas horarias configurables por usuario o por depósito.

## Dependencias y orden de implementación

1. 007, 008, 012 y 013 implementadas.
2. Sin migración propia; usa los índices creados en 0009 y 0014 (evaluar uno adicional solo si
   `EXPLAIN ANALYZE` lo justifica).
3. Alimenta el asistente (017) con consultas agregadas.
