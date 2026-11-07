# Contrato: tokens y contraste del móvil cyberpunk

**Feature**: 019-rediseno-visual-mobile | **Date**: 2026-11-07 | **Estado**: contractual

Este contrato amplía `specs/018-rediseno-visual-desktop/contracts/tokens.md` con la capa móvil. La
fuente de verdad de los valores es `frontend/src/lib/diseno/tokens.css` y
`frontend/src/lib/diseno/movil.css`; este documento fija el significado, los pares aprobados y los
mínimos que el test automático verifica.

## 1. Activación

```css
/* Solo en el layout móvil; el escritorio conserva la capa de 018. */
@media (max-width: 1023px) {
	html[data-tema='dark'] {
		/* tokens cyberpunk */
	}
}
```

- `html[data-tema='dark']` lo aplica `app.html` antes del primer pintado cuando el ancho es móvil.
- El selector de tema no se renderiza en el marco móvil (FR-104).
- La paleta cyberpunk no se aplica en escritorio ni en la pantalla de acceso de escritorio.

## 2. Paleta

| Token | Valor | Uso permitido |
|---|---|---|
| `--fondo` | `#05070a` | Fondo de la aplicación |
| `--fondo-elevado` | `#080b0f` | Cabecera, barra inferior |
| `--superficie` | `#0d1117` | Tarjetas, hojas, campos |
| `--superficie-2` | `#11161d` | Encabezados de lista, pies |
| `--superficie-3` | `#171f28` | Esqueletos, separadores de datos |
| `--sidebar` | `#080b0f` | Panel lateral |
| `--borde` | `#1e2936` | Bordes decorativos (sin mínimo) |
| `--borde-fuerte` | `#2c3b4d` | Separadores con relieve |
| `--borde-control` | `#526581` | Borde de campos y controles (≥ 3:1) |
| `--texto` | `#eaf2ff` | Texto principal |
| `--texto-suave` | `#a8b8cc` | Texto secundario |
| `--texto-tenue` | `#94a3b8` | Metadatos (nunca sobre `--superficie-3`) |
| `--texto-inverso` | `#041014` | Texto sobre rellenos neón |
| `--primario` | `#2de2e6` | Cian: activo, navegación, acción principal |
| `--primario-hover` | `#6ff0f2` | Hover/foco del cian |
| `--primario-suave` | `#0a2b2f` | Fondo de selección |
| `--primario-texto` | `#6ff0f2` | Texto/ícono cian sobre oscuro |
| `--primario-contraste` | `#041014` | Texto sobre relleno cian |
| `--acento-secundario` | `#c77dff` | Violeta: líneas, rellenos, glow (no texto) |
| `--violeta-texto` | `#d9a7ff` | Texto de métricas secundarias |
| `--exito` | `#3ddc84` | Relleno/borde de estado normal |
| `--exito-suave` | `#0d2a1d` | Fondo de badge normal |
| `--exito-texto` | `#5cf2a0` | Texto de badge normal |
| `--ambar` | `#ff9e2c` | Relleno/borde de estado bajo |
| `--ambar-suave` | `#2e2110` | Fondo de badge bajo |
| `--ambar-texto` | `#ffc46b` | Texto de badge bajo |
| `--peligro` | `#ff4d5e` | Relleno/borde/glow de alerta (no texto) |
| `--peligro-suave` | `#331318` | Fondo de badge crítico |
| `--peligro-texto` | `#ff8f9b` | Texto de badge crítico y errores |
| `--foco` | `#6ff0f2` | Anillo de foco (≥ 3:1) |
| `--brillo-cian` | `0 0 0 1px #2de2e6, 0 0 18px -6px #2de2e6` | Glow de interactivos |
| `--brillo-alerta` | `0 0 0 1px #ff4d5e, 0 0 18px -6px #ff4d5e` | Glow de alerta |

## 3. Matriz de contraste aprobada

Mínimos: **7:1** texto de cuerpo/datos/badges, **4,5:1** texto grande (≥ 18,66 px o 24 px), **3:1**
bordes de control, foco e indicadores no textuales.

