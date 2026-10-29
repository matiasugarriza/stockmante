# Data Model — Catálogo de productos y categorías

**Spec**: [spec.md](./spec.md) | **Plan**: [plan.md](./plan.md) | **Creado**: 2026-10-12 | **Replanteado**: 2026-10-25

> Tipos PostgreSQL; nombres en snake_case; fechas en `TIMESTAMPTZ`. Toda migración vive en
> `/migrations` (raíz). ANTES de cada migración o consulta: verificar el esquema real (constitución
> — Verificación de esquema).
>
> Las tablas se crearon en la fase foundational de 001 (migración `0002`); **la propiedad del
> esquema es de esta spec**. Esta versión reemplaza el modelo anterior: agrega
> `categorias.descripcion`, fija la forma final de `productos.codigo_barras` y elimina el borrado
> físico como vía de retiro. El DDL existente se cita como evidencia; **acá no se escribe el SQL de
> la migración nueva**.

## Diagrama de relaciones (texto)

```text
categorias 1────* productos
productos  1────* producto_ubicacion_permitida *────1 ubicaciones        (005)
productos  1────* movimientos / líneas de venta                          (futuro; sin spec)
```

> Toda tabla futura que referencie `productos.id` DEBE usar `ON DELETE RESTRICT`. Hoy la base no
> impide un `DELETE` directo sobre `productos`: la garantía de "nunca se borra" es de la
> aplicación (no existe la operación) y de esta spec, no de una restricción. La única tabla que hoy
> referencia `productos` es `producto_ubicacion_permitida` (005), con `ON DELETE CASCADE`; esa
> cascada solo se dispararía con SQL directo, porque la API no expone borrado (ver «Referencias y
> borrado»).

## categorias

| Campo | Tipo | Restricciones |
|-------|------|----------------|
| id | INT GENERATED ALWAYS AS IDENTITY | PK |
| nombre | VARCHAR(80) | NOT NULL, 1–80 caracteres, recortado en la app, único global sin distinguir mayúsculas |
| descripcion | VARCHAR(255) | NULL — **delta respecto de 0002**; vacío/blanco se normaliza a `NULL` |
| activo | BOOLEAN | NOT NULL DEFAULT TRUE |

Lista plana (spec, Supuestos). No tiene `creado_en` ni `actualizado_en` en `0002` y esta spec **no
los agrega**: la trazabilidad de altas, bajas y cambios sensibles la cubre FR-019 (auditoría de
006), que lleva su propio registro. La baja es siempre lógica; no existe borrado físico.

## productos

| Campo | Tipo | Restricciones |
|-------|------|----------------|
| id | INT GENERATED ALWAYS AS IDENTITY | PK |
| nombre | VARCHAR(160) | NOT NULL, 1–160 caracteres, recortado en la app |
| sku | VARCHAR(30) | NOT NULL, UNIQUE global, CHECK `^[A-Z0-9][A-Z0-9-]{2,29}$` |
| codigo_barras | VARCHAR(64) | NULL, UNIQUE (`NULLS DISTINCT`, forma final), 8–64 alfanuméricos |
| categoria_id | INT | NOT NULL, FK → `categorias(id)` ON DELETE RESTRICT |
| unidad_medida | VARCHAR(20) | NOT NULL; lista v1 `UNIDAD`, `KG`, `METRO`, `CAJA` (validada en la app) |
| costo | NUMERIC(12,2) | NOT NULL, CHECK `costo >= 0` |
| precio_venta | NUMERIC(12,2) | NOT NULL, CHECK `precio_venta >= 0` |
| stock_minimo | INT | NOT NULL DEFAULT 0, CHECK `stock_minimo >= 0` |
| activo | BOOLEAN | NOT NULL DEFAULT TRUE |
| creado_en / actualizado_en | TIMESTAMPTZ | NOT NULL DEFAULT now() |

> `productos.stock_actual` **no existe** en `0002`. La constitución lo describe como cache
> denormalizado del módulo de stock; si ese módulo lo agrega, lo hace en su propia migración y lo
> actualizan en la misma transacción que `stock_por_deposito`. Esta spec no lo introduce.

### Reglas

- **SKU**: se normaliza (trim + mayúsculas) antes de validar, comparar e insertar; unicidad global,
  incluso frente a productos inactivos.
- **Código de barras**: opcional; vacío o blanco → `NULL`; si se informa, 8–64 alfanuméricos y
  único entre todos los productos (activos o no). En la edición, `""` limpia el código (`NULL`) y
  omitir el campo lo conserva.
