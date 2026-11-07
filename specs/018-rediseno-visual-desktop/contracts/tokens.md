# Contrato de tokens visuales

**Feature**: `018-rediseno-visual-desktop` | **Archivo dueño**: `frontend/src/lib/diseno/tokens.css`

Este contrato es la única fuente de verdad del color, la tipografía, el espaciado y la forma. Las
páginas y componentes consumen tokens semánticos; no declaran colores de marca propios.

## 1. Contrato CSS

```css
/* frontend/src/lib/diseno/tokens.css */
:root {
  --fuente-sans: 'Outfit Variable', 'Outfit', ui-sans-serif, system-ui, -apple-system,
    'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
  --fuente-mono: ui-monospace, 'SFMono-Regular', 'Cascadia Mono', Menlo, Consolas, monospace;

  --fondo: #f1f4f9;
  --fondo-elevado: #f7f9fc;
  --superficie: #ffffff;
  --superficie-2: #f5f7fb;
  --superficie-3: #eaeff7;
  --sidebar: #ffffff;
  --topbar: #1f2a44;          /* barra superior azul marino de light-2 (FR-099) */

  --borde: #e3e8f0;
  --borde-fuerte: #c9d3e0;

  --texto: #1b2432;
  --texto-suave: #5f6f86;
  --texto-tenue: #5a6b83;
  --texto-inverso: #ffffff;

  --primario: #1f3bb3;        /* azul real de light-2 (FR-099) */
  --primario-hover: #182f94;
  --primario-suave: #e8ecfb;
  --primario-texto: #1f3bb3;
  --primario-contraste: #ffffff;

  --teal: #14b8bd;
  --teal-suave: #e3f8f8;
  --teal-texto: #0f7f84;
  --ambar: #f0a63a;
  --ambar-suave: #fdf3e2;
  --ambar-texto: #8a5a12;
  --exito: #22a45d;
  --exito-suave: #e7f7ee;
  --exito-texto: #167a45;
  --peligro: #dc3d43;
  --peligro-suave: #fdecec;
  --peligro-texto: #b42318;
  --info: #2f6bed;
  --info-suave: #e9f0ff;

  --foco: #2f6bed;
  --sombra-tarjeta: 0 1px 2px rgba(16, 24, 40, 0.05), 0 8px 24px -18px rgba(16, 24, 40, 0.25);
  --sombra-elevada: 0 12px 32px -12px rgba(16, 24, 40, 0.28);

  --radio-tarjeta: 10px;
  --radio-control: 6px;
  --radio-pildora: 999px;

  --espacio-1: 4px;
  --espacio-2: 8px;
  --espacio-3: 12px;
  --espacio-4: 16px;
  --espacio-5: 20px;
  --espacio-6: 24px;
  --espacio-8: 32px;

  --sidebar-ancho: 248px;
  --sidebar-ancho-colapsado: 76px;
  --topbar-alto: 64px;
  --contenido-max: 1440px;
}

html[data-tema='dark'] {
  --fondo: #121722;
  --fondo-elevado: #161c28;
  --superficie: #1b2230;
  --superficie-2: #222b3b;
  --superficie-3: #2a3447;
  --sidebar: #151b26;
  --topbar: #1a212d;

  --borde: #2b3648;
  --borde-fuerte: #3b4a63;

  --texto: #e8edf6;
  --texto-suave: #9aa9bf;
  --texto-tenue: #8f9db1;
  --texto-inverso: #0d1320;

  --primario: #3f5efb;        /* azul eléctrico de dark-2 (FR-099) */
  --primario-hover: #5872ff;
  --primario-suave: #1f2752;
  --primario-texto: #93a5ff;
  --primario-contraste: #ffffff;

  --teal: #2fd0d6;
  --teal-suave: #123639;
  --teal-texto: #2fd0d6;
  --ambar: #f3b45c;
  --ambar-suave: #3a2c17;
  --ambar-texto: #f3b45c;
  --exito: #3ecf7a;
  --exito-suave: #12301f;
  --exito-texto: #3ecf7a;
  --peligro: #ff6b6e;
  --peligro-suave: #3a1a1d;
  --peligro-texto: #ff8a8c;
  --info: #4c8dff;
  --info-suave: #1c2b48;

  --foco: #6ba1ff;
  --sombra-tarjeta: 0 1px 0 rgba(0, 0, 0, 0.35), 0 18px 40px -24px rgba(0, 0, 0, 0.85);
  --sombra-elevada: 0 20px 48px -18px rgba(0, 0, 0, 0.9);
}
```

## 2. Tipografía