| Primer plano | Fondo | Uso | Mín. | Ratio | Estado |
|---|---|---|---|---|---|
| `--texto` | `--fondo` | Cuerpo | 7 | 17,91 | OK |
| `--texto` | `--superficie` | Cuerpo en tarjeta | 7 | 16,80 | OK |
| `--texto` | `--superficie-2` | Cuerpo en cabecera de lista | 7 | 16,12 | OK |
| `--texto-suave` | `--superficie` | Secundario | 7 | 9,37 | OK |
| `--texto-suave` | `--superficie-2` | Secundario | 7 | 8,99 | OK |
| `--texto-suave` | `--superficie-3` | Secundario sobre esqueleto | 7 | 8,23 | OK |
| `--texto-tenue` | `--fondo` | Metadatos | 7 | 7,87 | OK |
| `--texto-tenue` | `--superficie` | Metadatos | 7 | 7,38 | OK |
| `--texto-tenue` | `--superficie-2` | Metadatos | 7 | 7,08 | OK |
| `--texto-tenue` | `--superficie-3` | — | 7 | 6,48 | **PROHIBIDO** |
| `--primario-texto` | `--superficie` | Activo/navegación | 7 | 13,90 | OK |
| `--primario-texto` | `--fondo` | Activo en cabecera | 7 | 14,81 | OK |
| `--primario-texto` | `--primario-suave` | Ítem seleccionado | 7 | 11,02 | OK |
| `--texto-suave` | `--primario-suave` | Texto secundario sobre selección | 7 | 7,43 | OK |
| `--primario` | `--superficie` | Ícono/indicador | 3 | 11,83 | OK |
| `--primario-contraste` | `--primario` | Texto sobre botón | 7 | 12,06 | OK |
| `--violeta-texto` | `--superficie` | Métrica secundaria | 7 | 9,84 | OK |
| `--violeta-texto` | `--superficie-2` | Métrica secundaria | 7 | 9,45 | OK |
| `--acento-secundario` | `--superficie` | Línea/relleno | 3 | 7,03 | OK |
| `--acento-secundario` | `--superficie-2` | Línea/relleno (no texto) | 3 | 6,75 | OK |
| `--exito-texto` | `--superficie` | Texto normal | 7 | 13,21 | OK |
| `--exito-texto` | `--exito-suave` | Badge normal | 7 | 10,73 | OK |
| `--ambar-texto` | `--superficie` | Texto bajo | 7 | 12,04 | OK |
| `--ambar-texto` | `--ambar-suave` | Badge bajo | 7 | 9,96 | OK |
| `--peligro-texto` | `--superficie` | Texto crítico | 7 | 8,70 | OK |
| `--peligro-texto` | `--peligro-suave` | Badge crítico | 7 | 7,74 | OK |
| `--peligro` | `--superficie` | Relleno/borde/glow (no texto) | 3 | 5,84 | OK |
| `--borde-control` | `--superficie` | Borde de campo | 3 | 3,19 | OK |
| `--foco` | `--superficie` | Anillo de foco | 3 | 13,90 | OK |

Reglas duras:
- `--peligro` y `--acento-secundario` no se usan como `color` de texto en ningún tamaño.
- `--texto-tenue` no se usa sobre `--superficie-3`.
- Un relleno sólido solo lleva texto con `--primario-contraste` y únicamente en `--primario`,
  `--exito` o `--ambar` (12,06, 10,80 y 9,34 respectivamente); sobre `--peligro` el texto oscuro da
  5,94 y no se usa. Los botones destructivos usan `--peligro-suave` de fondo, `--peligro-texto` de
  texto y borde/glow de `--peligro`, nunca relleno rojo con texto claro.
- Todo badge lleva etiqueta textual; el color nunca es la única señal.

## 4. Modos de accesibilidad

### `prefers-contrast: more`

Sube el contraste sin abandonar la identidad oscura:

