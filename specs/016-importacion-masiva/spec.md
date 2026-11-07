# Especificación de Funcionalidad: Importación masiva de productos y stock

**Spec**: `016-importacion-masiva` | **Caso de uso**: US16 | **Prioridad**: P3

**Creado**: 2026-10-26 | **Estado**: Clarificado, planificado y con tasks (2026-11-07) | **Módulo**: 4 —
Gobierno y asistencia

**Origen**: "importación masiva de productos (CSV)" figura fuera del alcance del Módulo 1. Es
la vía práctica para migrar un negocio que ya tiene planillas.

**Depende de**: 003 (productos y categorías), 007 (stock inicial como movimiento).

## Clarifications

### Session 2026-11-07

- Q: ¿Qué dialecto exacto de CSV debe aceptar la importación? → A: Autodetección coherente: el delimitador `,` o `;` se detecta por el encabezado; el decimal se interpreta como `,` cuando el delimitador es `;` y como `.` cuando es `,`; se toleran comillas dobles RFC 4180 y BOM UTF-8.
- Q: ¿La columna `stock_inicial` debe ser obligatoria o puede venir vacía? → A: Opcional; vacío = 0 y no genera movimiento; el depósito solo se valida cuando el stock es mayor que cero.
- Q: ¿Cómo debe devolver la API los errores de importación? → A: 422 con el sobre estándar `{ error: { codigo: "importacion_invalida", mensaje, detalles: [{ fila, columna, mensaje }] } }`.
- Q: ¿Qué ocurre si falta `deposito`, no existe el indicado o no existe "Principal"? → A: Sin `deposito` se usa "Principal"; el depósito indicado inexistente es un error de fila `"El depósito 'X' no existe."`; si no hay "Principal" y no se indicó uno, es un error de fila `"No se encontró el depósito Principal."`; solo se valida cuando `stock_inicial` > 0 y NUNCA se crean depósitos.
- Q: ¿Dónde debe viajar el parámetro `modo`? → A: Query param `?modo=validar|aplicar`, con `validar` por defecto.

## Escenarios de Usuario y Pruebas *(obligatorio)*

### User Story 16 - Cargar el catálogo inicial desde una planilla (Prioridad: P3)

Como **Gerente**, quiero importar productos y su stock inicial desde un CSV, validando antes de
aplicar, para poner en marcha el sistema sin cargar cientos de productos a mano.

**Por qué esta prioridad**: no bloquea la operación diaria, pero es lo que decide si un negocio
real puede adoptar el sistema en una tarde.

**Escenario 16.1 — Validación previa (dry-run)**

- **Dado** un CSV con 500 filas,
- **Cuando** se envía con `modo=validar`,
- **Entonces** responde **200** con el resumen (filas leídas, válidas, con error) y la lista de
  errores por fila y columna, sin escribir nada.

**Escenario 16.2 — Aplicación atómica**

- **Dado** un CSV sin errores,
- **Cuando** se envía con `modo=aplicar`,
- **Entonces** en una única transacción se crean los productos, se crea el stock inicial como un
  movimiento `ENTRADA` (motivo `IMPORTACION`) y responde **201** con el resumen.

**Escenario 16.3 — Una fila inválida**

- **Dado** un CSV con una fila sin SKU,
- **Cuando** se aplica,
- **Entonces** responde **422** `importacion_invalida` con el detalle por fila y **no** se escribe
  ningún producto, ni siquiera los válidos.

**Escenario 16.4 — Duplicados**

- **Dado** un CSV con un SKU ya existente en la base o repetido dentro del archivo,
- **Cuando** se valida,
- **Entonces** esa fila queda marcada como error con
  `"El SKU 'YERBA-1K' ya existe."` o `"El SKU 'YERBA-1K' está repetido en el archivo (filas 3 y 8)."`.

**Escenario 16.5 — Categoría inexistente**

- **Dado** un CSV que referencia una categoría que no existe,
- **Cuando** se valida,
- **Entonces** la fila queda marcada con `"La categoría 'Almacén' no existe."`; la importación no
  crea categorías.

**Escenario 16.6 — Límites**

