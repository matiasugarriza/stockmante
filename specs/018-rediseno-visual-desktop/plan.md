# Implementation Plan: Rediseño visual del escritorio

**Branch**: `018-rediseno-visual-desktop` | **Date**: 2026-11-07 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `specs/018-rediseno-visual-desktop/spec.md`

**Referencia visual**: `captures/samples/light-2.png` y `captures/samples/dark-2.png`
(panel StarAdmin, variantes clara y oscura).

## Summary

Rehacer la interfaz de escritorio de las 17 páginas capturadas en `captures/` con el lenguaje visual
de las muestras `light-2`/`dark-2`: marco con barra lateral y barra superior, tarjetas de superficie
sobre fondo gris azulado (claro) o azul noche (oscuro), acento primario azul, acentos teal y ámbar,
tablas de datos con badges y un tablero real en Inicio. El trabajo es **estrictamente de
presentación**: no cambia endpoints, contratos, esquema, permisos ni reglas de negocio. Se conserva
la lógica ya existente de tema (`frontend/src/lib/tema/tema.ts` y el script de `app.html`) y se
re-piela el login, que hoy usa una paleta crema/naranja retirada.

El enfoque es incremental y verificado: primero tokens, primitivas y marco; después acceso, tablero,
operación y administración; cada página se cierra con capturas en ambos temas y sin regresiones.

## Technical Context

**Language/Version**: TypeScript 6 + Svelte 5 (runes) sobre SvelteKit 3 en modo SPA, Vite 8 y
Tailwind CSS 4.

**Primary Dependencies**: las ya instaladas en `frontend/package.json` (SvelteKit, Tailwind, zod,
Vitest, svelte-check). NO se agregan librerías de UI, gráficos, iconos ni fuentes.

**Storage**: `localStorage` del navegador para la preferencia de tema (clave `stockmante:tema`). Sin
cambios de base de datos, migraciones ni endpoints.

**Testing**: Vitest en entorno `node` para lógica pura nueva (navegación, formato, escalas de
gráficos) y para la suite existente; `svelte-check` para tipos y plantillas; Playwright (ya
declarado en el `package.json` raíz) para las capturas de evidencia.

**Target Platform**: navegadores de escritorio modernos; 1440×900 como tamaño de referencia y
adaptación funcional hasta 320 px.

**Project Type**: aplicación web (frontend SvelteKit + backend Rust/Axum ya existente).

**Performance Goals**: sin regresión perceptible de carga; el tema se aplica antes del primer
pintado; sin layout shift al cargar tablas o gráficos; cero dependencias nuevas de runtime.

**Constraints**: sin cambios de comportamiento, API, esquema ni permisos; contraste WCAG AA en ambos
temas; sin desborde horizontal de 320 a 1440 px; textos en español (es-AR); un solo dueño de los
tokens visuales.

**Scale/Scope**: 18 rutas (17 capturas + el modo de cambio de contraseña), 8 páginas de
administración, 5 de operación, 1 tablero, 3 pantallas de acceso y ~20 componentes reutilizables.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principio | Cómo lo cumple el plan | Estado |
|---|---|---|
| **I. Simplicidad y anti-sobreingeniería** | No se agregan dependencias ni capas de estado; un catálogo acotado de componentes y un único archivo de tokens. Los gráficos son SVG propios porque no hay variación que justifique una librería. | PASS |
| **II. Mantenibilidad** | Componentes Svelte de una responsabilidad (`components/ui`, `components/marco`), lógica pura extraída a `lib/navegacion` y `lib/utils`, acceso HTTP solo por `lib/api/*.ts` (los clientes existentes; el tablero reutiliza `stock`, `movimientos`, `productos`, `reportes`). | PASS |
| **III. Claridad en mensajes** | Se conservan los mensajes reales de la API y los estados de error de cada página; el rediseño no reemplaza mensajes por genéricos y mantiene el resultado concreto post-acción (producto, depósito, stock). | PASS |
| **IV. Testabilidad primero** | Toda lógica nueva (navegación por rol, formato de números/fechas, escalas de gráficos) se escribe como función pura con test unitario previo; las plantillas se validan con `svelte-check` y las capturas cubren la aceptación visual. | PASS |
| **V. Integridad de datos** | Sin cambios en backend, SQL, transacciones ni stock; el tablero solo lee endpoints existentes y oculta widgets sin permiso. | PASS |

