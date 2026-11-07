# Research: Rediseño visual del escritorio

**Feature**: `018-rediseno-visual-desktop` | **Fecha**: 2026-11-07

## Pregunta de investigación

¿Cómo llevar las 17 pantallas capturadas en `captures/` al lenguaje visual de
`captures/samples/light-2.png` y `dark-2.png` sin cambiar comportamiento, sin sumar dependencias y
sin romper la suite existente?

## Método

1. Comparar cada captura con las muestras para identificar estructura, jerarquía, cromática,
   tipografía y patrones repetidos.
2. Inventariar rutas, componentes y endpoints reales del frontend para mapear cada pantalla.
3. Contrastar las decisiones con la constitución (YAGNI, mantenibilidad, testabilidad, mensajes).
4. Verificar el estado del arte local: Svelte 5 con runes, Tailwind 4, Vitest en `node`, Playwright
   instalado con Chromium.

## Lectura de la referencia (light-2 / dark-2)

### Estructura

- **Barra lateral fija** a la izquierda, con marca arriba, ítems agrupados por secciones y estado
  activo con fondo tenue y texto de acento.
- **Barra superior** con título/saludo, acciones contextuales (selector, fecha, búsqueda, iconos) y
  avatar de usuario.
- **Área de contenido** fluida, con una fila de indicadores, luego una grilla de tarjetas de distinto
  ancho (2/3 + 1/3) y finalmente tablas/listas.
- **Tarjetas** blancas (claro) o azul noche (oscuro), borde muy sutil, radio moderado (≈10 px) y
  sombra mínima.
- **Jerarquía** título de tarjeta (14–16 px, seminegrita), subtítulo gris, métrica grande (28–32 px)
  y etiquetas pequeñas en mayúsculas para encabezados de tabla.

### Cromática observada

| Rol | Claro | Oscuro |
|---|---|---|
| Fondo de la app | gris azulado muy claro | azul noche casi negro |
| Superficie | blanco | azul noche elevado |
| Barra lateral | blanco | azul noche más oscuro |
| Texto principal | azul muy oscuro | blanco azulado |
| Texto secundario | gris pizarra | gris azulado |
| Acento primario | azul | azul más luminoso |
| Acento secundario | teal/cian | teal/cian |
| Acento terciario | ámbar/naranja | ámbar/naranja |
| Éxito / alerta / error | verde, ámbar, rojo | verde, ámbar, rojo luminosos |

Los valores exactos se fijan en `contracts/tokens.md`. La muestra no usa crema ni negro cálido; por
eso la paleta crema/naranja de la iteración anterior de 018 queda retirada.

### Tipografía

- Sans del sistema, compacta, sin serif ni display.
- Títulos 20–24 px, peso 600–700, tracking ligeramente negativo.
- Cuerpo 13–14 px, line-height 1.45.
- Etiquetas de tabla y metadatos 11–12 px, mayúsculas y tracking amplio.
- Números grandes con tabular nums para que las métricas no salten.

### Componentes recurrentes

Botón primario/secundario/ghost, input y select con label, checkbox, badge de estado (ámbar, rojo,
teal, verde), tabla con encabezado tenue y filas separadas por línea, avatar circular, barra de
progreso, tarjeta KPI, gráfico de barras, dona, lista de actividad, paginación y menú de usuario.

## Decisiones

### D1 — Adoptar la identidad de las muestras y retirar crema/naranja

- **Decisión**: reemplazar los tokens crema/naranja por la paleta azul/teal/ámbar de `light-2` y
  `dark-2`, con superficies blancas o azul noche.
- **Razón**: es la referencia explícita del pedido; la implementación crema/naranja quedó a medio
  camino y no representa el producto objetivo.
- **Alternativas**: conservar crema/naranja (no cumple el pedido); híbrido (dos identidades
  conviviendo, peor consistencia).

### D2 — Marco en el layout raíz, no en cada página

- **Decisión**: `frontend/src/routes/+layout.svelte` decide entre `MarcoApp` (rutas privadas) y
  `LienzoAcceso` (login, registro, verificación). El `admin/+layout.svelte` conserva su guard.
- **Razón**: una sola fuente de verdad para barra lateral, barra superior y tema; evita duplicar
  marco en 15 páginas.
- **Alternativas**: layout por grupo de rutas (más archivos y duplicación); marco por página (peor
  mantenimiento).

### D3 — Navegación como modelo puro con RBAC

- **Decisión**: `lib/navegacion/navegacion.ts` exporta grupos e ítems tipados y una función
  `itemsVisibles(familiaRol, rutaActual, alertas)`; la barra lateral solo renderiza.
- **Razón**: la visibilidad por rol y el estado activo son lógica testeable sin DOM; cumple IV.
- **Alternativas**: condicionales `{#if}` dispersos en la barra (difícil de testear y duplicado).

### D4 — Conservar `tema.ts` y el script pre-pintado

- **Decisión**: se mantiene `frontend/src/lib/tema/tema.ts`, su test y el script inline de
  `app.html`. Se agrega `BotonTema.svelte` para la barra superior y se reutiliza el selector en el
  acceso.
- **Razón**: ya resuelve persistencia, `system`, cambio en vivo y anti-destello; reescribirlo sería
  trabajo sin valor.
- **Alternativas**: reimplementar el tema (riesgo de destello y regresión).

### D5 — Gráficos SVG/CSS con escalas puras

- **Decisión**: `GraficoBarras.svelte` y `GraficoDona.svelte` dibujan SVG con `viewBox`; la
  matemática vive en `lib/utils/graficos.ts` (`escalaBarras`, `arcoDona`, `porcentaje`) y tiene
  tests.
