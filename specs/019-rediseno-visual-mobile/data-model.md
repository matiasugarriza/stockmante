# Data Model: Rediseño visual móvil en modo oscuro cyberpunk

**Feature**: 019-rediseno-visual-mobile | **Date**: 2026-11-07

Modelo de diseño, sin datos nuevos ni cambios de esquema. Todo lo que sigue es estado de interfaz o
derivación pura de datos que ya devuelve la API.

## 1. Estado de dispositivo

Campos calculados por `lib/dispositivo/dispositivo.ts`:

| Campo | Valores | Origen |
|---|---|---|
| `ancho` | píxeles CSS | `matchMedia`/`resize` |
| `alto` | píxeles CSS | `matchMedia`/`resize` |
| `layout` | `telefono` (< 768), `tablet` (768–1023), `escritorio` (≥ 1024) | `clasificarDispositivo(ancho)` |
| `esMovil` | booleano (`ancho < 1024`) | `esLayoutMovil(ancho)` |
| `orientacion` | `vertical` / `horizontal` | comparación `ancho`/`alto` |

Reglas:
- El layout móvil abarca teléfono y tablet; en ambos el tema es oscuro y la navegación es híbrida.
- La orientación horizontal no cambia la categoría de dispositivo; cambia la disposición (una o dos
  columnas).
- El estado se inicializa antes del render de las pantallas y se actualiza con `resize` y
  `orientationchange`, sin recargar.

## 2. Pantalla móvil

| # | Ruta | Categoría | Rol mínimo | Patrón móvil | Lista | Acción flotante |
|---|---|---|---|---|---|---|
| 1 | `/` | tablero | ambos | Cabecera + carrusel de métricas + widgets | Actividad reciente en tarjetas | Accesos rápidos, sin FAB |
| 2 | `/caja` | operación | ambos | Buscador + resultados en tarjetas + hoja de venta | `TarjetaInventario` | "Nueva venta" → `HojaVenta` |
| 3 | `/movimientos` | operación | ambos | Formulario plegado + historial | `TarjetaRegistro` | "Nuevo movimiento" → `HojaMovimiento` |
| 4 | `/transferencias` | operación | ambos | Formulario + historial | `TarjetaRegistro` | "Nueva transferencia" → foco al formulario |
| 5 | `/conteos` | operación | `GESTION` | Conteos e historial | `TarjetaRegistro` | "Abrir conteo" → foco al formulario |
| 6 | `/asistente` | operación | ambos | Conversación a ancho completo + entrada sticky | Mensajes en tarjetas | "Escribir" → foco a la entrada |
| 7 | `/admin/productos` | administración | `GESTION` | Filtros plegables + catálogo | `TarjetaRegistro` | Sin FAB (acción en encabezado) |
| 8 | `/admin/categorias` | administración | `GESTION` | Categorías | `TarjetaRegistro` | Sin FAB |
| 9 | `/admin/depositos` | administración | `GESTION` | Depósito + árbol de ubicaciones como lista jerárquica | Lista jerárquica | Sin FAB |
| 10 | `/admin/stock` | operación | `GESTION` | Filtros plegables + inventario | `TarjetaInventario` | "Ajuste de stock" → `HojaMovimiento` |
| 11 | `/admin/proveedores` | administración | `GESTION` | Proveedores | `TarjetaRegistro` | Sin FAB |
| 12 | `/admin/compras` | administración | `GESTION` | Compras | `TarjetaRegistro` | Sin FAB |
| 13 | `/admin/ventas` | administración | `GESTION` | Ventas | `TarjetaRegistro` | Sin FAB |
| 14 | `/admin/clientes` | administración | `GESTION` | Clientes | `TarjetaRegistro` | Sin FAB |
| 15 | `/admin/reportes` | administración | `GESTION` | Pestañas + reportes | Fila como tarjeta (CSS) | Sin FAB |
| 16 | `/admin/importacion` | administración | `GESTION` | Carga + resumen y errores | Fila como tarjeta (CSS) | Sin FAB |
| 17 | `/admin/usuarios` | administración | `GESTION` | Usuarios | `TarjetaRegistro` | Sin FAB |
| 18 | `/admin/auditoria` | administración | `GESTION` | Auditoría | `TarjetaRegistro` | Sin FAB |
| 19 | `/login` | acceso | público | Tarjeta centrada a pantalla completa | — | — |
| 20 | `/registro` | acceso | público | Tarjeta centrada a pantalla completa | — | — |
| 21 | `/verificar-correo` | acceso | público | Tarjeta centrada a pantalla completa | — | — |
| 22 | `/login` (cambio obligatorio) | acceso | autenticado con clave temporal | Tarjeta centrada, misma estética | — | — |

Reglas:
- Toda pantalla conserva campos, validaciones, mensajes, permisos y navegación de su spec de origen.
- Los roles son los de 018: `OPERACION` ve solo Operación; `GESTION` ve ambas familias.
- `HojaVenta` y `HojaMovimiento` son modales; el resto de las pantallas usan sus formularios
  existentes, ahora táctiles.

## 3. Destino de navegación móvil

| Campo | Tipo | Regla |
|---|---|---|
| `etiqueta` | texto | La de `ITEMS_NAVEGACION` |
| `ruta` | texto | La de `ITEMS_NAVEGACION` |
| `icono` | `NombreIcono` | El existente; Ventas, Clientes, Reportes e Importar reutilizan iconos de 018 |
| `grupo` | `operacion` / `administracion` | El existente |
| `roles` | `GESTION` / `OPERACION` | RBAC de 018 |
| `contador` | `alertas-stock` o ausente | Solo Stock, solo `GESTION` |
| `ubicacion` | `barra` / `panel` | Derivada |

