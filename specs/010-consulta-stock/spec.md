# Especificación de Funcionalidad: Consulta y tablero de stock

**Spec**: `010-consulta-stock` | **Caso de uso**: US10 | **Prioridad**: P1

**Creado**: 2026-10-26 | **Estado**: Clarificado (2026-11-07); `plan.md` y `tasks.md` regenerados
tras `/speckit analyze` | **Módulo**: 2 — Operación de stock

**Origen**: la constitución fija `stock_por_deposito` como fuente de verdad y
`productos.stock_actual` como cache; falta la superficie de consulta para operarios y gestores.

**Depende de**: 007 (stock y movimientos), 005 (depósitos y ubicaciones).

**Relación con 007**: `GET /api/stock` y `GET /api/stock/{producto_id}` nacieron en 007 (FR-029).
Esta spec **extiende** ese contrato (FR-038/FR-039: `faltante`, valorizado por perfil) y es la dueña
de las pantallas; 007 FR-029 sigue siendo la referencia del contrato base.

## Clarifications

### Session 2026-11-07

- Q: En `/caja`, ¿de qué depósito se muestra el stock disponible? → A: Selector de depósitos
  activos en `/caja`; se muestra el stock de ese depósito (sin migración, sin depósito fijo).
- Q: ¿Cómo se protege el valorizado a costo, reservado a `GESTION`? → A: El backend lo omite:
  `/api/stock` devuelve `costo` y `valorizado` solo a sesiones `GESTION`; para `OPERACION` vienen
  `null`. Test HTTP para ambos perfiles.
- Q: ¿El listado debe traer el desglose por depósito en cada fila? → A: No; el listado trae
  `stock_actual` y, con `deposito_id`, `stock_deposito`. El desglose por depósito y ubicación queda
  en el detalle `GET /api/stock/{producto_id}`, que la UI abre desde la fila.
- Q: ¿Se agrega un índice para la búsqueda por SKU antes de medir SC-018? → A: Medir primero con
  `EXPLAIN ANALYZE` y prueba HTTP sobre 10.000 productos; solo si no cumple se agrega una migración
  con índice trigram sobre `productos.sku`.
- Q: ¿Dónde se calcula la diferencia contra el mínimo? → A: Campo `faltante` en la API,
  `max(stock_minimo − stock_actual, 0)`, calculado en el backend con test unitario.

## Escenarios de Usuario y Pruebas *(obligatorio)*

### User Story 10 - Ver cuánto hay y dónde está (Prioridad: P1)

Como **usuario con sesión `OPERACION` o `GESTION`**, quiero consultar el stock por producto,
depósito y ubicación, y ver qué productos están por debajo del mínimo, para decidir una reposición
o atender una venta sin adivinar.

**Por qué esta prioridad**: un movimiento sin visibilidad de stock obliga a consultar la base a
mano; es la contraparte natural de 007 y la base de alertas y reportes.

**Escenario 10.1 — Listado de stock**

- **Dado** una sesión vigente y 200 productos,
- **Cuando** consulta `GET /api/stock?deposito_id=1&pagina=1`,
- **Entonces** obtiene la página ordenada por nombre con `stock_actual` (global) y
  `stock_deposito` (del depósito 1) en cada fila.

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
- **Entonces** obtiene solo esos productos, con el mínimo y `faltante = stock_minimo − stock_actual`.

**Escenario 10.5 — Producto inexistente**

- **Dado** un id que no existe,
- **Cuando** consulta el detalle,
- **Entonces** responde **404** `recurso_no_encontrado` con
  `"Producto no encontrado."`.

**Escenario 10.6 — Pantallas**

- **Dado** una sesión `OPERACION`,
- **Cuando** abre `/caja` y elige un depósito en el selector,
- **Entonces** ve el stock disponible en ese depósito para los productos que busca, y no ve costo
  ni valorizado.
- **Dado** una sesión `GESTION`,
- **Cuando** abre `/admin/stock`,
- **Entonces** ve el listado con filtros (texto, depósito, categoría, bajo mínimo, activo), mínimo,
  faltante, costo y valorizado, y puede abrir el desglose de un producto.