- **Razón**: no hay variación que justifique una librería; SVG hereda tokens y es accesible.
- **Alternativas**: Chart.js/ECharts (dependencia nueva, peso, tema duplicado); canvas (peor
  accesibilidad y nitidez).

### D6 — Iconografía inline

- **Decisión**: un `Icono.svelte` con un mapa de paths SVG de 18–20 px, trazo 1.75, `aria-hidden`
  por defecto y `title` opcional.
- **Razón**: evita una dependencia de iconos y mantiene el control de color.
- **Alternativas**: lucide/heroicons (dependencia y bundle extra).

### D7 — Tabla de datos compartida, no un data grid

- **Decisión**: `TablaDatos.svelte` resuelve el shell visual (encabezado, densidad, hover, scroll
  interno, estado vacío/cargando) y cada página declara sus columnas y acciones. La paginación sigue
  siendo la del backend con `Paginacion.svelte`.
- **Razón**: las tablas actuales ya tienen su lógica; solo cambia la presentación.
- **Alternativas**: grid genérico con sorting/filtros (funcionalidad nueva, fuera de alcance);
  virtualización (innecesaria con paginación de 20–100).

### D8 — Tablero con totales exactos y widgets por rol

- **Decisión**: los indicadores usan los `total` que ya devuelven los listados paginados:
  `listarStock({bajoMinimo:true, porPagina:1})`, `listarMovimientos({desde, porPagina:1})`,
  `listarProductos({activo:'true', porPagina:1})`; el valor del stock usa
  `/api/reportes/stock-valorizado` (GESTION) mediante un cliente nuevo
  `lib/api/reportes.ts`. Los gráficos cuentan por tipo y bajo mínimo con consultas acotadas, sin
  agregaciones nuevas en backend.
- **Razón**: no inventa series ni aproxima; respeta RBAC y no requiere endpoints nuevos.
- **Alternativas**: endpoint de dashboard (cambio de backend prohibido por el alcance); traer 10 000
  filas al cliente (lento e inexacto).

### D9 — Formularios y modales con primitivas

- **Decisión**: `Campo.svelte` encapsula label, ayuda y error; `Modal.svelte` encapsula overlay,
  foco, Escape y scroll lock. Los formularios de dominio conservan su lógica y solo cambian clases.
- **Razón**: los formularios actuales repiten markup y estilos; centralizarlos reduce divergencia.
- **Alternativas**: dejar cada formulario con Tailwind suelto (imposible garantizar consistencia).

### D10 — Responsive por breakpoints de contenido

- **Decisión**: ≥1280 barra lateral expandida y grillas de 3–4 columnas; 1024–1279 colapsada a
  iconos; <1024 panel superpuesto con backdrop; <640 una columna y barra superior compacta. Las
  tablas hacen scroll dentro de su tarjeta.
- **Razón**: el objetivo es escritorio; 320 px se garantiza sin desborde, no con una UX móvil nueva.
- **Alternativas**: diseño móvil completo (fuera de alcance); ocultar tablas en móvil (pierde
  información).

### D11 — Accesibilidad verificable

- **Decisión**: `aria-current="page"` en la navegación, landmarks, foco visible con token, botones
  icónicos con `aria-label`, modales con foco atrapado y Escape, gráficos con `role="img"` y texto
  alternativo, `prefers-reduced-motion` respetado.
- **Razón**: FR-085 y SC-034; además mejora la usabilidad general.
- **Alternativas**: auditoría posterior (más costosa y propensa a regresiones).

### D12 — Evidencia y pruebas

- **Decisión**: tests unitarios para `navegacion`, `formato` y `graficos` (Vitest, node);
  `svelte-check` para plantillas; script Playwright `scripts/capturar-rediseno.mjs` que recorre las
  18 pantallas en claro/oscuro y guarda en `specs/018.../evidencia/`. La suite existente sigue siendo la
  red funcional.
- **Razón**: cubre lógica, tipos y aceptación visual sin montar infraestructura de tests de
  componentes que el proyecto no tiene.
- **Alternativas**: tests de componentes con Testing Library (dependencia y configuración nuevas);
  solo revisión manual (sin trazabilidad).

## Verificación técnica local

- **Svelte 5 + runes**: el proyecto fuerza runes; los componentes nuevos usan `$props`, `$state`,
  `$derived` y snippets, igual que las páginas actuales.
- **Tailwind 4**: `layout.css` ya hace `@import 'tailwindcss'`; se importa `lib/diseno/tokens.css`
  antes de la base y los componentes usan `var(--token)` en clases propias dentro de
  `@layer components`, combinadas con utilidades de layout.
- **Vitest**: el proyecto solo tiene el project `server` (`environment: 'node'`); las funciones puras
  nuevas encajan sin configuración adicional.
- **Playwright**: `@playwright/test` y Chromium 1248 están instalados; el frontend corre en
  `http://localhost:5173` y el backend en `http://localhost:3000` (`.env.example`). El script puede
  tomar capturas headless sin servidor adicional.
- **Estado previo**: login y tokens crema/naranja están en el árbol de trabajo sin commit; se
  re-pielan en esta feature y no se descarta `tema.ts`.

## Incógnitas resueltas

Todas las incógnitas del `Technical Context` quedaron resueltas: no hay `NEEDS CLARIFICATION`
pendientes. Las cuatro decisiones de alcance, paleta, gráficos y evidencia fueron confirmadas por el
usuario en la sesión de clarificación del 2026-11-07.