Reglas de la barra inferior (`destinosBarraInferior`):
1. Siempre Inicio, Caja y Movimientos (si el rol los permite).
2. Cuarto destino: Stock para `GESTION`; Transferencias para `OPERACION`.
3. Quinto ítem: "Más", activo cuando la ruta vigente no está entre los cuatro anteriores.
4. El panel conserva los 17 destinos agrupados y el contador de alertas.
5. Ningún destino accesible en más de dos toques.

## 4. Tarjeta de inventario y tarjeta de registro

**TarjetaInventario** (`TarjetaInventario.svelte`):

| Campo | Fuente | Presentación |
|---|---|---|
| `sku` | `StockItem.sku` / `Producto.sku` | Monoespaciada, 1 línea |
| `nombre` | `producto_nombre` / `nombre` | Hasta 2 líneas con elipsis y `title` |
| `stock_actual` | API | Número + badge |
| `badge` | `clasificarStock` | Texto + color (Normal/Bajo/Crítico) |
| `contexto` | depósito, ubicación o fecha | 1 línea tenue |
| `acciones` | por pantalla | Botón ≥ 44×44 + menú desbordable |

**Badge de stock** (`clasificarStock(stockActual, bajoMinimo)`):

| Estado | Condición | Etiqueta | Semántica cromática |
|---|---|---|---|
| `critico` | `stock_actual <= 0` | "Crítico" | Rojo neón (relleno/borde), texto claro verificado |
| `bajo` | `bajo_minimo === true` y stock > 0 | "Bajo" | Naranja neón |
| `normal` | resto | "Normal" | Verde |

**TarjetaRegistro** (`TarjetaRegistro.svelte`): `titulo`, `subtitulo?`, `mono?`,
`badge?: { texto, variante }`, `campos: { etiqueta, valor }[]`, `acciones?: Snippet`,
`onabrir?: () => void`. Se usa en las pantallas de administración y en los historiales.

## 5. Métrica del tablero

| Campo | Origen |
|---|---|
| `etiqueta` | Texto fijo ("Stock total", "Productos activos", "Alertas críticas", "Movimientos de hoy") |
| `valor` | Endpoint existente (`listarStock`, `listarProductos`, `listarAlertas`, `listarMovimientos`) |
| `contexto` | Texto corto con referencia temporal o de estado |
| `acento` | `cian` (activo), `violeta` (secundario), `ambar`/`peligro` (alerta) |
| `rolMinimo` | `GESTION` para valorizado; ambos para el resto |

Reglas: si hay alertas, la tarjeta de alertas va primera; cada métrica resuelve su carga, vacío y
error; el carrusel ajusta una tarjeta por vista y expone botones y teclado.

## 6. Hoja de acción (movimiento y venta)

Estados:

| Estado | Entrada | Salida |
|---|---|---|
| `cerrada` | — | Abrir desde tarjeta/FAB |
| `abriendo` | Se piden depósitos y desglose de stock | `lista` o `error` |
| `lista` | Datos disponibles | `enviando` o `cerrada` |
| `enviando` | Confirmación válida | `exito` o `error` |
| `exito` | Respuesta 201 | Aviso + `cerrada` |
| `error` | 4xx/5xx | `lista` con mensaje concreto |

Reglas:
- El fondo no se desplaza y el foco queda dentro; al cerrar se restaura el foco y el scroll.
- Con datos sin guardar, cerrar pide confirmación (Escape, botón atrás, gesto o fondo).
- El botón de confirmar se deshabilita en `enviando` (FR-126).

## 7. Aviso flotante

| Campo | Regla |
|---|---|
| `id` | Secuencial |
| `mensaje` | Tipo, producto, depósito, cantidad y stock resultante |
| `variante` | `exito` o `error` |
| `duracionMs` | 5000 por defecto, inyectable para tests |
| `cierre` | Manual y automático; `role="status"`; una sola pulsación de destello |

## 8. Par de contraste

| Campo | Regla |
|---|---|
| `tokenTexto` | Token de primer plano (o valor literal documentado) |
| `tokenFondo` | Token de superficie |
| `uso` | Texto de cuerpo, texto grande, badge, borde de control, foco |
| `minimo` | 7 / 4,5 / 3 según uso |
| `ratio` | Calculado por el test; se publica en `contracts/tokens-movil.md` |
| `verificado` | Siempre por test automático |

Reglas:
- Ningún par aprobado puede bajar de su mínimo; el test falla si falta un par para un par de tokens
  usado por la capa móvil.
- Los tonos `peligro` y `violeta` crudos no pueden figurar como `tokenTexto` de uso textual.

## 9. Reglas de validación del registro rápido

- Cantidad vacía, cero, negativa, no numérica o con decimales cuando la unidad es entera: error junto
  al campo.
- Entrada: cantidad > 0.
- Salida y merma: valor absoluto ≤ stock disponible en el depósito elegido; se envía con signo
  negativo.
- Ajuste: solo `GESTION`; cantidad distinta de cero.
- Producto y depósito activos; motivo válido para el tipo (`MOTIVOS_POR_TIPO` de 007).
- Errores del servidor: se muestran dentro de la hoja con el mensaje concreto y no se pierde lo
  cargado.