- **Dado** un archivo de más de 5 MB o más de 5.000 filas,
- **Cuando** se envía,
- **Entonces** responde **422** `importacion_invalida` con
  `"El archivo supera el límite de 5.000 filas o 5 MB."`.

**Escenario 16.7 — Operario**

- **Dado** una sesión `OPERACION`,
- **Cuando** intenta importar,
- **Entonces** responde **403** `sin_permisos`.

**Escenario 16.8 — Depósito inexistente**

- **Dado** un CSV con `stock_inicial` mayor que cero que referencia un depósito que no existe, o sin
  columna `deposito` cuando no hay un depósito "Principal",
- **Cuando** se valida,
- **Entonces** la fila queda marcada con `"El depósito 'X' no existe."` o
  `"No se encontró el depósito Principal."`, y no se crea ningún depósito automáticamente.

## Requisitos Funcionales

- **FR-063 Importación CSV.** El sistema DEBE aceptar un CSV UTF-8 con encabezado y las columnas
  `nombre`, `sku`, `categoria`, `unidad_medida`, `costo`, `precio_venta`, `stock_minimo`,
  `codigo_barras` (opcional), `stock_inicial` (opcional, vacío = 0) y `deposito` (opcional), en
  `modo=validar` o `modo=aplicar` recibido como query param (`?modo=`), con `validar` por defecto.
  DEBE detectar el delimitador `,` o `;` por el encabezado e interpretar el decimal como `.`
  (delimitador `,`) o `,` (delimitador `;`), tolerando comillas dobles y BOM UTF-8.
- **FR-064 Stock inicial.** En `modo=aplicar`, si `stock_inicial` es mayor que cero, el sistema DEBE
  generar un movimiento `ENTRADA` con motivo `IMPORTACION` en el depósito indicado o en el
  "Principal" por defecto, dentro de la misma transacción, actualizando `stock_por_deposito` y el
  cache `productos.stock_actual` del producto en esa misma transacción. Si `deposito` viene vacío se
  usa el depósito "Principal"; si el depósito indicado no existe, o no existe "Principal" cuando no
  se indicó, la fila es inválida. La importación NUNCA crea depósitos ni categorías.
- **FR-065 Límites, errores y permisos.** La importación DEBE ser atómica, devolver el detalle de
  errores por fila y columna en el sobre de error estándar `{ error: { codigo:
  "importacion_invalida", mensaje, detalles: [{ fila, columna, mensaje }] } }`, respetar los límites
  de 5.000 filas y 5 MB, y exigir `GESTION` (403 `sin_permisos`). En `modo=validar` con filas
  inválidas responde **200** con el resumen (`filas_leidas`, `validas`, `con_error`, `errores`) y no
  escribe nada; en `modo=aplicar` con cualquier error responde **422** y no escribe nada.

## Criterios de Éxito Medibles

- **SC-024** Un archivo de 5.000 filas válidas se aplica en menos de 60 segundos y deja exactamente
  5.000 productos y los movimientos de stock correspondientes; un archivo con errores no deja
  ningún producto nuevo.

## Supuestos

- El formato es CSV; no se soporta Excel (`.xlsx`) en esta spec. El dialecto se autodetecta según
  FR-063.
- Las categorías y los depósitos deben existir antes de importar; no se crean al vuelo.
- El stock inicial se imputa al depósito indicado o al "Principal" solo cuando es mayor que cero; no
  se reparte por ubicación.
- La importación no actualiza productos existentes: solo crea; un SKU existente es un error.
- Se acepta una dependencia nueva (`csv`) justificada por el parser; se evalúa antes de agregarla.

## Fuera de Alcance

- Actualización masiva de precios o stock de productos existentes.
- Importación de compras, ventas o clientes.
- Plantillas descargables y mapeo interactivo de columnas.
- Procesamiento asíncrono con cola y progreso en vivo.

## Dependencias y orden de implementación

1. 003 y 007 implementadas.
2. Sin migración propia: usa `productos`, `stock_por_deposito` y `movimientos`.
3. Puede reutilizar el transporte de correo para avisar el resultado (opcional, no obligatorio).