| Token | Valor | Ratio sobre `--superficie` |
|---|---|---|
| `--texto` | `#ffffff` | 18,92 |
| `--texto-suave` | `#d7e3f2` | 14,56 |
| `--texto-tenue` | `#c2d2e6` | 12,31 |
| `--borde` | `#8aa0b8` | 7,03 (borde visible) |
| `--borde-control` | `#a8b8cc` | 9,37 |
| `--foco` | `#a8b8cc` | 9,37 |
| `--primario-texto` | `#8ff6f8` | ≥ 7 |
| `--violeta-texto` | `#e6c8ff` | 12,68 |
| `--ambar-texto` | `#ffd79a` | 13,91 |
| `--peligro-texto` | `#ffb3ba` | 11,18 |
| `--exito-texto` | `#9bf7c1` | 14,84 |

Además: glow reducido a un borde de 1 px, sin sombras difusas, y `--texto-tenue` permitido sobre
`--superficie-3` (el valor `#c2d2e6` da 10,81).

### `forced-colors: active`

- `box-shadow`, `text-shadow` y `filter` se anulan; el glow no se dibuja.
- Bordes de tarjetas, badges y campos usan `CanvasText`/`ButtonBorder`; los rellenos usan
  `Canvas`/`ButtonFace`.
- Los badges conservan su etiqueta textual y agregan borde de 1 px para no depender del relleno.
- `forced-color-adjust: auto` en todos los elementos; no se fuerza la paleta de marca.

### `prefers-reduced-motion: reduce`

- Sin transición de panel/hoja ni pulsación del destello; los cambios de estado son instantáneos.
- El glow estático se conserva (no es movimiento).

## 5. Geometría, área segura y efectos

| Token | Valor | Uso |
|---|---|---|
| `--toque-min` | `44px` | Área mínima de todo control |
| `--cabecera-alto-movil` | `56px` | Cabecera compacta |
| `--barra-inferior-alto` | `64px` | Barra de navegación |
| `--fab-tamano` | `56px` | Acción flotante |
| `--radio-tarjeta-movil` | `12px` | Tarjetas y hojas |
| `--espacio-movil` | `12px` | Padding lateral de pantalla |
| `--contenido-max-movil` | `720px` | Ancho máximo en tablet |
| `--zona-segura-inferior` | `env(safe-area-inset-bottom, 0px)` | Barra inferior y hojas |
| `--zona-segura-superior` | `env(safe-area-inset-top, 0px)` | Cabecera |
| `--scroll-padding-inferior` | `calc(var(--barra-inferior-alto) + var(--zona-segura-inferior) + 16px)` | Anclas y foco |

Reglas:
- La barra inferior fija usa `padding-bottom: var(--zona-segura-inferior)`.
- El contenido reserva el alto de la barra más el área segura para que la última tarjeta no quede
  tapada.
- El FAB se posiciona por encima de la barra y no tapa acciones.
- Los campos usan `font-size: 16px` como mínimo para evitar el zoom automático en iOS.
- El alto de pantalla usa `dvh`, nunca `100vh`.

## 6. Guard automático

`frontend/src/lib/utils/contraste.test.ts`:

1. Lee `frontend/src/lib/diseno/tokens.css` y `movil.css` con `fs` y extrae los valores de los
   bloques `:root`, `html[data-tema='dark']`, `@media (max-width: 1023px)` y
   `prefers-contrast: more`.
2. Calcula luminancia relativa y ratio WCAG con `utils/contraste.ts` (funciones puras).
3. Verifica cada fila de la matriz §3 y §4 con estado **OK** contra su mínimo; las filas marcadas
   como prohibidas o "solo no texto" son restricciones de uso, no pares verificados.
4. Falla con un mensaje que nombra el token y el ratio obtenido si un valor cambia por debajo del
   mínimo.
5. Busca en `movil.css` las declaraciones `color:` y falla si usan `--peligro` o
   `--acento-secundario` como color de texto.

El uso de `--texto-tenue` sobre `--superficie-3` (§3, fila prohibida) se revisa en la aceptación
visual y en la revisión de código; el test cubre los valores y la regla de texto con tonos crudos.

Criterio de aceptación: el test forma parte de `npm run test` y una modificación de la paleta que
rompa un par aprobado falla la suite (SC-042, SC-049).