- **Montos**: `NUMERIC(12,2)`, ≥ 0, máximo 2 decimales y menores a `10^10` (`9 999 999 999,99` es
  el máximo representable). PostgreSQL **redondea** los decimales sobrantes al escribir
  `NUMERIC(12,2)`; por eso el rechazo de un tercer decimal es responsabilidad de la app, no del
  tipo.
- **Categoría**: obligatoria y existente; al crear o reasignar debe estar activa. La FK es
  `ON DELETE RESTRICT`, así que ni el SQL directo puede dejar productos huérfanos.
- **Borrado físico**: no existe ruta ni operación; `DELETE` responde 405. El retiro es
  `activo = FALSE`.
- **Baja lógica**: ver sección propia.
- **Precio y baja**: solo familia GESTION (FR-014); toda escritura del catálogo exige GESTION
  (FR-003).

## Baja lógica

**Definición**: `activo = FALSE` marca un registro retirado del catálogo operativo. No es un
borrado: la fila, su `id` y todas sus columnas se conservan; tampoco se archiva en otra tabla.

### Operaciones que bloquea

| Operación | Efecto |
|---|---|
| Listado y búsqueda estándar (`activo=true`, default) | El producto o la categoría no aparece |
| Búsqueda de caja (004, FR-013) | El producto inactivo no se ofrece para venta ni carga |
| Movimientos nuevos de entrada, salida, merma, ajuste o transferencia (FR-022) | 409 `producto_inactivo`; sin escritura de stock |
| Asignación de un producto a una categoría inactiva (crear o reasignar) | 422 `validacion` |
| Cualquier borrado físico | No existe la operación (405) |

### Operaciones que permite

| Operación | Efecto |
|---|---|
| `GET /{id}` (detalle) | 200 con todos los datos, incluso inactivo |
| Listado con `?activo=false` o `?activo=all` | Devuelve los inactivos |
| Edición de los demás campos del producto | 200; la baja no congela la corrección de datos |
| Reactivación (`POST …/reactivar`) | Vuelve a `activo = TRUE` |
| Reportes históricos y documentos ya emitidos | Siguen resolviendo el `id` y el SKU originales |
| Asignaciones de ubicación (005) | Se conservan intactas |

### Qué se conserva

- La fila completa: `id`, nombre, SKU, código de barras, `categoria_id`, unidad, costos, precios,
  `stock_minimo`, `creado_en` y `actualizado_en`.
- El SKU y el código de barras quedan **reservados**: no se liberan para otro producto.
- Las referencias existentes: asignaciones en `producto_ubicacion_permitida` (005) y los movimientos
  históricos que genere el módulo de stock (posterior a 003, aún sin spec).
- El nombre de la categoría queda reservado: otra categoría no puede reutilizarlo.

### Reactivación

- Permitida para productos y categorías, idempotente (204 si ya estaba activo), 404 si el `id` no
  existe y 403 si el actor es de la familia OPERACION.
- Reactivar un producto **no** reactiva su categoría ni al revés; no hay cascada en ningún sentido.
- No se valida el estado de la categoría al reactivar un producto: un producto cuya categoría fue
  desactivada puede volver a estar activo y conserva esa categoría.

### Búsquedas por defecto vs. histórico

| Consumidor | Filtro de estado | Resultado |
|---|---|---|
| Listado de gestión (003) | Sin parámetro → `activo = TRUE` | Solo activos |
| Listado de gestión con `?activo=false` / `?activo=all` | `FALSE` / sin filtro | Inactivos / todos |
| Búsqueda de caja (004) | Excluye inactivos por defecto; `incluir_inactivos` solo GESTION | Solo activos en operación |
| Movimientos nuevos (módulo de stock, futuro) | Exige `activo = TRUE` en la transacción | Rechazo 409 `producto_inactivo` |
| Reportes históricos (stock / auditoría 006) | Sin filtro de `activo` | Todos, con sus identificadores originales |

## Transiciones de estado

```text
[producto activo=true]  --POST /productos/{id}/desactivar-->  [activo=false]
[producto activo=false] --POST /productos/{id}/reactivar-->   [activo=true]

[categoría activa]      --POST /categorias/{id}/desactivar--> [inactiva]
[categoría inactiva]    --POST /categorias/{id}/reactivar-->  [activa]

desactivar/reactivar son idempotentes (204) y exigen familia GESTION.
La baja de una categoría NO cambia el estado de sus productos.
```

## Índices y restricciones