**Escenario 10.7 — Valorizado reservado a gestión**

- **Dado** una sesión `OPERACION`,
- **Cuando** consulta `GET /api/stock`,
- **Entonces** cada fila trae `costo: null` y `valorizado: null`; con una sesión `GESTION` vienen
  `costo` (de `productos.costo`) y `valorizado = costo × stock_actual`
  (o `× stock_deposito` si se filtró por depósito).

### Casos borde y errores

- Sin depósitos activos, `/caja` muestra "No hay depósitos activos. Pedile a un gestor que active
  uno en Administración › Depósitos." y no consulta stock.
- Si falla la carga del stock o de los depósitos por red, la pantalla dice qué falló y qué hacer
  ("No se pudo cargar el stock: revisá la conexión y tocá Reintentar."), con botón Reintentar; nunca
  descarta el error en silencio (Principio III).
- `stock_minimo = 0` nunca cuenta como bajo mínimo y su `faltante` es 0.
- Producto inactivo con stock remanente: se lista con `activo=false` cuando el filtro lo incluye.

## Requisitos Funcionales

- **FR-038 Listado de stock.** El sistema DEBE exponer `GET /api/stock` paginado con filtros por
  `q` (nombre o SKU, sin distinguir mayúsculas), `deposito_id`, `categoria_id`, `bajo_minimo`,
  `activo`, `producto_ids` (hasta 50, para la caja) y orden por nombre; cada fila DEBE incluir `stock_actual`, `stock_minimo`, `bajo_minimo`,
  `faltante = max(stock_minimo − stock_actual, 0)` y, si se envió `deposito_id`, `stock_deposito`
  de ese depósito. El desglose completo por depósito y ubicación NO va en el listado: lo da FR-039.
- **FR-039 Detalle por producto.** El sistema DEBE exponer `GET /api/stock/{producto_id}` con el
  total, el desglose por depósito y ubicación, el mínimo y `faltante`; un id inexistente responde 404
  con `"Producto no encontrado."`.
- **FR-040 Vistas y permisos.** El frontend DEBE ofrecer `/admin/stock` (gestión) y un selector de
  depósito en `/caja` (operación) que muestre el stock disponible en el depósito elegido. La lectura
  DEBE estar disponible para cualquier sesión vigente. El valorizado a costo DEBE reservarse a
  `GESTION` **en el backend**: `GET /api/stock` incluye `costo` y `valorizado` solo para sesiones de
  familia `GESTION` y los devuelve `null` para `OPERACION`; ocultarlo solo en la UI no alcanza.

## Criterios de Éxito Medibles

- **SC-018** Con 10.000 productos y 3 depósitos, una página de `GET /api/stock` (con y sin `q`)
  responde en menos de 1 segundo, medido con `EXPLAIN ANALYZE` y prueba HTTP. Índices de referencia:
  `idx_productos_nombre_trgm` (0007) e `idx_stock_producto`/`idx_stock_deposito` (0009). Si la
  búsqueda por SKU no cumple, se agrega una migración con índice trigram sobre `productos.sku`.

## Supuestos

- El listado usa el cache `productos.stock_actual` (filtro `bajo_minimo` incluido) y
  `stock_por_deposito` para `stock_deposito` y el desglose; la suite contrasta que el desglose suma
  el cache (SC-014 de 007).
- El valorizado usa `productos.costo` (no hay costo promedio ponderado todavía).
- La consulta no muestra movimientos; eso es el kardex (014).
- Los productos inactivos se muestran con `activo=false` y stock remanente, para poder corregirlos.
- El depósito elegido en `/caja` es una preferencia de pantalla, no un dato persistido en la BD.

## Fuera de Alcance

- Proyecciones de demanda o sugerencias automáticas de reposición.
- Reservas, stock comprometido o disponible para venta.
- Reportes históricos y valorización por período (014).
- Desglose por depósito dentro de cada fila del listado.

## Dependencias y orden de implementación

1. 007 implementada (tablas y cache).
2. Sin migración propia salvo que SC-018 falle por la búsqueda de SKU (ver SC-018).
3. Habilita 011 (alertas) y 014 (reportes).
