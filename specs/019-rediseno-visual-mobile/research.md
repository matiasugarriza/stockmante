# Research: Rediseño visual móvil en modo oscuro cyberpunk

**Feature**: 019-rediseno-visual-mobile | **Date**: 2026-11-07

Este documento resuelve las decisiones técnicas de la spec. No hay `NEEDS CLARIFICATION` abiertos.

## D1. Oscuro obligatorio y accesibilidad (hallazgo cerrado)

**Decision**: El layout móvil usa siempre la identidad oscura cyberpunk, sin selector de tema, y la
accesibilidad se garantiza por cuatro vías: (1) paleta verificada contra **WCAG AAA** para texto de
cuerpo, datos y badges (7:1), 4,5:1 para texto grande y 3:1 para bordes de control y foco; (2) los
tonos crudos de estado (rojo `#ff4d5e`, violeta `#c77dff`) nunca se usan como texto, solo como
relleno, borde o resplandor, y el texto usa sus variantes claras verificadas; (3) soporte de
`prefers-contrast: more` y `forced-colors: active`; (4) un test automático que lee los tokens y falla
si un par aprobado baja de su mínimo o si aparece un texto con un tono crudo.

**Rationale**: el usuario confirmó que el móvil debe ser siempre oscuro y sin selector; la vía de
escape no puede ser un tema claro. Subir el estándar de contraste y respetar las preferencias del
sistema cierra el riesgo sin contradecir la decisión de producto ni sumar un control nuevo que el
usuario pueda no encontrar. El test convierte una promesa de diseño en un guard de regresión.

**Alternatives considered**:
- Mantener AA (4,5:1) y confiar en la revisión visual: insuficiente para un modo sin escape;
  descartado por el hallazgo.
- Reintroducir un tema claro en móvil: contradice la decisión confirmada y duplica la paleta.
- Agregar un selector de contraste propio en la cabecera: más superficie y estado que mantener; las
  preferencias del sistema son el mecanismo estándar (y `forced-colors` ya existe para ello).

## D2. Forzado del tema antes del primer pintado

**Decision**: El script inline de `frontend/src/app.html` evalúa
`window.matchMedia('(max-width: 1023px)').matches`; si es móvil, aplica `data-tema="dark"` y
`color-scheme: dark` sin leer la preferencia guardada. `tema.ts` incorpora `resolverTema(...,
esLayoutMovil)` y escucha el cambio de layout con `matchMedia` para re-aplicar al cruzar el punto de
corte; al volver a escritorio se relee `stockmante:tema`.

**Rationale**: el script corre antes del pintado, así que no hay destello claro ni salto; la misma
regla en `tema.ts` evita que un resize o la rotación deje el tema desincronizado. No requiere SSR
(la app es SPA con fallback estático).

**Alternatives considered**:
- Resolver el oscuro solo con `@media (max-width: 1023px)` en CSS: el atributo `data-tema` seguiría
  en claro y `color-scheme`/controles nativos quedarían incoherentes; además la lógica de tema
  seguiría creyendo que está en claro.
- Forzar por user agent: frágil; el ancho es la señal que ya usa el marco de 018.

## D3. Un solo DOM: store de dispositivo

**Decision**: `lib/dispositivo/dispositivo.ts` expone un clasificador puro
(`clasificarDispositivo(ancho)` → `telefono` | `tablet` | `escritorio`; `esLayoutMovil(ancho)` →
`ancho < 1024`) y un store con `{ ancho, layout, orientacion }` inicializado desde `matchMedia` en
el layout raíz y actualizado en `resize`/`orientationchange`. Las pantallas deciden con `$layout`
qué renderizar.

**Rationale**: renderizar tabla y tarjetas y ocultar una por CSS duplica el DOM (rendimiento y
accesibilidad: controles ocultos siguen siendo enfocables si no se usa `display:none` con cuidado).
Un store probado mantiene un solo marcado y permite tests deterministas de los puntos de corte.

**Alternatives considered**:
- Doble renderizado + `display:none`: descartado por peso y por riesgo de focos ocultos.
- Container queries: no resuelven la elección de componente sin duplicar el marcado; el ancho de
  ventana es suficiente para esta aplicación.

## D4. Navegación híbrida

**Decision**: `navegacion.ts` suma `destinosBarraInferior(rutaActual, contexto)`, que devuelve 4
destinos + "Más": Inicio, Caja y Movimientos fijos, el cuarto según rol (Stock para `GESTION`,
Transferencias para `OPERACION`) y "Más" activo cuando la ruta vigente no está en la barra. El panel
"Menú" reutiliza `BarraLateral` en su modo móvil, con foco atrapado, Escape, bloqueo de scroll y
cierre por navegación.

**Rationale**: la lista de navegación y la visibilidad por rol ya existen y están testeadas; derivar
la barra evita duplicar RBAC. Reutilizar `BarraLateral` conserva los 17 destinos agrupados y el
contador de alertas.

**Alternatives considered**:
- Un panel nuevo independiente: duplicaría grupos, iconos y RBAC.
- Barra inferior con todos los destinos desplazable: incómoda y sin etiquetas legibles.

## D5. Tablas a tarjetas

**Decision**: `ListaTarjetas.svelte` (shell de estados, contador y paginación) +
`TarjetaRegistro.svelte` (título, subtítulo, campos etiqueta/valor, badge y snippet de acciones) +
`TarjetaInventario.svelte` (SKU, nombre truncado, badge de stock y acciones rápidas). Cada pantalla
conserva su tabla de escritorio y agrega una rama móvil con su snippet de tarjeta. Para las tablas
analíticas escritas a mano (Kardex, Rotación, Valorizado, Ventas e Importación) se usa una variante
CSS de "fila como tarjeta": la tabla se muestra en bloque y cada celda antepone su etiqueta desde
`data-etiqueta`.