| Índice / restricción | Columnas | Origen | Propósito |
|---|---|---|---|
| `productos_sku_key` | `sku` UNIQUE | 0002 | Unicidad global del SKU (incluye inactivos) |
| `productos_codigo_barras_key` | `codigo_barras` UNIQUE, `NULLS DISTINCT` | 0002, corregido (delta 2) | Unicidad del código entre valores no nulos |
| `categorias_nombre_key` | `nombre` UNIQUE | 0002 | Guarda de duplicado exacto; se conserva por compatibilidad |
| `uq_categorias_nombre_normalizado` | `lower(btrim(nombre))` UNIQUE | delta 4 | Unicidad global sin distinguir mayúsculas ni espacios |
| `idx_productos_categoria` | `categoria_id` | delta 3 | Filtro por categoría y chequeo de la FK `RESTRICT` |

- **Sin índice sobre `activo`**: tiene dos valores y el volumen previsto es de miles de filas; no
  aporta selectividad suficiente para justificarlo (constitución I).
- **Sin índices de texto propios de 003**: ver el apartado siguiente.

### Filtrado por texto (nombre/SKU)

`q` se compara contra `nombre` (subcadena, sin distinguir mayúsculas) y contra `sku` (subcadena,
sobre el SKU ya normalizado a mayúsculas); ambos extremos se recortan y un `q` vacío equivale a no
filtrar.

- **003 no crea un índice de texto.** Con el volumen previsto (`plan.md`: miles de productos), el
  recorrido se acota con `idx_productos_categoria` y el filtro de estado, y se mantiene dentro de
  SC-012. Adelantar un GIN trigram sin medición contradice el Principio I.
- **El índice trigram pertenece a 004**: `FR-011` y `SC-002` lo justifican para la búsqueda de
  caja. Su migración está pendiente y **su número debe redefinirse** porque el `0005` que el ledger
  le reservaba ya está ocupado por `0005_sesion_unica.sql`. Cuando exista (GIN sobre `nombre`), la
  consulta de 003 lo aprovecha sin cambiar el contrato.
- **SKU**: la igualdad exacta y la unicidad usan `productos_sku_key`. El filtro por subcadena de
  SKU no tiene índice dedicado en v1: comparte el recorrido acotado descrito arriba.

## Validaciones de integridad (app + BD)

| Regla | Nivel |
|---|---|
| SKU normalizado (trim + mayúsculas) antes de validar, comparar y persistir | app (servicio) |
| SKU con formato `^[A-Z0-9][A-Z0-9-]{2,29}$` | BD CHECK + app (mensaje accionable) |
| SKU único global, incluidos los inactivos | UNIQUE + mapeo `23505` → `sku_duplicado` |
| Código de barras opcional; vacío/blanco → `NULL` | app (normalización) |
| Código de barras de 8–64 alfanuméricos | app (la columna acota a 64) |
| Código de barras único entre valores no nulos (también frente a inactivos) | UNIQUE (`NULLS DISTINCT`) + mapeo `23505` |
| Categoría existente al crear o reasignar | BD (FK) + app (mensaje distinto de "categoría inactiva") |
| Categoría activa al crear o reasignar | servicio (lee la categoría en la misma transacción) |
| Nombre de categoría de 1–80 caracteres, no vacío, recortado | app + longitud de columna |
| Nombre de categoría único sin distinguir mayúsculas ni espacios | UNIQUE (índice funcional) + app (`btrim`) |
| Descripción de categoría ≤ 255; vacío/blanco → `NULL` | app + longitud de columna |
| Montos ≥ 0 | BD CHECK |
| Montos con ≤ 2 decimales y < `10^10` | app (el tipo redondea, no rechaza) |
| `stock_minimo` entero ≥ 0 | BD CHECK + app |
| `unidad_medida` dentro de la lista v1 | app |
| `activo` solo se cambia por `/desactivar` y `/reactivar` | app (el `PUT` no acepta el campo: 422) |
| Sin borrado físico de productos ni categorías | app (no existe ruta) + prohibición de la spec |
| Producto inactivo sin movimientos nuevos | servicio del **módulo de stock** (futuro, sin spec), dentro de la transacción |

## Reglas de unicidad

| Regla | Alcance | Por qué | Dónde se garantiza |
|---|---|---|---|
| `productos.sku` | Global: activos **e inactivos** | No se recicla la identidad de un producto histórico; comprobantes, movimientos y reportes siguen apuntando a ese SKU | `productos_sku_key` + normalización en la app |
| `productos.codigo_barras` | Global entre valores no nulos, activos e inactivos | Mismo motivo; `NULL` es "sin código" y puede repetirse | UNIQUE `NULLS DISTINCT` |
| `categorias.nombre` | Global, sin distinguir mayúsculas, con extremos recortados, activas e inactivas | Evita dos categorías "Almacén"/"ALMACÉN" que confundan filtros y reportes históricos | Índice único funcional + `btrim` en la app |
| `categorias.nombre` (exacto) | Global, idéntico byte a byte | Compatibilidad con la restricción creada en `0002`; queda como guarda redundante | `categorias_nombre_key` |
| `productos.nombre` | **No único** | Dos productos pueden llamarse igual; la identidad la da el SKU | — (decisión explícita) |