**Post-design re-check (Phase 1)**: los artefactos de diseño no introducen persistencia nueva,
escrituras ni dependencias; el principio IV se refuerza con pruebas unitarias de las funciones
puras y evidencia visual por página. Sin violaciones ni waivers.

## Project Structure

### Documentation (this feature)

```text
specs/018-rediseno-visual-desktop/
├── plan.md              # Este archivo
├── spec.md              # Especificación funcional
├── research.md          # Fase 0: decisiones y extracción de la referencia
├── data-model.md        # Fase 1: inventario de páginas, tokens, componentes y widgets
├── quickstart.md        # Fase 1: guía de validación end-to-end
├── contracts/
│   ├── tokens.md        # Contrato de tokens y variantes claro/oscuro
│   └── ui.md            # Contrato del marco, navegación, componentes y evidencia
├── checklists/
│   └── requirements.md  # Checklist de calidad de la spec
├── evidencia/           # Capturas de aceptación (claro, oscuro y 320 px)
└── tasks.md             # Fase 2 (/speckit.tasks)
```

### Source Code (repository root)

```text
frontend/
├── src/
│   ├── app.html                                  # Script de tema pre-pintado (se conserva)
│   ├── routes/
│   │   ├── +layout.svelte                        # Host del marco / lienzo de acceso
│   │   ├── layout.css                            # Importa tokens y base; retira estilos crema
│   │   ├── +page.svelte                          # Tablero de Inicio
│   │   ├── login/+page.svelte                    # Login + cambio obligatorio re-pielados
│   │   ├── registro/+page.svelte
│   │   ├── verificar-correo/+page.svelte
│   │   ├── caja/+page.svelte
│   │   ├── movimientos/+page.svelte
│   │   ├── transferencias/+page.svelte
│   │   ├── conteos/+page.svelte
│   │   ├── asistente/+page.svelte
│   │   ├── admin/+layout.svelte                  # Guard de GESTION (se conserva)
│   │   └── admin/{auditoria,categorias,compras,depositos,productos,proveedores,stock,usuarios}/+page.svelte
│   └── lib/
│       ├── diseno/
│       │   ├── tokens.css                        # Tokens semánticos claro/oscuro + base
│       │   └── acceso.css                        # Estilos del lienzo de acceso (ex login-*)
│       ├── components/
│       │   ├── ui/                               # Primitivas reutilizables
│       │   │   ├── Boton.svelte
│       │   │   ├── Campo.svelte
│       │   │   ├── EncabezadoPagina.svelte
│       │   │   ├── Tarjeta.svelte
│       │   │   ├── EtiquetaEstado.svelte
│       │   │   ├── TablaDatos.svelte
│       │   │   ├── Paginacion.svelte
│       │   │   ├── Modal.svelte
│       │   │   ├── Aviso.svelte
│       │   │   ├── EstadoVacio.svelte
│       │   │   ├── Esqueleto.svelte
│       │   │   ├── Icono.svelte
│       │   │   ├── TarjetaKpi.svelte
│       │   │   ├── GraficoBarras.svelte
│       │   │   ├── GraficoDona.svelte
│       │   │   └── BotonTema.svelte
│       │   ├── marco/
│       │   │   ├── MarcoApp.svelte
│       │   │   ├── BarraLateral.svelte
│       │   │   ├── BarraSuperior.svelte
│       │   │   ├── ItemNavegacion.svelte
│       │   │   ├── MenuUsuario.svelte
│       │   │   └── LienzoAcceso.svelte
│       │   └── inicio/
│       │       ├── PanelActividad.svelte
│       │       └── AccesosRapidos.svelte
│       ├── navegacion/
│       │   ├── navegacion.ts                     # Modelo puro de grupos/ítems/roles
│       │   └── navegacion.test.ts
│       ├── utils/
│       │   ├── formato.ts                        # Moneda, enteros, fechas cortas
│       │   ├── formato.test.ts
│       │   ├── graficos.ts                       # Escalas y arcos SVG puros
│       │   └── graficos.test.ts
│       ├── api/
│       │   ├── reportes.ts                       # Cliente tipado de /api/reportes/stock-valorizado
│       │   └── stock.ts / movimientos.ts / productos.ts / conteos.ts / ...
│       ├── schemas/
│       │   ├── reportes.ts
│       │   └── ...
│       ├── tema/tema.ts                          # Preferencia Light/Dark/System (se conserva)
│       └── components/{productos,usuarios,compras,proveedores,categorias,asistente}/*  # Re-piel
└── package.json
scripts/
└── capturar-rediseno.mjs                         # Playwright: capturas por ruta y tema
```

