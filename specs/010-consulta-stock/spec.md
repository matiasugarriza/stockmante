# Especificación de Funcionalidad: Consulta y tablero de stock

**Spec**: `010-consulta-stock` | **Caso de uso**: US10 | **Prioridad**: P1

**Creado**: 2026-10-26 | **Estado**: Especificado (sin `plan.md` ni `tasks.md`; se generan al
implementar) | **Módulo**: 2 — Operación de stock

**Origen**: la constitución fija `stock_por_deposito` como fuente de verdad y
`productos.stock_actual` como cache; falta la superficie de consulta para operarios y gestores.

**Depende de**: 007 (stock y movimientos), 005 (depósitos y ubicaciones).

## Escenarios de Usuario y Pruebas *(obligatorio)*

### User Story 10 - Ver cuánto hay y dónde está (Prioridad: P1)

Como **Operario o Gerente**, quiero consultar el stock por producto, depósito y ubicación, y ver
qué productos están por debajo del mínimo, para decidir una reposición o atender una venta sin
adivinar.

**Por qué esta prioridad**: un movimiento sin visibilidad de stock obliga a consultar la base a
mano; es la contraparte natural de 007 y la base de alertas y reportes.

**Escenario 10.1 — Listado de stock**

- **Dado** una sesión vigente y 200 productos,
- **Cuando** consulta `GET /api/stock?deposito_id=1&pagina=1`,
- **Entonces** obtiene la página con stock global y por depósito, ordenada por nombre.

**Escenario 10.2 — Búsqueda por SKU o nombre**

- **Dado** un producto `YERBA-1K`,
- **Cuando** consulta `GET /api/stock?q=yerba`,
- **Entonces** lo encuentra aunque el stock sea cero.

**Escenario 10.3 — Detalle con desglose**

- **Dado** un producto con stock en dos depósitos,
- **Cuando** consulta `GET /api/stock/{producto_id}`,
- **Entonces** ve el total y el desglose por depósito y ubicación, con el mínimo configurado.

**Escenario 10.4 — Bajo mínimo**

- **Dado** productos con `stock_actual` menor que `stock_minimo`,
- **Cuando** consulta `GET /api/stock?bajo_minimo=true`,
- **Entonces** obtiene solo esos productos, con la diferencia y el mínimo.

**Escenario 10.5 — Producto inexistente**

- **Dado** un id que no existe,
- **Cuando** consulta el detalle,
- **Entonces** responde **404** `recurso_no_encontrado` con
  `"Producto no encontrado."`.

**Escenario 10.6 — Pantallas**

- **Dado** una sesión `OPERACION`,
- **Cuando** abre `/caja`,
- **Entonces** ve el stock disponible del depósito de venta y puede buscar productos.
- **Dado** una sesión `GESTION`,
- **Cuando** abre `/admin/stock`,
- **Entonces** ve el listado completo con filtros, mínimo y valorizado a costo.

## Requisitos Funcionales

- **FR-038 Listado de stock.** El sistema DEBE exponer `GET /api/stock` paginado con filtros por
  `q` (nombre o SKU, sin distinguir mayúsculas), `deposito_id`, `categoria_id`, `bajo_minimo`,
  `activo` y orden por nombre; cada fila DEBE incluir `stock_actual`, el mínimo y el desglose por
  depósito.
- **FR-039 Detalle por producto.** El sistema DEBE exponer `GET /api/stock/{producto_id}` con el
  total, el desglose por depósito y ubicación y el mínimo; un id inexistente responde 404 con
  `"Producto no encontrado."`.
- **FR-040 Vistas y permisos.** El frontend DEBE ofrecer `/admin/stock` (gestión, con valorizado a
  costo) y mostrar el stock disponible en `/caja` (operación). La lectura DEBE estar disponible para
  cualquier sesión vigente; el valorizado a costo DEBE reservarse a `GESTION`.

## Criterios de Éxito Medibles

- **SC-018** Con 10.000 productos y 3 depósitos, una página de `GET /api/stock` responde en menos
  de 1 segundo con los índices de la migración 0009 (medido con `EXPLAIN ANALYZE` y prueba HTTP).

## Supuestos

- El stock se lee siempre de `stock_por_deposito`; `productos.stock_actual` solo se usa como cache
  de listado y se contrasta en la suite.
- El valorizado usa `productos.costo` (no hay costo promedio ponderado todavía).
- La consulta no muestra movimientos; eso es el kardex (014).
- Los productos inactivos se muestran con `activo=false` y stock remanente, para poder corregirlos.

## Fuera de Alcance

- Proyecciones de demanda o sugerencias automáticas de reposición.
- Reservas, stock comprometido o disponible para venta.
- Reportes históricos y valorización por período (014).

## Dependencias y orden de implementación

1. 007 implementada (tablas y cache).
2. Sin migración propia: usa la 0009 y sus índices.
3. Habilita 011 (alertas) y 014 (reportes).