## Paginación, orden y filtros

- `GET /api/productos?activo=&q=&categoria_id=&pagina=&por_pagina=`
  - `activo`: ausente o vacío → solo activos; `true` / `false` / `all`; otro valor → 422.
  - `q`: texto recortado; subcadena sobre `nombre` (sin distinguir mayúsculas) o sobre `sku`
    (normalizado a mayúsculas); vacío = sin filtro.
  - `categoria_id`: entero positivo; si no existe, resultado vacío (200), no 404.
  - `pagina >= 1`; `1 <= por_pagina <= 100` (default 20); fuera de rango → 422. `offset` se calcula
    con aritmética verificada para no desbordar.
  - `total` cuenta las filas que cumplen los filtros, antes de paginar.
  - Orden: `nombre ASC, id ASC` (estable y reproducible).
- `GET /api/categorias?activo=`: mismo default (solo activas), orden `nombre ASC`, sin paginar.

## Referencias y borrado

- `productos.categoria_id → categorias(id) ON DELETE RESTRICT`: una categoría con productos no se
  puede borrar ni por SQL directo. Como la API tampoco ofrece `DELETE`, la integridad es doble.
- `producto_ubicacion_permitida.producto_id → productos(id) ON DELETE CASCADE` (0003, propiedad de
  005): la cascada **no es alcanzable desde la API** porque 003 no expone borrado. Si 005 necesita
  una protección adicional ante SQL directo, cambia su propia FK en su spec.
- Tablas futuras de movimientos, ventas y auditoría: DEBEN referenciar `productos.id` con
  `ON DELETE RESTRICT` y consultar `activo` para decidir si admiten operaciones nuevas.
- No existe `DELETE /api/productos/{id}` ni `DELETE /api/categorias/{id}`: el método responde 405.
  La alternativa que se ofrece al usuario es la baja lógica, nunca el borrado.

## Delta de esquema respecto de 0002

`0002_categorias_productos.sql` es la fuente del esquema actual. Esta spec necesita cuatro cambios,
que se materializan en **una** migración nueva:

| # | Elemento | Estado en 0002 | Acción | Motivo |
|---|---|---|---|---|
| 1 | `categorias.descripcion` | No existe | Agregar `VARCHAR(255) NULL` | FR-010 (atributo del requerimiento) |
| 2 | `productos.codigo_barras` | `UNIQUE NULLS NOT DISTINCT` | Reemplazar por `UNIQUE (codigo_barras)` (`NULLS DISTINCT`) | Permitir varios productos sin código |
| 3 | `productos(categoria_id)` | Sin índice (el modelo anterior lo daba por existente: **error del documento, no de la base**) | Crear índice btree | Filtro por categoría y chequeo de la FK `RESTRICT` |
| 4 | `categorias.nombre` normalizado | Solo `UNIQUE (nombre)`, sensible a mayúsculas y espacios | Crear índice único funcional sobre `lower(btrim(nombre))` | Unicidad sin distinguir mayúsculas ni espacios (Clarifications) |

**Migración necesaria**: `0006_catalogo_descripcion_codigo_barras.sql`. Es el siguiente número
libre: `0004` no se reutiliza (la numeración global no se recicla) y `0005` ya está tomada por
`0005_sesion_unica.sql`, lo que deja desactualizadas las reservas de `0005` (004) y `0006` (006)
del ledger de [specs/README.md](../README.md). El nombre final del archivo y el orden de las
sentencias se definen en `plan.md`/`tasks.md`; acá no se escribe SQL.

### Drift conocido de la base local

El constraint de `productos.codigo_barras` ya está en la forma final (`UNIQUE (codigo_barras)`) en
`stockmante` y `stockmante_test`, aplicado **a mano** el 2026-10-25 y **sin fila en
`_sqlx_migrations`** (la `0004` del WIP nunca se publicó; ver [progress.md](./progress.md)).

- En una base creada desde cero, `0002` deja `NULLS NOT DISTINCT` y la migración nueva debe
  corregirlo.
- En las bases locales, la migración nueva vuelve a dejar el constraint en la misma forma. El
  reemplazo por nombre (`productos_codigo_barras_key`) es seguro **si se verifica antes el nombre
  real del constraint**; no se debe asumir que falta ni que el nombre cambió.
- `cargo sqlx prepare --check` **no detecta este drift** (es DDL, no una consulta): la verificación
  es contra el esquema real (`\d productos`, `sqlx-cli` o el MCP de PostgreSQL), como exige la
  constitución.
