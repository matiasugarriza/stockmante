# Contrato: interfaz móvil

**Feature**: 019-rediseno-visual-mobile | **Date**: 2026-11-07 | **Estado**: contractual

Amplía `specs/018-rediseno-visual-desktop/contracts/ui.md`. Las pantallas, sus patrones y sus roles
están en [data-model.md §2](../data-model.md); este documento fija componentes, navegación, estados y
reglas de accesibilidad.

## 1. Puntos de corte

| Categoría | Ancho | Layout | Navegación | Tema |
|---|---|---|---|---|
| Teléfono | < 768 px | Una columna | Barra inferior + panel | Oscuro cyberpunk |
| Tablet | 768–1023 px | Una o dos columnas | Barra inferior + panel | Oscuro cyberpunk |
| Escritorio | ≥ 1024 px | 018 sin cambios | Barra lateral de 018 | Claro/Oscuro/Sistema |

- La capa móvil se activa con `@media (max-width: 1023px)`; el estado lógico usa
  `esLayoutMovil(ancho) = ancho < 1024`.
- La orientación horizontal de teléfono y tablet reorganiza listas en dos columnas; no cambia la
  categoría.

## 2. Componentes

### `MarcoApp.svelte` (modificado)

Props actuales (`titulo`, `alertasStock`, `children`, `acciones`) se conservan. Si `$layout.esMovil`,
renderiza `MarcoMovil`; si no, el marco de 018. No cambia la API pública para las páginas.

### `MarcoMovil.svelte` (nuevo)

| Prop | Tipo | Regla |
|---|---|---|
| `titulo` | `string` | Título de la ruta |
| `alertasStock` | `number` | Contador para Stock/barra |
| `children` | `Snippet` | Contenido de la página |

Compone: `CabeceraMovil`, `main` con `scroll-padding`, `BarraInferior`, `BarraLateral` en modo
panel, `AvisoFlotante` y el botón "Saltar al contenido".

### `CabeceraMovil.svelte` (nuevo)

- Título de la ruta con elipsis; botón de búsqueda (`aria-label="Buscar producto"`), menú de usuario.
- Sin selector de tema (FR-104).
- `position: sticky` con área segura superior.

### `BarraInferior.svelte` (nuevo)

| Prop | Tipo |
|---|---|
| `rutaActual` | `string` |
| `familiaRol` | `FamiliaRol` |
| `alertasStock` | `number` |
| `onabrirPanel` | `() => void` |

Renderiza `destinosBarraInferior(...)`: 4 destinos + "Más" (`ruta: '#panel'`, icono `mas`).
`aria-current="page"` en el activo; contador de alertas en Stock; alto 64 px + área segura.

### `BarraLateral.svelte` (modificado)

Conserva sus props. En móvil: panel fijo de ancho completo hasta `--sidebar-ancho`, abre con
`movilAbierta`, cierra con fondo/Escape/navegación, atrapa el foco, bloquea el scroll y devuelve el
foco al disparador. En escritorio, comportamiento actual.

### `BuscadorMovil.svelte` (nuevo)

- Se abre desde la cabecera; overlay a pantalla completa con campo enfocado y resultados.
- Debounce 250 ms, máximo 20 resultados, descarte de respuestas viejas.
- Usa `buscarInventario(q)` (composición de D6) y muestra `TarjetaInventario`.
- Estados: inicial (sugerencia), cargando (esqueletos), vacío (mensaje + limpiar), error (reintento).
- Al elegir un resultado: navega a `/admin/stock` con el producto o abre la hoja de movimiento si la
  pantalla lo permite.

### `ListaTarjetas.svelte` (nuevo, genérico)

| Prop | Tipo | Regla |
|---|---|---|
| `items` | `T[]` | Datos de la página |
| `etiqueta` | `string` | Nombre accesible de la lista |
| `cargando` | `boolean` | 3 esqueletos |
| `error` | `string \| null` | Aviso con reintento |
| `vacio` | `Snippet` | Estado vacío con acción |
| `tarjeta` | `Snippet<[T]>` | Tarjeta por ítem |
| `pie` | `Snippet` | Paginación o "cargar más" |

Reglas: contador "N resultados"; refresco conserva las tarjetas visibles; el primer ítem recibe el
foco solo si el usuario pidió "actualizar".

### `TarjetaRegistro.svelte` (nuevo)

`titulo`, `subtitulo?`, `mono?`, `badge?: { texto, variante }`, `campos: { etiqueta, valor }[]`,
`acciones?: Snippet`, `onabrir?`. Altura mínima de 72 px; títulos a dos líneas con elipsis; acción
principal de 44×44; menú desbordable para el resto.

### `TarjetaInventario.svelte` (nuevo)

`item` con `sku`, `nombre`, `stockActual`, `bajoMinimo`, `contexto`, más `acciones` y `onabrir`.
Usa `clasificarStock` y muestra SKU monoespaciado, nombre a dos líneas, badge con texto y acciones.
El badge y el texto son obligatorios; el color es refuerzo.

### `CarruselMetricas.svelte` (nuevo)

`metricas: MetricaTablero[]`; scroll-snap de una tarjeta por vista, botones anterior/siguiente,
puntos indicadores, teclado (flechas) y `aria-roledescription="carrusel"`. En tablet muestra dos por
vista.

### `FiltrosMovil.svelte` (nuevo)

