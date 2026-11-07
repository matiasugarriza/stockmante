# Contrato de interfaz: marco, navegación y componentes

**Feature**: `018-rediseno-visual-desktop` | **Alcance**: presentación

Este contrato fija la anatomía del marco, el modelo de navegación, la API de las primitivas y los
patrones de página. **No cambia contratos HTTP, esquema, permisos ni reglas de negocio**; los
clientes de `frontend/src/lib/api/*.ts` y sus schemas siguen siendo la única vía de acceso a datos.

## 1. Composición del layout

```text
+layout.svelte
├── esRutaPublica(pathname)
│   ├── true  → LienzoAcceso (sin sidebar/topbar)
│   └── false → MarcoApp
│                 ├── BarraLateral
│                 │   ├── Marca
│                 │   ├── GrupoNavegacion × 2
│                 │   └── VersionSistema
│                 ├── BarraSuperior
│                 │   ├── BotonMenu (solo < 1024)
│                 │   ├── ContextoPagina (título + migaja)
│                 │   ├── AccionesRapidas (opcional por página)
│                 │   ├── BotonTema (único selector de tema del marco, T100)
│                 │   └── MenuUsuario (nombre, rol, cerrar sesión)
│                 └── <main id="contenido"> children </main>
```

**Medidas**: sidebar 248 px (76 px colapsada), topbar 72 px, contenido con padding 24 px y ancho
máximo 1440 px centrado. En < 1024 px la sidebar pasa a panel superpuesto con backdrop y `Escape`.

**Regla**: las rutas públicas existentes (`/login`, `/registro`, `/verificar-correo`) no muestran
marco; el resto sí, incluso antes de que el guard de sesión redirija.

## 2. Contrato de navegación

```ts
// frontend/src/lib/navegacion/navegacion.ts
export interface ContextoNavegacion {
  familiaRol: 'GESTION' | 'OPERACION';
  alertasStock: number;
}

export interface ItemNavegacionVisible extends ItemNavegacion {
  activo: boolean;
  contadorVisible: number | null;
}

export function gruposVisibles(
  rutaActual: string,
  contexto: ContextoNavegacion
): Array<{ id: GrupoNavegacion; etiqueta: string; items: ItemNavegacionVisible[] }>;
```

**Garantías**:

- `gruposVisibles` es pura, determinista y no toca `window`; se testea con Vitest.
- Un ítem sin roles compatibles no se devuelve.
- `activo` es `true` solo en la ruta exacta `/` o por prefijo en el resto; nunca hay dos activos.
- El contador de Stock solo aparece si `alertasStock > 0` y el rol es `GESTION`.
- El orden de los grupos es Operación → Administración; dentro de cada uno, el orden de la tabla de
  `data-model.md`.
- El total de alertas se sigue cargando como hoy en `+layout.svelte`; si falla, se usa `0` sin
  romper el marco.

## 3. API de primitivas

Todas las primitivas usan runes y snippets de Svelte 5.

### `Boton.svelte`

```ts
interface Props {
  variante?: 'primario' | 'secundario' | 'fantasma' | 'peligro';
  tamano?: 'sm' | 'md';
  tipo?: 'button' | 'submit' | 'reset';
  href?: string;
  cargando?: boolean;
  deshabilitado?: boolean;
  icono?: NombreIcono;
  onclick?: (event: MouseEvent) => void;
  children: Snippet;
}
```

- Renderiza `<button>`; si hay `href`, se usa una variante `<a>` con el mismo estilo.
- `cargando` deshabilita y muestra spinner; conserva el ancho para evitar layout shift.
- Foco visible con `outline: 2px solid var(--foco)` y `outline-offset: 2px`.

### `Campo.svelte`

```ts
interface Props {
  etiqueta: string;
  nombre: string;
  tipo?: 'text' | 'email' | 'password' | 'number' | 'search' | 'date' | 'select' | 'textarea';
  valor?: string | number;
  opciones?: Array<{ valor: string; etiqueta: string }>;
  error?: string | null;
  ayuda?: string;
  requerido?: boolean;
  deshabilitado?: boolean;
  autocomplete?: string;
  oninput?: (valor: string) => void;
  onchange?: (valor: string) => void;
  children?: Snippet;
}
```

- `label[for]` asociado; `aria-invalid` y `aria-describedby` cuando hay error/ayuda.
- El error se anuncia con `role="alert"`; no se usa placeholder como etiqueta.

### `Tarjeta.svelte`

```ts
interface Props {
  titulo?: string;
  descripcion?: string;
  accion?: Snippet;
  children: Snippet;
  pie?: Snippet;
  densa?: boolean;
}
```

Superficie con `background: var(--superficie)`, borde, radio y sombra de tarjeta.

### `TablaDatos.svelte`

```ts
interface Columna<T> {
  clave: string;
  encabezado: string;
  alineacion?: 'izquierda' | 'derecha' | 'centro';
  clase?: string;
}

interface Props<T> {
  columnas: Columna<T>[];
  filas: T[];
  cargando?: boolean;
  error?: string | null;
  vacio?: Snippet;
  fila?: Snippet<[T]>;
  compacta?: boolean;
  pie?: Snippet;
  etiqueta: string; // aria-label de la tabla
}
```

- El contenedor hace `overflow-x: auto`; el `body` nunca se desborda.
- `cargando` muestra esqueletos de fila; `error` muestra un aviso con reintento; `vacio` un estado
  vacío con acción.
- El encabezado usa el rol `etiqueta` y las filas `border-top: 1px solid var(--borde)`.

### `Modal.svelte`

```ts
interface Props {
  abierto: boolean;
  titulo: string;
  descripcion?: string;
  oncerrar: () => void;
  children: Snippet;
  acciones?: Snippet;
}
```

