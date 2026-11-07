# Implementation Plan: Rediseño visual móvil en modo oscuro cyberpunk

**Branch**: `019-rediseno-visual-mobile` | **Date**: 2026-11-07 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `specs/019-rediseno-visual-mobile/spec.md`

**Referencia visual**: no hay mock móvil; la referencia es la paleta, los efectos y la ergonomía de
la spec, aplicados sobre la identidad ya implementada en 018. El escritorio (1024 px o más) no
cambia.

## Summary

Adaptar las 21 pantallas de Stockmante al teléfono y la tablet con una identidad cyberpunk oscura
obligatoria, navegación híbrida (barra inferior de 4 destinos + panel "Más"), tablas convertidas en
tarjetas de inventario, tablero con métricas deslizables y un registro rápido de movimientos en
hoja a pantalla completa con aviso neón del resultado.

El trabajo es **de presentación y estado local de interfaz**: no cambia endpoints, contratos,
esquema, permisos ni reglas de negocio. Se apoya en la capa de tokens y en los componentes de 018.

**Hallazgo cerrado — accesibilidad del oscuro obligatorio**: al ocultar el selector de tema, el
usuario pierde la vía de escape a un tema claro. Se resuelve sin reintroducir el tema claro:
paleta verificada contra **WCAG AAA (7:1)** con texto de estado en variantes claras (FR-103),
respeto por `prefers-contrast: more` y `forced-colors: active` (FR-133), y un **test automático de
contraste** que lee los tokens y falla si un par aprobado baja de su mínimo o si aparece un texto
con un tono crudo (SC-042, SC-049). Comunicar estado nunca depende solo del color: todo badge
lleva etiqueta textual y todo aviso lleva texto e icono.

## Technical Context

**Language/Version**: TypeScript 6 + Svelte 5 (runes) sobre SvelteKit 3 en modo SPA, Vite 8 y
Tailwind CSS 4.

**Primary Dependencies**: las ya instaladas en `frontend/package.json` (SvelteKit, Tailwind, zod,
Vitest 4, svelte-check) y Playwright 1.64 en la raíz para evidencia. NO se agregan librerías de UI,
iconos, gráficos, gestos ni contraste.

**Storage**: `localStorage` solo para la preferencia de tema existente (`stockmante:tema`), que en
móvil se conserva pero no se aplica; estado de panel, hoja, filtros y avisos en memoria. Sin base de
datos ni endpoints nuevos.

**Testing**: Vitest en entorno `node` para lógica pura nueva (clasificación de dispositivo,
contraste de tokens, umbrales de badge, destinos de la barra inferior, estado del carrusel y filtros,
temporizador de avisos, composición de la búsqueda de inventario y validación de cantidades);
`svelte-check` para tipos y plantillas; Playwright con emulación de dispositivos para la evidencia.

**Target Platform**: navegadores móviles Android/iOS. Referencia 390×844; mínimo 320×568;
horizontal 844×390; tablet 768–1023. Escritorio desde 1024 px sin cambios.

**Project Type**: aplicación web SPA (frontend SvelteKit + backend Rust/Axum ya existente).

**Performance Goals**: la lista filtrada se pinta en menos de 300 ms p95 con 10 000 productos;
desplazamiento fluido (objetivo 60 fps) con hasta 100 tarjetas por página; sin saltos de layout al
cargar (esqueletos del mismo alto) y sin bloqueos de interacción.

**Constraints**: sin cambios de API, esquema ni permisos; área de toque mínima 44×44 px; texto de
cuerpo AAA (7:1); sin dependencias nuevas; sin lectura por cámara; sin PWA ni offline; el layout
móvil es siempre oscuro.

