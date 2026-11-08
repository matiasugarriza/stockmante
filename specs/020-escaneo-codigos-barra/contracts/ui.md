# Contrato de UI: escaneo, asignación y etiquetas (020)

## Componentes nuevos (`frontend/src/lib/components/codigos/`)

### `BotonEscanear.svelte`

| Prop | Tipo | Notas |
|---|---|---|
| `onLectura` | `(lectura: { valor: string; formato: FormatoLectura }) => void` | se llama una sola vez por activación |
| `etiqueta` | `string?` | default «Escanear con cámara» (`aria-label`) |
| `deshabilitado` | `boolean?` | |

Botón con icono de cámara, área de toque ≥ 44×44 px (019). Abre `EscanerCamara`.

### `EscanerCamara.svelte`

- Overlay a pantalla completa en móvil y modal en escritorio; `role="dialog"`, `aria-modal`, foco
  atrapado, `Escape` cancela (FR-156).
- Video de la cámara trasera, recuadro guía y texto «Apuntá al código de barras o QR».
- Aviso `aria-live="polite"` con el estado: «Pidiendo permiso…», «Buscando código…», «Leído:
  7791234567898».
- Errores (de `clasificarErrorCamara`), siempre con qué pasó, por qué y qué hacer (P-III):

  | Tipo | Mensaje |
  |---|---|
  | `inseguro` | «La cámara no está disponible: la página no se abrió por HTTPS. Abrí Stockmante con `https://` o usá el lector o la búsqueda manual.» |
  | `sin_soporte` | «Este navegador no permite usar la cámara. Usá el lector físico o escribí el código.» |
  | `permiso_denegado` | «No diste permiso para usar la cámara. Habilitalo en los ajustes del navegador o escribí el código.» |
  | `sin_camara` | «No encontramos una cámara en este dispositivo. Usá el lector físico o escribí el código.» |
  | `ocupada` | «La cámara está en uso por otra aplicación. Cerrala y tocá Reintentar.» |
  | `desconocido` | «No se pudo iniciar la cámara. Tocá Reintentar o escribí el código.» |

- Botones: «Cancelar» siempre; «Reintentar» en error (sin recargar, FR-152).
- Apaga todas las pistas al leer, cancelar, en error y en `onDestroy` (FR-153).
- No bloquea el buscador de la pantalla: al cerrar, el foco vuelve al buscador (FR-153).

### `AsignarCodigo.svelte` (catálogo, solo gestión)

- Opciones «Automático» (muestra la vista previa calculada con `ean13Automatico(id)`) y «Desde 12
  dígitos» (campo numérico; muestra el EAN-13 resultante o el error de validación en vivo).
- Si el producto ya tiene código: muestra el actual y exige marcar «Reemplazar el código actual
  (7791234567898)» antes de habilitar «Asignar» (FR-145); envía `reemplazar: true`.
- Resultado: «Código 2000000000428 asignado a Yerba 1 kg (SKU YER-1KG).» (P-III, resultado
  concreto).

### `EtiquetaProducto.svelte` (catálogo, solo gestión)

- Selector de formato EAN-13 / Code 128 / QR según la tabla de
  [data-model.md § Etiqueta](../data-model.md#etiqueta); las opciones no disponibles se muestran
  deshabilitadas con su motivo.
- Vista previa a tamaño real 50 × 30 mm (FR-154) y aviso «Imprimí al 100 % (tamaño real), sin
  ajustar a la página».
- Botones «Imprimir» (`window.print()`) y «Descargar SVG» (`etiqueta-<SKU>-<formato>.svg`).
- Sin código y formato EAN-13 → botón «Generar código» que abre `AsignarCodigo`.

## Lógica pura nueva (`frontend/src/lib/codigos/`)

| Módulo | Funciones | Tests |
|---|---|---|
| `escaner.ts` | `normalizarLectura`, `clasificarErrorCamara`, `elegirLectura`, `FORMATOS_LECTURA` | `escaner.test.ts` |
| `ean13.ts` | `digitoVerificadorEan13`, `esEan13Valido`, `ean13Automatico`, `completarEan13` | `ean13.test.ts` |
| `etiqueta.ts` | `formatosDisponibles(producto)`, `valorCodificado(producto, formato)`, `nombreArchivo` | `etiqueta.test.ts` |

## Integración por pantalla

Ver la tabla de [research.md § R8](../research.md#r8--integración-en-las-5-pantallas). Reglas
comunes:

- El botón va junto al buscador de producto de la pantalla, en escritorio y en móvil.
- `encontrado` → acción propia de la pantalla; `inactivo` → aviso «El producto <nombre> (SKU) está
  dado de baja; no se puede agregar.»; `no_encontrado` → aviso «No se encontró un producto con el
  código <valor>. Buscalo por nombre o SKU.» y el valor queda en el buscador; `error_red` → aviso
  con «Reintentar» que reusa el valor leído (FR-155).
- Ninguna lectura escribe en la API (FR-150, SC-054).
- Los botones de asignar código y de etiqueta solo se muestran a gestión (FR-148).