- `role="dialog"`, `aria-modal="true"`, foco inicial en el primer control, foco atrapado, `Escape` y
  clic en backdrop cierran; al cerrar se devuelve el foco al disparador.
- Bloquea el scroll del `body` mientras está abierto.

### `GraficoBarras.svelte` / `GraficoDona.svelte`

```ts
interface DatoGrafico { etiqueta: string; valor: number; color?: string }

interface PropsBarras { datos: DatoGrafico[]; alto?: number; etiqueta: string; }
interface PropsDona { datos: DatoGrafico[]; etiqueta: string; }
```

- SVG con `viewBox`, `role="img"` y `<title>`/`<desc>`; la leyenda es HTML, no texto dentro del SVG.
- Sin datos: `EstadoVacio`, no un SVG vacío.
- La matemática (`escalaBarras`, `arcoDona`) vive en `lib/utils/graficos.ts` y se testea.

### `Icono.svelte`

```ts
type NombreIcono =
  | 'inicio' | 'caja' | 'movimientos' | 'transferencias' | 'conteos' | 'asistente'
  | 'productos' | 'categorias' | 'depositos' | 'stock' | 'proveedores' | 'compras'
  | 'usuarios' | 'auditoria' | 'menu' | 'cerrar' | 'sol' | 'luna'
  | 'sistema' | 'buscar' | 'flecha' | 'alerta' | 'check';
```

- 18–20 px, `stroke="currentColor"`, `stroke-width="1.75"`, `fill="none"`.
- `aria-hidden="true"` por defecto; con `titulo` se renderiza `<title>`.

## 4. Patrones de página

### 4.1 Acceso

```text
LienzoAcceso (fondo --fondo, centrado)
└── panel 392–420 px
    ├── BotonTema alineado a la derecha
    ├── Tarjeta
    │   ├── Marca + kicker + h1
    │   ├── subtítulo
    │   ├── Campo × N
    │   ├── Aviso (error/aviso)
    │   └── Boton primario
    └── enlaces secundarios (SSO, registro, volver)
```

### 4.2 Operativa

```text
EncabezadoPagina (título + descripción + acción principal)
├── Tarjeta "Formulario" (campos en grilla 1/2 columnas)
├── Aviso (resultado concreto o error)
└── Tarjeta "Historial/resultado"
    ├── barra de filtros
    └── TablaDatos + Paginacion
```

### 4.3 Administración

```text
EncabezadoPagina (título + descripción + botón "Nuevo …")
├── Tarjeta de filtros (búsqueda, select, tabs de estado)
├── Aviso (error/éxito)
└── Tarjeta de tabla
    ├── TablaDatos
    ├── contador "N de M … · página X de Y"
    └── Paginacion
```

### 4.4 Tablero

```text
EncabezadoPagina (saludo + fecha)
├── grilla de TarjetaKpi (4/2/1 columnas)
├── grilla 2/3 + 1/3
│   ├── Tarjeta "Movimientos por tipo" (GraficoBarras)
│   └── Tarjeta "Estado del stock" (GraficoDona)
├── grilla 2/3 + 1/3
│   ├── Tarjeta "Actividad reciente" (lista)
│   └── Tarjeta "Casos urgentes" (lista de bajo mínimo)
└── AccesosRapidos
```

## 5. Contrato de estados

| Estado | Representación | Regla |
|---|---|---|
| Carga | `Esqueleto` o spinner en el botón | No se muestra "vacío" mientras carga |
| Vacío | `EstadoVacio` con icono, texto y acción | Nunca una tabla en blanco |
| Error | `Aviso` variante error con mensaje real y reintento | No se oculta el motivo; no se usa "Error interno" |
| Éxito | `Aviso` variante éxito o badge | Incluye el resultado concreto (producto, depósito, stock) |
| Guardado | Botón con spinner y texto "Guardando…" | Evita doble envío |
| Sin permiso | Widget/ítem oculto | No se muestra 403 crudo |

## 6. Contrato de accesibilidad

1. Un solo `<main id="contenido">` por página; `<nav aria-label="Navegación principal">` en la
   sidebar; `<header>` en la topbar.
2. `aria-current="page"` en el ítem activo; botones icónicos con `aria-label`.
3. Orden de tabulación: marca → navegación → topbar → contenido; sin trampas fuera del modal.
4. Foco visible en todos los interactivos con `--foco`; contraste de foco ≥ 3:1.
5. `prefers-reduced-motion: reduce` desactiva transiciones no esenciales.
6. Los gráficos tienen texto alternativo con la serie y los valores; las tablas tienen `caption` o
   `aria-label`.
7. Los formularios mantienen sus labels y mensajes; los errores se anuncian con `role="alert"`.

## 7. Contrato de evidencia

| Aspecto | Regla |
|---|---|
| Navegador | Chromium headless de Playwright |
| Viewport | 1440×900 (referencia) y 320×800 (angosto) |
| Tema | `localStorage['stockmante:tema']` + `data-tema` antes de navegar |
| Espera | `networkidle` + `document.fonts.ready` + 150 ms de estabilización |
| Datos | Usuario de prueba con datos sembrados; sin credenciales reales en la imagen |
| Destino | `specs/018-rediseno-visual-desktop/evidencia/{claro,oscuro,angosto}/` |
| Nombres | Mismo nombre que la captura original (`04-inicio.png`, etc.) |
| Criterio | Cada imagen se compara contra `captures/` y `samples/` en la revisión |

## 8. Fuera del contrato

- No se crean rutas, endpoints, tablas ni permisos.
- No se cambian los textos funcionales ni los mensajes de la API.
- No se introduce un sistema de componentes de terceros ni theming por CSS-in-JS.