| Token de rol | Tamaño / peso / alto | Uso |
|---|---|---|
| `titulo-pagina` | 22px / 700 / 1.2, tracking −0.02em | `h1` de cada página |
| `titulo-tarjeta` | 15px / 600 / 1.3 | Encabezado de tarjeta |
| `subtitulo` | 13px / 400 / 1.45, `--texto-suave` | Descripción bajo el título |
| `cuerpo` | 14px / 400 / 1.45 | Texto y tablas |
| `etiqueta` | 11px / 600 / 1.2, mayúsculas, tracking 0.06em, `--texto-tenue` | Encabezados de tabla, metadatos |
| `metrica` | 28px / 700 / 1.1, `font-variant-numeric: tabular-nums` | KPI |
| `mono` | 12px / 500 / 1.3, `--fuente-mono` | SKU, IDs, fechas técnicas |

**Regla**: la fuente sans es **Outfit** (FR-098), autoalojada con `@fontsource-variable/outfit`
e importada desde `routes/layout.css`; no se cargan fuentes desde CDN. Los números de métricas y
tablas usan `tabular-nums`. La monoespaciada se reserva para SKU, IDs y valores técnicos.

## 3. Espaciado, radios y sombras

- Grilla base de 4 px; separación entre tarjetas 16–20 px; padding interno de tarjeta 20–24 px.
- Radios: tarjetas y modales 10 px; botones, inputs y badges suaves 6 px; pills 999 px.
- Sombras: una sola `--sombra-tarjeta` para superficies; `--sombra-elevada` solo para modal/menú.
- Bordes: 1 px `--borde` en tarjetas; `--borde-fuerte` en controles interactivos.

## 4. Breakpoints

| Nombre | Ancho | Comportamiento |
|---|---|---|
| `sm` | < 640px | Una columna, topbar compacta, tablas con scroll interno |
| `md` | 640–1023px | Dos columnas de KPI, sidebar superpuesta cerrada |
| `lg` | 1024–1279px | Sidebar colapsada a iconos, grillas de 2–3 columnas |
| `xl` | ≥ 1280px | Sidebar expandida, KPI en 4 columnas, contenido de 1440 px como referencia |

## 5. Pares de contraste obligatorios (WCAG AA)

| Par | Claro | Oscuro | Mínimo |
|---|---|---|---|
| `--texto` sobre `--superficie` | `#1b2432` / `#ffffff` | `#e8edf6` / `#1b2230` | 4.5:1 |
| `--texto-suave` sobre `--superficie` | `#5f6f86` / `#ffffff` | `#9aa9bf` / `#1b2230` | 4.5:1 |
| `--texto-tenue` sobre `--superficie-3` | `#5a6b83` / `#eaeff7` | `#8f9db1` / `#2a3447` | 4.5:1 |
| `--primario-contraste` sobre `--primario` | `#ffffff` / `#1f3bb3` (8.99) | `#ffffff` / `#3f5efb` (4.99) | 4.5:1 |
| `--primario-texto` sobre `--superficie` | `#1f3bb3` / `#ffffff` (8.99) | `#93a5ff` / `#1b2230` (6.88) | 4.5:1 |
| Texto de `.zona-oscura` sobre `--topbar` (solo claro) | `#f1f4fb` / `#1f2a44` (12.95); suave `#b7c1d6` (7.88) | — (topbar = fondo) | 4.5:1 |
| `--peligro` sobre `--peligro-suave` | `#b42318` / `#fdecec` | `#ff8a8c` / `#3a1a1d` | 4.5:1 |
| `--exito` sobre `--exito-suave` | `#167a45` / `#e7f7ee` | `#3ecf7a` / `#12301f` | 4.5:1 |
| `--ambar` sobre `--ambar-suave` | `#8a5a12` / `#fdf3e2` | `#f3b45c` / `#3a2c17` | 4.5:1 |

**Zona oscura (FR-099)**: en tema claro la barra superior lleva la clase `.zona-oscura`, que
redefine localmente `--texto`, `--texto-suave`, `--texto-tenue`, `--superficie*`, `--borde` y
`--primario-suave/-texto` con valores oscuros; así `BotonTema` y `MenuUsuario` se adaptan sin
estilos propios. En tema oscuro la clase no aplica y la barra usa el fondo de la página.

**Nota**: para texto sobre fondos suaves se usa una variante oscurecida del acento (p. ej. ámbar
`#8a5a12` en claro) si el acento puro no alcanza 4.5:1; la verificación real se hace con el
validador de contraste en `quickstart.md`.

## 6. Reglas de uso

1. `tokens.css` se importa una sola vez, antes de la base de Tailwind en `routes/layout.css`.
2. Ningún archivo fuera de `tokens.css` declara `#rrggbb` de marca; se permiten
   `color-mix(in oklab, var(--token) …)` para estados derivados.
3. Los estados nunca se comunican solo por color: badge con texto, icono o ambos.
4. `color-scheme` se fija con el tema (`light`/`dark`) para controles nativos.
5. El tema se aplica con `data-tema` en `<html>`; no se usan clases `dark:` de Tailwind para la
   identidad, así el mismo markup sirve a ambos temas.
