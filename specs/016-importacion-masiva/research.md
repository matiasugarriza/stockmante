# Research — Importación masiva de productos y stock (016)

**Fecha**: 2026-11-07 | **Spec**: [spec.md](./spec.md) | **Plan**: [plan.md](./plan.md)

Decisiones técnicas que resuelven las ambigüedades de la spec y las brechas detectadas en el
análisis. Cada decisión indica fundamento y alternativas descartadas.

## D1. Dialecto CSV y parseo

- **Decisión**: leer el archivo como bytes, quitar el BOM UTF-8 si existe, tomar la primera línea y
  contar `;` y `,` fuera de comillas dobles. Si hay más `;`, el delimitador es `;` y el decimal es
  `,`; en cualquier otro caso el delimitador es `,` y el decimal es `.`. Comillas dobles RFC 4180.
  No se aceptan separadores de miles.
- **Rationale**: Excel en es-AR exporta `;` con decimal `,`; el estándar RFC 4180 usa `,` con `.`.
  Deducir el decimal del delimitador evita la ambigüedad de "1.234" (¿mil doscientos o uno con tres
  decimales?).
- **Alternativas**: exigir RFC 4180 (falla la migración real desde Excel es-AR); formato es-AR fijo
  (rechaza CSV estándar); selector en la UI (más superficie y más pruebas, sin beneficio claro).
- **Regla de importes**: `costo` y `precio_venta` se parsean con el separador decidido y se validan
  con `validar_monto` (≥ 0, ≤ 2 decimales, < 10^10). Si el valor contiene ambos separadores → error
  de columna.
- **Encabezado**: comparación `trim` + minúsculas; columnas desconocidas se ignoran. Faltante de una
  columna obligatoria → 422 global con `fila: 1` y `columna` = nombre faltante.
- **Filas**: `csv::ReaderBuilder::flexible(true)`; si una fila tiene más celdas que el encabezado →
  error de fila; celdas faltantes al final se tratan como vacías (solo válido para columnas
  opcionales).
- **Vacío**: archivo sin filas de datos → 422 `"El archivo no contiene filas para importar."`.

## D2. Atomicidad

- **Decisión**: `modo=aplicar` ejecuta **todo** el archivo dentro de una única transacción
  `pool.begin()`; cualquier error de fila aborta antes de abrir la transacción o hace `rollback`.
  Nunca se confirma una carga parcial.
- **Rationale**: FR-065, escenarios 16.2/16.3 y Principio V. La decisión anterior del plan
  ("fila por fila, no todo-o-nada") contradecía la spec y se elimina.
- **Alternativas**: carga parcial con reporte (viola FR-065 y deja datos a medias); transacción por
  fila (peor rendimiento y misma violación).

## D3. Contrato de error

- **Decisión**: nueva variante `AppError::ImportacionInvalida(Vec<DetalleImportacion>)` con
  `DetalleImportacion { fila: usize, columna: String, mensaje: String }` en `error.rs`. Status 422,
  código `importacion_invalida`. `IntoResponse` agrega `detalles` al sobre `{ error: { codigo,
  mensaje, detalles } }` solo para esa variante; `ErrorRespuesta`/`ErrorDetalle` de `docs.rs` se
  amplían con `detalles` opcional.
- **Rationale**: FR-065 y Principio III; el frontend necesita fila/columna para renderizar la tabla
  de errores. Mantiene el sobre estándar del proyecto.
- **Alternativas**: devolver el `ResumenImportacion` crudo (rompe el sobre y obliga a ramificar en el
  cliente); reutilizar `validacion` (pierde el código específico y el detalle estructurado).
- **Dry-run**: `modo=validar` con errores responde **200** con el resumen y `errores` (sin sobre de
  error); `modo=aplicar` con cualquier error responde **422** con `detalles`.

## D4. Validación reutilizando 003

- **Decisión**: cada celda se valida con las funciones de `models/producto.rs`:
  `validar_nombre_producto`, `validar_sku` (normaliza a mayúsculas), `validar_unidad` (UNIDAD, KG,
  METRO, CAJA), `validar_monto`, `validar_codigo_barras`, `validar_stock_minimo`. El `AppError` de
  cada validador se traduce a `DetalleImportacion` con la fila y la columna; no aborta el dry-run.
- **Rationale**: una sola fuente de reglas (P-II/P-IV) y mensajes coherentes con el alta individual.
- **Alternativas**: duplicar reglas en el importador (divergen con el tiempo); insertar y dejar que la
  BD falle (errores 500 y sin fila/columna).

## D5. Duplicados

- **Decisión**: prefetch de `SELECT sku FROM productos` y de códigos de barras existentes. SKU
  normalizado; si existe en la BD → `"El SKU 'X' ya existe."`; si se repite en el archivo →
  `"El SKU 'X' está repetido en el archivo (filas N y M)."` usando la primera fila vista. Igual
  criterio para `codigo_barras` (`"El código de barras 'X' ya existe."` / repetido en el archivo).
