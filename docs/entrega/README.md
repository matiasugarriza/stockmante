# Entrega — Grupo 2

Documento de entrega del Trabajo Práctico, con la estética del proyecto en modo claro.

## Archivos

| Archivo | Uso |
|---|---|
| [`entrega-grupo-2.pdf`](./entrega-grupo-2.pdf) | **PDF final** para subir |
| [`entrega-grupo-2.html`](./entrega-grupo-2.html) | Fuente editable (carátula + casos de uso) |
| [`assets/`](./assets/) | Logo, captura y tipografía Outfit usados por el HTML |
| [`preview-1.png`](./preview-1.png), [`preview-2.png`](./preview-2.png) | Vista previa de cada página |

## Contenido

- Página 1: carátula con grupo, resumen del proyecto, tecnologías y métricas.
- Página 2: los 10 casos de uso con descripción breve y estado; alcance adicional y proceso.

## Regenerar

```bash
node scripts/generar-entrega-pdf.mjs
```

El script abre el HTML con Chromium/Playwright y exporta A4 con las fuentes e imágenes
embebidas. No requiere servidores levantados.

## Editar integrantes o datos

1. Abrir `entrega-grupo-2.html`.
2. Cambiar el bloque `.meta` de la página 1 (grupo, integrantes, fecha) y los pies de página.
3. Ejecutar el script de regeneración.

La captura de la portada es `assets/captura-productos.png` (productos en tema claro). Se puede
reemplazar por otra captura del sistema manteniendo la proporción.