**Scale/Scope**: 21 pantallas + modo de cambio obligatorio de contraseña; 17 destinos de navegación
(4 en la barra inferior + "Más"); listas de hasta 100 filas por página; catálogo de referencia de
10 000 productos.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principio | Evaluación | Evidencia |
|---|---|---|
| I. Simplicidad / anti-sobreingeniería | CUMPLE | Se reutilizan tokens, `BarraLateral`, `Modal`, `TarjetaKpi`, `Paginacion` y `EtiquetaEstado`; no se agregan dependencias ni una DSL de tarjetas: un shell genérico y tarjetas por entidad. Los destinos de la barra salen de la lista de navegación existente. |
| II. Mantenibilidad | CUMPLE | Componentes móviles de una responsabilidad; toda llamada HTTP sigue pasando por `$lib/api/*` (se agrega `inventario.ts`, que compone clientes existentes); el backend no se toca; nombres de negocio en español. |
| III. Claridad en mensajes | CUMPLE | El aviso de confirmación informa tipo, producto, depósito, cantidad y stock resultante; los errores de validación y de API usan el mensaje concreto existente, sin genéricos. |
| IV. Testabilidad primero | CUMPLE | Tests de lógica pura antes de la implementación (Red → Green) para dispositivo, contraste, badges, navegación, avisos, búsqueda y validación; las plantillas se validan con `svelte-check` y la aceptación visual con capturas. |
| V. Integridad de datos | CUMPLE (N/A) | No hay SQL, migraciones ni transacciones nuevas; el registro rápido llama a `POST /api/movimientos`, que conserva la transacción atómica y las validaciones del backend. |

**Resultado**: sin violaciones; no se requiere Complexity Tracking.

## Project Structure

### Documentation (this feature)

```text
specs/019-rediseno-visual-mobile/
├── plan.md              # Este archivo
├── research.md          # Fase 0: decisiones técnicas
├── data-model.md        # Fase 1: pantallas, tarjetas, navegación, contraste
├── quickstart.md        # Fase 1: validación end-to-end
├── contracts/
│   ├── tokens-movil.md  # Paleta cyberpunk, matriz AAA, modos de accesibilidad
│   └── ui-movil.md      # Componentes, navegación, estados, mapa por pantalla
└── tasks.md             # Fase 2 (/speckit-tasks, no se crea acá)
```

### Source Code (repository root)

```text
frontend/
├── src/
│   ├── app.html                                  # MOD: tema móvil antes del primer pintado
│   ├── lib/
│   │   ├── api/
│   │   │   ├── inventario.ts                     # NUEVO: búsqueda de inventario compuesta
│   │   │   └── inventario.test.ts                # NUEVO
│   │   ├── dispositivo/
│   │   │   ├── dispositivo.ts                    # NUEVO: clasificación y store de layout
│   │   │   └── dispositivo.test.ts               # NUEVO
│   │   ├── diseno/
│   │   │   ├── tokens.css                        # MOD: tokens geométricos y modos de accesibilidad
│   │   │   └── movil.css                         # NUEVO: capa cyberpunk del layout móvil
│   │   ├── navegacion/
│   │   │   ├── navegacion.ts                     # MOD: destinos de barra inferior
│   │   │   └── navegacion.test.ts                # MOD
│   │   ├── stores/
│   │   │   ├── avisos.ts                         # NUEVO: cola de avisos con temporizador
│   │   │   └── avisos.test.ts                    # NUEVO
│   │   ├── tema/
│   │   │   ├── tema.ts                           # MOD: oscuro forzado en layout móvil
│   │   │   └── tema.test.ts                      # MOD
│   │   ├── utils/
│   │   │   ├── inventario.ts                     # NUEVO: badge y validación de cantidades
│   │   │   ├── inventario.test.ts                # NUEVO
│   │   │   ├── contraste.ts                      # NUEVO: luminancia y ratio WCAG
│   │   │   ├── contraste.test.ts                 # NUEVO: guard de la matriz de tokens
│   │   │   ├── filtros.ts                        # NUEVO: conteo y resumen de filtros activos
│   │   │   └── filtros.test.ts                   # NUEVO
│   │   ├── components/
│   │   │   ├── movil/
│   │   │   │   ├── MarcoMovil.svelte             # NUEVO: cabecera + contenido + barra inferior
│   │   │   │   ├── CabeceraMovil.svelte          # NUEVO: título, buscador, usuario
│   │   │   │   ├── BarraInferior.svelte          # NUEVO: 4 destinos + Más
│   │   │   │   ├── BuscadorMovil.svelte          # NUEVO: búsqueda con resultados en tarjetas
│   │   │   │   ├── ListaTarjetas.svelte          # NUEVO: shell de estados + paginación
│   │   │   │   ├── TarjetaRegistro.svelte        # NUEVO: tarjeta genérica de datos
│   │   │   │   ├── TarjetaInventario.svelte      # NUEVO: tarjeta con SKU y badge de stock
│   │   │   │   ├── CarruselMetricas.svelte       # NUEVO: métricas deslizables del tablero
│   │   │   │   ├── FiltrosMovil.svelte           # NUEVO: panel colapsable de filtros
│   │   │   │   └── BotonFlotante.svelte          # NUEVO: acción rápida de 56×56
│   │   │   ├── ui/
│   │   │   │   ├── Modal.svelte                  # MOD: variante hoja a pantalla completa
│   │   │   │   ├── AvisoFlotante.svelte          # NUEVO: aviso neón con resultado
│   │   │   │   ├── Icono.svelte                  # MOD: icono `puntos` (menú desbordable)
│   │   │   │   └── iconos.ts                     # MOD
│   │   │   ├── marco/
│   │   │   │   ├── MarcoApp.svelte               # MOD: elige marco móvil o escritorio
│   │   │   │   ├── BarraLateral.svelte           # MOD: foco atrapado, área segura, estilo móvil
│   │   │   │   └── BarraSuperior.svelte          # MOD: sin selector de tema en móvil
│   │   │   ├── movimientos/
│   │   │   │   └── HojaMovimiento.svelte         # NUEVO: registro rápido con stock por depósito
│   │   │   └── ventas/
│   │   │       └── HojaVenta.svelte              # NUEVO: carrito móvil de Caja
│   │   └── ...
│   └── routes/
│       ├── +page.svelte                          # MOD: tablero con carrusel
│       ├── +layout.svelte                        # MOD: inicia layout móvil y avisos
│       ├── caja/+page.svelte                     # MOD: tarjetas, buscador y hoja de venta
│       ├── movimientos/+page.svelte              # MOD: tarjetas y hoja de movimiento
│       ├── transferencias/+page.svelte           # MOD: tarjetas
│       ├── conteos/+page.svelte                  # MOD: tarjetas
│       ├── asistente/+page.svelte                # MOD: ajustes de lectura y entrada
│       ├── login|registro|verificar-correo/      # MOD: lienzo y teclado
│       └── admin/*/+page.svelte                  # MOD: tarjetas por pantalla
scripts/
└── capturar-movil.mjs                            # NUEVO: Playwright 390×844, 844×390, 320×568
```