- **Rationale**: evita `23505` → 500 y cumple el mensaje exacto del escenario 16.4.
- **Alternativas**: confiar en la restricción UNIQUE (pierde la fila y el mensaje accionable).

## D6. Depósito y stock inicial

- **Decisión**: `stock_inicial` vacío = 0 y no genera movimiento. Si es > 0: `deposito` vacío usa el
  depósito activo llamado "Principal"; si el nombre indicado no existe o no hay "Principal" →
  `DetalleImportacion` de columna `deposito`. Nunca se crean depósitos.
- **Rationale**: clarificación 2026-11-07 y supuestos de la spec; evita fallar importaciones de
  catálogo sin stock por no tener "Principal".
- **Alternativas**: depósito obligatorio siempre (fricción innecesaria); crear depósito al vuelo
  (contradice la spec y puede duplicar depósitos por errores de tipeo).

## D7. Persistencia y performance

- **Decisión**: prefetch de categorías y depósitos en dos consultas; por fila válida `INSERT INTO
  productos ... RETURNING id`; si hay stock, upsert en `stock_por_deposito` con `ON CONFLICT ...
  DO NOTHING`, `SELECT ... FOR UPDATE`, `UPDATE` de cantidad, `INSERT` en `movimientos` +
  `movimiento_lineas` y `UPDATE productos SET stock_actual = ...` con el saldo resultante. Auditoría
  con `auditoria::registrar` en la misma transacción.
- **Rationale**: 5.000 filas < 60 s (SC-024); evita N+1 fuera de la transacción y mantiene el cache
  denormalizado consistente (P-V).
- **Alternativas**: `COPY` (más rápido pero no devuelve ids ni permite movimientos por fila);
  inserciones por lote (complejidad innecesaria para el volumen).

## D8. Límite de cuerpo HTTP

- **Decisión**: aplicar `DefaultBodyLimit::max(16 * 1024 * 1024)` a la ruta de importación para que
  el límite propio de 5 MB sea alcanzable y un archivo de 5 a 16 MB responda 422
  `importacion_invalida` (Axum limita el body a 2 MB por defecto).
- **Rationale**: sin esto, un archivo de 3–5 MB muere antes de la validación de negocio; con 16 MB,
  el caso "> 5 MB" del escenario 16.6 llega al chequeo y devuelve el mensaje exacto, no un 413.
- **Alternativas**: subir el límite global (afecta a todos los endpoints); streaming a disco
  (innecesario para 5 MB). Los archivos > 16 MB quedan rechazados por Axum; es un borde aceptado y
  documentado.

## D9. Permisos y auditoría

- **Decisión**: `familia_rol == "GESTION"`; `OPERACION` → 403 `sin_permisos`. Cada producto y cada
  movimiento creados se registran en auditoría dentro de la transacción (patrón existente).
- **Rationale**: FR-065 y matriz RBAC de 006.
- **Alternativas**: permiso a nivel de rol individual (la familia ya resuelve la matriz).

## D10. Frontend

- **Decisión**: `$lib/api/importaciones.ts` con `importarProductos(archivo, modo)` usando `FormData`
  y un helper `apiFormData` en `client.ts` (sin `Content-Type` manual); `ApiError` expone
  `detalles`. Página `/admin/importacion` con selección de archivo, "Validar", "Aplicar" (deshabilitado
  si hay errores), resumen y tabla de errores (fila, columna, mensaje). Enlace en el layout para
  `GESTION`.
- **Rationale**: Principio II (HTTP centralizado y tipado) y III (errores accionables).
- **Alternativas**: `fetch` dentro del componente (prohibido por P-II); subida automática al elegir
  archivo (impide el dry-run explícito).

## Verificación de esquema (P-V)

Contrastado contra `migrations/0002_categorias_productos.sql` y
`migrations/0009_stock_movimientos.sql`:

- `productos`: `sku VARCHAR(30) UNIQUE` con `CHECK` de mayúsculas/dígitos/guiones, `codigo_barras
  VARCHAR(64) UNIQUE NULLS NOT DISTINCT`, `costo`/`precio_venta NUMERIC(12,2) CHECK >= 0`,
  `stock_minimo >= 0`, `stock_actual >= 0` (agregado en 0009).
- `stock_por_deposito`: unicidad `(producto_id, deposito_id, ubicacion_id)` con `NULLS NOT
  DISTINCT`; `cantidad >= 0`.
- `movimientos`: `motivo` admite `IMPORTACION`; `movimiento_lineas.cantidad_delta <> 0` y
  `stock_resultante >= 0`.
- `categorias.nombre` y `depositos.nombre` se comparan por nombre (sin IDs en el CSV).

## Riesgos

- **Dialecto**: un CSV con comillas mal cerradas se reporta como archivo inválido; el mensaje debe
  indicar que se revise el formato.
- **Performance**: 5.000 filas con `FOR UPDATE` en una sola transacción pueden tardar; el test de
  SC-024 debe medir y fallar si supera 60 s.
- **Duplicados de código de barras**: se validan antes de insertar para no convertir un 23505 en 500.