`resumen`, `activos: { etiqueta, valor }[]`, `onlimpiar`, `children`. Panel colapsable con chips de
filtros activos; el botón "Filtros" indica cuántos hay; limpiar no recarga de más.

### `BotonFlotante.svelte` (nuevo)

`etiqueta`, `icono`, `onclick`, `secundario?`. 56×56, `bottom` por encima de la barra + área segura,
`aria-label=etiqueta`, tooltip en escritorio. Nunca cubre la última tarjeta.

### `AvisoFlotante.svelte` (nuevo)

Consume `$ultimoAviso`; `role="status"`, variante éxito/error, cierre manual, duración inyectable,
encima de la barra inferior y debajo de las hojas. El mensaje no usa emojis.

### `Modal.svelte` (modificado)

Suma `variante?: 'dialogo' | 'hoja'`. En móvil, la hoja ocupa `100dvh`, con cabecera fija, cuerpo
desplazable y pie de acciones `sticky`; conserva trap de foco, Escape, `aria-modal`, bloqueo de
scroll y restauración de posición al cerrar.

### `HojaMovimiento.svelte` (nuevo)

`abierta`, `producto?: Producto | StockItem`, `oncerrar`, `onregistrado`.
Campos: tipo (Entrada/Salida/Merma/Ajuste según rol), depósito, cantidad, motivo, observación.
Al abrir pide `listarDepositosActivos` y `obtenerStockProducto`; valida con `validarCantidad`; llama
a `crearMovimiento`; al confirmar, emite el resultado y muestra el aviso.

### `HojaVenta.svelte` (nuevo)

`abierta`, `carrito`, `totales`, `depositoId`, `clientes`, `oncerrar`, `onconfirmar`.
Reproduce el carrito de Caja (utilidades existentes) en pantalla completa; nunca confirma con
carrito vacío ni con descuento inválido.

## 3. Navegación

```ts
destinosBarraInferior(rutaActual, { familiaRol, alertasStock }) => [
  Inicio, Caja, Movimientos, Stock | Transferencias, Más
]
```

- `Más` (`ruta: '#panel'`) está activo cuando la ruta vigente no es ninguna de las cuatro.
- El panel lista los 17 destinos agrupados y conserva los contadores.
- Todo destino permitido está a ≤ 2 toques desde cualquier pantalla.

## 4. Estados

| Estado | Lista de tarjetas | Widget del tablero | Hoja |
|---|---|---|---|
| Carga | 3 esqueletos del alto de tarjeta | Esqueleto con la forma de la métrica | Pie deshabilitado |
| Vacío | `EstadoVacio` con acción siguiente | Mensaje positivo y acción | No aplica |
| Error | Aviso + "Reintentar" | Error propio + "Reintentar" | Mensaje dentro de la hoja |
| Refresco | Conserva tarjetas, sin salto | Valores previos atenuados | Botón "Guardando…" |
| Éxito | Se actualiza la lista | Se actualizan valores | Aviso + cierre |

## 5. Accesibilidad

- Área mínima de toque 44×44 px para todo control, incluidos los de fila y los cierres.
- Campos con etiqueta asociada; errores con `aria-describedby` y foco en el campo inválido.
- Panel y hoja: `role="dialog"`, `aria-modal`, foco atrapado, foco devuelto al disparador, Escape.
- Navegación: `nav` con nombre accesible, `aria-current="page"` en la ruta activa.
- Avisos: `role="status"` + texto completo; el color no es la única señal.
- Imágenes icónicas con nombre accesible; iconos decorativos `aria-hidden`.
- Texto escalable al 200 % sin pérdida de contenido; campos a 16 px mínimo.
- `prefers-reduced-motion` y `forced-colors` según el contrato de tokens.
- Orden de foco coherente con el orden visual; sin trampas de foco en listas ni carrusel.

## 6. Búsqueda, tarjetas y avisos

- Búsqueda: recorta el texto; con menos de 2 caracteres muestra la sugerencia inicial; debounce de
  250 ms; hasta 20 resultados; descarta respuestas fuera de orden.
- Tarjeta: título a 2 líneas con `text-overflow: ellipsis` y `title`; SKU a 1 línea; en pantallas de
  menos de 360 px el título baja a 13 px y el subtítulo se oculta antes de desbordar.
- Aviso de movimiento: `Entrada registrada: {producto} · {depósito} · {cantidad} → stock {stock}`
  (o el tipo correspondiente). Error: el mensaje concreto de la API, sin texto genérico.
- Acciones destructivas (baja, anulación): confirmación con el nombre del registro y el resultado
  concreto al terminar.

## 7. Iconografía

Se conservan los iconos de 018. Se agregan: `ventas` (reutiliza `caja` si el trazo alcanza),
`clientes` (reutiliza `usuarios`), `reportes` (reutiliza `stock`), `importar` (reutiliza `subir`) y
`puntos` (menú desbordable). Solo se dibujan trazos nuevos si el icono reutilizado se confunde en la
barra inferior; no se incorporan librerías.

## 8. Reglas de no regresión

- Ningún cambio de API, contrato de datos, validación de backend ni permisos.
- El escritorio a 1024 px o más conserva estructura, tokens y comportamiento de 018.
- Las rutas de acceso mantienen campos, mensajes y redirecciones de 001 y 018; solo cambia la piel.
- `npm run test`, `npm run check` y `npm run build` en verde antes de cerrar cada incremento.