**Structure Decision**: se mantiene la estructura real de `frontend/src` y se agregan dos carpetas de
presentación (`lib/diseno` y `lib/components/{ui,marco}`) más dos módulos puros (`lib/navegacion`,
`lib/utils/{formato,graficos}`). Las páginas y componentes de dominio existentes se re-pielan en su
lugar; no se mueven de carpeta para no romper imports ni tests.

## Complexity Tracking

> Sin violaciones de la constitución. No se agregan proyectos, dependencias ni capas de
> persistencia.

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| — | — | — |

## Phase 0: Outline & Research

Salida: [research.md](./research.md). Decisiones principales:

1. **Identidad**: reemplazar la paleta crema/naranja por la de las muestras; extraer tokens
   semánticos y conservar `tema.ts`.
2. **Marco**: `+layout.svelte` decide entre `MarcoApp` (rutas privadas) y `LienzoAcceso` (públicas);
   la barra lateral se genera desde un modelo puro con visibilidad por rol.
3. **Gráficos**: SVG/CSS propios con funciones de escala puras y testeables; sin librería.
4. **Tablero**: indicadores y gráficos calculados con totales exactos de endpoints existentes; los
   widgets de gestión se ocultan a `OPERACION` y cada uno resuelve su error.
5. **Tablas**: un `TablaDatos` compartido para el shell visual y columnas declarativas por página;
   sin grid genérico ni virtualización (paginación del backend).
6. **Evidencia**: script Playwright que recorre las 18 rutas en claro/oscuro a 1440×900 y una
   comprobación de 320 px.

## Phase 1: Design & Contracts

Salidas:

- [data-model.md](./data-model.md): inventario de 18 rutas, grupos de navegación, tokens, catálogo de
  componentes, widgets del tablero y reglas de evidencia.
- [contracts/tokens.md](./contracts/tokens.md): nombres y valores de tokens en ambos temas, escalas
  tipográficas, radios, sombras y breakpoints.
- [contracts/ui.md](./contracts/ui.md): anatomía del marco, contrato de navegación, API de las
  primitivas, patrones de página y convención de capturas.
- [quickstart.md](./quickstart.md): puesta en marcha, escenarios de validación por historia y
  comandos de verificación.

**Post-design Constitution Check**: PASS (sin cambios respecto del gate previo).

## Implementation Order

1. **Fundaciones** (bloquea todo): tokens, tipografía, iconos, primitivas, tema, modelo de
   navegación y marco en `+layout.svelte`.
2. **Acceso**: login, cambio obligatorio, registro y verificación con el nuevo lienzo.
3. **Tablero**: Inicio con KPIs, gráficos, actividad y accesos rápidos.
4. **Operación**: Caja, Movimientos, Transferencias, Conteos y Asistente.
5. **Administración**: las ocho páginas de tabla y sus formularios/modales.
6. **Cierre**: contraste, teclado, 320 px, capturas, regresión y documentación.

## Risks & Mitigations

| Riesgo | Mitigación |
|---|---|
| El login actual tiene 340 líneas de CSS propio con la paleta retirada. | Migrar los estilos a `lib/diseno/acceso.css` con tokens y borrar las reglas crema; el componente conserva su lógica. |
| Cambios masivos en 18 páginas pueden romper flujos existentes. | Refactor por página con la suite en verde y evidencia visual; no se toca lógica de API ni validaciones. |
| El tablero requiere datos que la API no agrega. | Usar solo totales exactos disponibles (`total` de stock, movimientos, productos, reportes) y ocultar widgets sin endpoint o sin rol. |
| Tablas anchas en 320 px. | Contenedor con `overflow-x: auto` dentro de la tarjeta y columnas prioritarias; nunca overflow del `body`. |
| Dos temas con contraste insuficiente. | Tokens con pares texto/fondo verificados y lista de comprobación AA en quickstart. |
| Capturas manuales inconsistentes. | Script Playwright reproducible con viewport y tema fijados. |

## Verification Gates

- `npm run test` y `npm run check` en `frontend/` en verde (incluye tests nuevos de navegación,
  formato y gráficos).
- Cero errores de `svelte-check` y cero `console.error` en las rutas recorridas.
- Capturas de las 18 rutas en claro y oscuro + chequeo de 320 px en las páginas con tabla.
- Recorrido manual de los cuatro flujos de acceso y de una acción por página operativa/administrativa
  con el mismo resultado que antes.
