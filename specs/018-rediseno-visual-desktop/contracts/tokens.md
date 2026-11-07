# Contrato de tokens visuales

**Feature**: `018-rediseno-visual-desktop` | **Archivo dueño**: `frontend/src/lib/diseno/tokens.css`

Este contrato es la única fuente de verdad del color, la tipografía, el espaciado y la forma. Las
páginas y componentes consumen tokens semánticos; no declaran colores de marca propios.

## 1. Contrato CSS

```css
/* frontend/src/lib/diseno/tokens.css */
:root {
  --fuente-sans: ui-sans-serif, system-ui, -apple-system, 'Segoe UI', Roboto,
    'Helvetica Neue', Arial, sans-serif;
  --fuente-mono: ui-monospace, 'SFMono-Regular', 'Cascadia Mono', Menlo, Consolas, monospace;

  --fondo: #f1f4f9;
  --fondo-elevado: #f7f9fc;
  --superficie: #ffffff;
  --superficie-2: #f5f7fb;
  --superficie-3: #eaeff7;
  --sidebar: #ffffff;
  --topbar: #ffffff;

  --borde: #e3e8f0;
  --borde-fuerte: #c9d3e0;

  --texto: #1b2432;
  --texto-suave: #5f6f86;
  --texto-tenue: #93a1b5;
  --texto-inverso: #ffffff;

  --primario: #2f6bed;
  --primario-hover: #2459d1;
  --primario-suave: #e9f0ff;
  --primario-contraste: #ffffff;

  --teal: #14b8bd;
  --teal-suave: #e3f8f8;
  --ambar: #f0a63a;
  --ambar-suave: #fdf3e2;
  --exito: #22a45d;
  --exito-suave: #e7f7ee;
  --peligro: #dc3d43;
  --peligro-suave: #fdecec;
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
  --texto-tenue: #6c7c93;
  --texto-inverso: #0d1320;

  --primario: #4c8dff;
  --primario-hover: #6ba1ff;
  --primario-suave: #1c2b48;
  --primario-contraste: #0d1320;

  --teal: #2fd0d6;
  --teal-suave: #123639;
  --ambar: #f3b45c;
  --ambar-suave: #3a2c17;
  --exito: #3ecf7a;
  --exito-suave: #12301f;
  --peligro: #ff6b6e;
  --peligro-suave: #3a1a1d;
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

**Regla**: los números de métricas y tablas usan `tabular-nums`; no se cargan fuentes externas.

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
| `--primario-contraste` sobre `--primario` | `#ffffff` / `#2f6bed` | `#0d1320` / `#4c8dff` | 4.5:1 |
| `--peligro` sobre `--peligro-suave` | `#dc3d43` / `#fdecec` | `#ff6b6e` / `#3a1a1d` | 4.5:1 |
| `--exito` sobre `--exito-suave` | `#22a45d` / `#e7f7ee` | `#3ecf7a` / `#12301f` | 4.5:1 |
| `--ambar` sobre `--ambar-suave` | `#8a5a12` / `#fdf3e2` | `#f3b45c` / `#3a2c17` | 4.5:1 |

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