**Structure Decision**: se conserva el proyecto único `frontend/` con SvelteKit y se agrega un
directorio `components/movil/` para la capa móvil, siguiendo el agrupamiento por dominio de 018
(`marco/`, `ui/`, `productos/`, etc.). La lógica pura nueva vive en `lib/dispositivo`,
`lib/utils/inventario.ts`, `lib/utils/contraste.ts` y `lib/stores/avisos.ts`, separada de la vista
para poder probarla sin DOM.

## Phase 0: Outline & Research

Resultado completo en [research.md](./research.md). Decisiones principales:

1. **Oscuro obligatorio con garantía AAA** (hallazgo cerrado): paleta verificada + preferencias del
   sistema + guard automático. Se descarta reintroducir el tema claro o un tercer tema.
2. **Forzado antes del primer pintado** en `app.html` + `matchMedia('(max-width: 1023px)')`, con
   re-evaluación en `tema.ts` al cruzar el punto de corte. Sin destello.
3. **Un solo DOM**: store de dispositivo con clasificador puro; se renderiza tabla o tarjetas, no
   ambas. Se descarta duplicar el marcado y ocultarlo por CSS.
4. **Navegación híbrida** derivada de `ITEMS_NAVEGACION` con una función pura de destinos; el panel
   reutiliza `BarraLateral` con foco atrapado y bloqueo de scroll.
5. **Tarjetas de inventario** con shell `ListaTarjetas` + `TarjetaRegistro`/`TarjetaInventario`;
   cada pantalla aporta su snippet de tarjeta. Se descarta una DSL configurable.
6. **Búsqueda de inventario** compuesta: `listarStock({q})` para nombre/SKU y, si no hay resultados
   y el texto parece código de barras, `buscarProductos` + `listarStock({productoIds})`.
7. **Hoja de acción** como variante de `Modal` (foco, Escape y scroll ya resueltos), con restauración
   de la posición de scroll.
8. **Avisos** con store y temporizador inyectable; `role="status"` y resultado concreto.
9. **Validación de cantidades** previa con stock por depósito (`obtenerStockProducto`) y función pura.
10. **Badge de stock**: Crítico = stock actual ≤ 0; Bajo = bajo mínimo; Normal = resto.

## Phase 1: Design & Contracts