**Rationale**: las tablas de inventario necesitan una jerarquía distinta (nombre, SKU y stock
primero; acciones táctiles), mientras que los reportes se leen como filas de datos y no requieren
componentes nuevos. El shell y la tarjeta genérica evitan diez componentes casi iguales sin caer en
una DSL configurable.

**Alternatives considered**:
- Una DSL de columnas → tarjetas: más abstracción de la necesaria (Principio I).
- Solo CSS para todo: no permite priorizar campos ni acciones táctiles de 44 px en las tarjetas de
  inventario.

## D6. Búsqueda de inventario con código de barras

**Decision**: `lib/api/inventario.ts` expone `buscarInventario(q)`:
1. `listarStock({ q, porPagina: 20 })` (subcadena sobre nombre o SKU, con stock y badge).
2. Si no hay resultados y `q` parece un código (`^[0-9A-Za-z]{8,64}$`), `buscarProductos({ q,
   campo: 'codigo' })` y luego `listarStock({ productoIds })` para completar el stock.

**Rationale**: `/api/stock` no busca por código de barras y `/api/productos` no devuelve stock; la
composición reutiliza endpoints existentes y evita N+1. La segunda llamada solo ocurre en el caso
de código, así que el caso común sigue siendo una sola petición.

**Alternatives considered**:
- Agregar código de barras al filtro de `/api/stock`: cambio de backend fuera de alcance (FR-128).
- Buscar producto y pedir stock por cada resultado: N+1 y peor latencia.

## D7. Hoja de acción reutilizando Modal

**Decision**: `Modal.svelte` suma `variante?: 'dialogo' | 'hoja'`; en móvil la variante hoja ocupa
la pantalla, alinea el contenido arriba y el pie de acciones abajo con `sticky`. Se conserva el trap
de foco, Escape, `aria-modal` y el bloqueo de scroll; se agrega la restauración de la posición de
scroll y `overscroll-behavior: contain`.

**Rationale**: el trap y el manejo de foco ya están resueltos y probados en 018; duplicarlos en un
componente nuevo multiplicaría los bugs de accesibilidad.

**Alternatives considered**:
- Un `HojaAccion` independiente: duplicaría foco, Escape y bloqueo de scroll.

## D8. Avisos con resultado concreto

**Decision**: `stores/avisos.ts` mantiene una cola de avisos con temporizador inyectable
(`mostrarAviso(mensaje, { duracionMs, temporizador })`), y `AvisoFlotante.svelte` la renderiza con
`role="status"`, cierre manual y un destello neón de una sola pulsación. El mensaje se arma con el
`MovimientoDetalle` devuelto: tipo, producto, depósito, cantidad y `stock_resultante`.

**Rationale**: el resultado concreto es exigencia del Principio III; el temporizador inyectable
permite testear la cola sin relojes reales (Principio IV).

**Alternatives considered**:
- Aviso local por página: repetiría markup y temporizadores en cada pantalla.
- Mensaje genérico "Operación exitosa": viola el Principio III.

## D9. Validación de cantidades antes del envío

**Decision**: al abrir la hoja se obtiene el desglose con `obtenerStockProducto(productoId)`; la
función pura `validarCantidad({ tipo, cantidad, disponible })` devuelve el mensaje o `null`. Entradas
> 0; salidas y mermas usan el valor absoluto y no pueden superar el disponible del depósito; ajuste
requiere rol `GESTION` y valor distinto de cero. El backend sigue siendo la autoridad final.

**Rationale**: FR-124 pide bloquear el envío por encima del stock antes de llegar al servidor;
conocer el disponible por depósito es lo que permite un mensaje concreto ("3 disponibles, 5
solicitados").

**Alternatives considered**:
- Confiar solo en el 409 del backend: el usuario descubre el problema después de enviar y el
  mensaje llega más tarde.

## D10. Umbrales del badge de stock

**Decision**: `clasificarStock(stockActual, bajoMinimo)` → `critico` si `stock_actual <= 0`, `bajo`
si `bajo_minimo` es verdadero, `normal` en el resto. El badge siempre muestra etiqueta textual
("Crítico", "Bajo", "Normal") además del color.

**Rationale**: el esquema solo expone `stock_actual`, `stock_minimo` y `bajo_minimo`; el cero es el
único caso "crítico" distinguible sin inventar datos. La etiqueta evita depender del color.

**Alternatives considered**:
- Tres niveles derivados solo de `stock_minimo` (por ejemplo, 50 %): inventa una regla de negocio
  que no existe en el backend.

## D11. Evidencia y verificación

**Decision**: `scripts/capturar-movil.mjs` (Playwright, ya instalado en la raíz) inicia sesión con
`CAPTURE_EMAIL`/`CAPTURE_PASSWORD`, emula 390×844, 844×390 y 320×568, recorre las 21 pantallas y el
modo de cambio de contraseña, y guarda capturas en `specs/019-rediseno-visual-mobile/evidencia/`.
El contraste se verifica con el test de tokens; `forced-colors` se emula con Playwright y
`prefers-contrast` queda como comprobación manual en quickstart.

**Rationale**: reutiliza la herramienta y las credenciales de 018; no agrega dependencias.

**Alternatives considered**:
- Capturas manuales: no reproducibles ni comparables entre iteraciones.
