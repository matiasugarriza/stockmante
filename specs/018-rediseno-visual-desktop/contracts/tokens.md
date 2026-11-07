# Contrato de tokens visuales

**Feature**: `018-rediseno-visual-desktop` | **Archivo dueño**: `frontend/src/lib/diseno/tokens.css`

Este contrato es la única fuente de verdad del color, la tipografía, el espaciado y la forma. Las
páginas y componentes consumen tokens semánticos; no declaran colores de marca propios.

## 1. Contrato CSS

```css
/* frontend/src/lib/diseno/tokens.css — paleta derivada del logo (FR-101) */
:root {
  --fuente-sans:
    'Outfit Variable', 'Outfit', ui-sans-serif, system-ui, -apple-system, 'Segoe UI', Roboto,
    'Helvetica Neue', Arial, sans-serif;
  --fuente-mono:
    ui-monospace, 'SFMono-Regular', 'Cascadia Mono', Menlo, Consolas, monospace;

  --fondo: #f1f6f7;
  --fondo-elevado: #f6fafa;
  --superficie: #ffffff;
  --superficie-2: #f3f8f9;
  --superficie-3: #e4eef0;
  --sidebar: #ffffff;
  --topbar: #0d1518;

  --borde: #dce7ea;
  --borde-fuerte: #c2d3d8;

  --texto: #10212a;
  --texto-suave: #4c626c;
  --texto-tenue: #526872;
  --texto-inverso: #ffffff;

  /* Marca: petróleo/cian del logo (FR-101). */
  --primario: #0b7a91;
  --primario-hover: #09667a;
  --primario-suave: #ddf3f6;
  --primario-texto: #086a7e;
  --primario-contraste: #ffffff;

  --ambar: #f0a63a;
  --ambar-suave: #fdf3e2;
  --ambar-texto: #8a5a12;
  --exito: #22a45d;
  --exito-suave: #e7f7ee;
  --exito-texto: #167a45;
  --peligro: #cc3238;
  --peligro-suave: #fdecec;
  --peligro-texto: #b42318;
  --info: #0b7a91;
  --info-suave: #ddf3f6;

  --foco: #0b7a91;
  --logo-inicio: #16b3cc;
  --logo-fin: #0b7a91;
  --logo-acento: #0b7a91;
  --sombra-tarjeta:
    0 1px 2px rgba(13, 33, 40, 0.05), 0 8px 24px -18px rgba(13, 33, 40, 0.25);
  --sombra-elevada: 0 12px 32px -12px rgba(13, 33, 40, 0.28);

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
  --topbar-alto: 72px;
  --contenido-max: 1440px;
}

html[data-tema='dark'] {
  --fondo: #0d1518;
  --fondo-elevado: #111b1f;
  --superficie: #141f23;
  --superficie-2: #19282d;
  --superficie-3: #1f2f35;
  --sidebar: #10191c;
  --topbar: var(--fondo);

  --borde: #22343a;
  --borde-fuerte: #31484f;

  --texto: #e7f2f4;
  --texto-suave: #9fb5bb;
  --texto-tenue: #8ea5ac;
  --texto-inverso: #06262b;

  /* Marca: cian del logo (FR-101); el texto sobre el primario es oscuro. */
  --primario: #2fd0dc;
  --primario-hover: #5becef;
  --primario-suave: #123940;
  --primario-texto: #5becef;
  --primario-contraste: #06262b;

  --ambar: #f3b45c;
  --ambar-suave: #3a2c17;
  --ambar-texto: #f3b45c;
  --exito: #3ecf7a;
  --exito-suave: #12301f;
  --exito-texto: #3ecf7a;
  --peligro: #ff6b6e;
  --peligro-suave: #3a1a1d;
  --peligro-texto: #ff8a8c;
  --info: #2fd0dc;
  --info-suave: #123940;

  --foco: #5becef;
  --logo-inicio: #5becef;
  --logo-fin: #1fa3c4;
  --logo-acento: #5becef;
  --sombra-tarjeta:
    0 1px 0 rgba(0, 0, 0, 0.35), 0 18px 40px -24px rgba(0, 0, 0, 0.85);
  --sombra-elevada: 0 20px 48px -18px rgba(0, 0, 0, 0.9);
}

html:not([data-tema='dark']) .zona-oscura {
  color-scheme: dark;
  --texto: #eef7f8;
  --texto-suave: #a9bec4;
  --texto-tenue: #93aab1;
  --superficie: #16242a;
  --superficie-2: #1c2e34;
  --superficie-3: #23383f;
  --borde: #26393f;
  --borde-fuerte: #364f57;
  --primario-suave: #143a42;
  --primario-texto: #7fe8f0;
  --foco: #7fe8f0;
  --sombra-elevada: 0 18px 40px -16px rgba(0, 0, 0, 0.7);
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
| `--texto` sobre `--superficie` | `#10212a` / `#ffffff` (16.5) | `#e7f2f4` / `#141f23` (14.72) | 4.5:1 |
| `--texto-suave` sobre `--superficie` | `#4c626c` / `#ffffff` (6.42) | `#9fb5bb` / `#141f23` (7.84) | 4.5:1 |
| `--texto-tenue` sobre `--superficie-3` | `#526872` / `#e4eef0` (4.97) | `#8ea5ac` / `#1f2f35` (5.36) | 4.5:1 |
| `--primario-contraste` sobre `--primario` | `#ffffff` / `#0b7a91` (4.99) | `#06262b` / `#2fd0dc` (8.47) | 4.5:1 |
| `--primario-contraste` sobre `--primario-hover` | `#ffffff` / `#09667a` (6.58) | `#06262b` / `#5becef` (11.13) | 4.5:1 |
| `--primario-texto` sobre `--superficie` | `#086a7e` / `#ffffff` (6.23) | `#5becef` / `#141f23` (11.75) | 4.5:1 |
| `--primario-texto` sobre `--primario-suave` | `#086a7e` / `#ddf3f6` (5.40) | `#5becef` / `#123940` (8.72) | 4.5:1 |
| Texto de `.zona-oscura` sobre `--topbar` (solo claro) | `#eef7f8` / `#0d1518` (16.96); suave `#a9bec4` (9.54); tenue `#93aab1` (7.58) | — (topbar = fondo) | 4.5:1 |
| `--texto-inverso` sobre `--peligro` (contador, botón peligro) | `#ffffff` / `#cc3238` (5.15) | `#06262b` / `#ff6b6e` (5.74) | 4.5:1 |
| `--peligro` sobre `--peligro-suave` | `#b42318` / `#fdecec` | `#ff8a8c` / `#3a1a1d` | 4.5:1 |
| `--exito` sobre `--exito-suave` | `#167a45` / `#e7f7ee` | `#3ecf7a` / `#12301f` | 4.5:1 |
| `--ambar` sobre `--ambar-suave` | `#8a5a12` / `#fdf3e2` | `#f3b45c` / `#3a2c17` | 4.5:1 |

**Logo (FR-100)**: `--logo-inicio`/`--logo-fin` definen el degradé vertical del símbolo
(claro `#16b3cc → #0b7a91`; oscuro `#5becef → #1fa3c4`, los colores del logo original) y
`--logo-acento` colorea "ST" del logotipo (claro `#0b7a91`, 4.53:1 sobre `--fondo`; oscuro
`#5becef`, 12.07:1 sobre `--sidebar`).

**Paleta (FR-101)**: el primario es el petróleo/cian del logo; el fondo oscuro y la barra superior
clara usan su negro petróleo `#0d1518`. Se retiraron `--teal*` y `--cian` (coincidían con el
primario); ámbar, verde y rojo quedan como colores de estado.

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