- [data-model.md](./data-model.md): las 21 pantallas y su patrón, destinos de navegación, tarjetas,
  métricas, estados de hoja y aviso, entidad de par de contraste y reglas de validación.
- [contracts/tokens-movil.md](./contracts/tokens-movil.md): paleta cyberpunk con ratios calculados,
  modos `prefers-contrast`/`forced-colors`, geometría, área segura y reglas del guard automático.
- [contracts/ui-movil.md](./contracts/ui-movil.md): contrato de componentes, puntos de corte,
  navegación, estados, accesibilidad y mapa pantalla por pantalla.
- [quickstart.md](./quickstart.md): comandos, recorridos de validación y evidencia.

### Re-evaluación de la Constitución (post-diseño)

Sin cambios en el resultado: el diseño no agrega dependencias, no toca backend ni SQL, mantiene los
mensajes concretos y concentra la lógica nueva en módulos puros con tests previstos. El único punto
de atención es el guard de contraste: el test debe leer los tokens reales (no una copia) para que el
contrato no se desincronice; se resuelve en `utils/contraste.test.ts` leyendo `tokens.css` y
`movil.css` con `fs` y exigiendo que todo par declarado esté registrado.

## Risks & Mitigations

| Riesgo | Mitigación |
|---|---|
| Sin tema claro de escape, un par de color ilegible deja la app inaccesible | Matriz AAA verificada, variantes claras para estado, `prefers-contrast`, `forced-colors` y test de contraste en la suite (FR-103, FR-133, SC-042, SC-049) |
| Destello claro al forzar el oscuro o al cruzar 1023/1024 px | Forzado en el script inline antes del pintado y escucha de `matchMedia` en `tema.ts`; prueba de 10 recargas (SC-048) |
| Tabla y tarjetas duplicadas en el DOM | Store de dispositivo con clasificador puro y render condicional; `svelte-check` y revisión de que solo exista un marcado |
| Barra inferior o FAB tapando la última tarjeta o el contenido | `padding-bottom` con área segura, `scroll-padding` y FAB por encima de la barra; prueba con 50 tarjetas |
| Teclado del teléfono tapando el campo o el botón de confirmar | `visualViewport`/`dvh`, campos de 16 px y botón de confirmar sticky dentro de la hoja |
| Scroll del fondo con la hoja abierta (iOS) | Bloqueo con restauración de posición y `overscroll-behavior`; se reutiliza la lógica de `Modal` |
| Foco perdido al cerrar panel u hoja | Devolución al disparador y foco atrapado; se reutiliza el trap de `Modal` y se agrega a `BarraLateral` |
| `prefers-contrast` no es emulable de forma fiable en Playwright | Se valida con el test de contraste sobre tokens y una comprobación manual en quickstart; `forced-colors` sí se emula |
| Regresión en escritorio | Toda la capa móvil vive bajo `@media (max-width: 1023px)` y los componentes existentes conservan su rama; evidencia de escritorio a 1440×900 |
| Colores literales (`rgba`, hex) en componentes de 018 fuera de los tokens | Auditoría al inicio de la implementación: todo color de la capa móvil pasa por tokens y los overlays usan tokens de sombra/borde; el guard cubre los tokens y la revisión cubre los literales |
| Test de contraste frágil al parsear CSS | Formato de tokens estable por bloque y mensaje de error que nombra el token; el contrato lista los pares aprobados |

## Complexity Tracking

Sin violaciones de la Constitución. No se agregan proyectos, capas, dependencias ni patrones.

## Dependencias y orden de implementación

1. **Fundaciones**: tokens móviles, modos de accesibilidad, dispositivo, tema forzado, avisos y
   contraste (FR-102…FR-106, FR-129, FR-133).
2. **Marco y navegación**: `MarcoMovil`, cabecera, barra inferior, panel, buscador (FR-107…FR-113).
3. **Tablero**: carrusel de métricas y estados por widget (FR-114…FR-116).
4. **Listas**: `ListaTarjetas` y adaptación de las 13 pantallas con tabla (FR-117…FR-121).
5. **Registro rápido**: FAB, hoja de movimiento, hoja de venta, validación y aviso (FR-122…FR-126).
6. **Cierre**: acceso, administración, accesibilidad, evidencia y no regresión (FR-127, FR-128,
   FR-130…FR-132; SC-039…SC-049).

La spec no introduce migraciones ni endpoints; se apoya en los contratos de 001–017 y en la capa
visual de 018.
