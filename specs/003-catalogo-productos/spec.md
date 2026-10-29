# Especificación de Funcionalidad: Catálogo de productos y categorías

**Spec**: `003-catalogo-productos` | **Caso de uso**: US3 | **Prioridad**: P2

**Creado**: 2026-10-12 (dividida de `001-user-auth-catalog`) | **Replanteada**: 2026-10-25 | **Implementada**: 2026-10-26; verificada por la suite automatizada e incluida en el commit único de cierre del Módulo 1

**Versión 2 — supersede la v1.** Esta versión reemplaza por completo la anterior (estado
"Replanteo pendiente — WIP de backend deshecho") porque: (a) el requerimiento vigente amplía las
categorías a CRUD completo con descripción y estado; (b) prohíbe el borrado físico de productos
**y de categorías**; (c) agrega el filtrado por texto y por categoría al listado de gestión; y
(d) el WIP de backend se deshizo el 2026-10-25 (ver [progress.md](./progress.md)). La v1 queda en
el historial de git y **no se cita como fuente**; donde esta spec se aparta de ella, lo dice
explícitamente.

**Depende de**: [001-autenticacion-sesion](../001-autenticacion-sesion/spec.md) — identidad,
sesiones, `AppError` y guards RBAC (FR-003, FR-008a).

**Consumida por**: [004-busqueda-caja](../004-busqueda-caja/spec.md) (FR-011, FR-013),
[005-depositos-ubicaciones](../005-depositos-ubicaciones/spec.md) (asignación producto–ubicación)
y [006-rbac-auditoria](../006-rbac-auditoria/spec.md) (FR-019).

**Entrada** (requerimiento del usuario, verbatim — fuente de verdad del alcance):

> "CRUD completo de Productos y Categorías con soporte para Baja Lógica."
>
> "Los productos y categorías NUNCA deben eliminarse físicamente de la base de datos para
> preservar la integridad de los movimientos históricos futuros."
>
> "La baja lógica debe actualizar un campo de estado (ej. `is_active: false` o
> `deleted_at: timestamp`)."

El pedido completo agrega, además, las reglas de negocio 1–3 (gestión de categorías, gestión de
productos y mecanismo de baja lógica); esta spec las desarrolla en «Requisitos» y «Casos de Uso».

**Origen**: US3 de la especificación original del Módulo 1: "Como Administrador, quiero registrar
productos con datos comerciales (SKU, precio, costo, unidad, categoría, stock mínimo) y darlos de
baja lógica si ya tienen historial, para que el catálogo sea la fuente de verdad para caja y
almacén."

## Clarifications

### Session 2026-10-12

- Q: ¿Qué formato debe cumplir el SKU de un producto para ser considerado válido? → A: Mayúsculas,
  dígitos y guiones; 3–30 caracteres; se normaliza a mayúsculas antes de validar unicidad
  (ej. `SKU-001`, `POL-CAJA-10`).

### Session 2026-10-25 (replanteo)

- Q: ¿La baja lógica se modela con `deleted_at` o con un booleano `activo`? → A:
  `activo BOOLEAN NOT NULL DEFAULT TRUE`. Es la columna que ya existe en `categorias` y
  `productos` desde la migración `0002` y la misma semántica que usan 001 (sesiones) y 002
  (usuarios). Se descarta `deleted_at` porque: (1) obligaría a mantener dos señales para el mismo
  hecho —`activo` para la disponibilidad y `deleted_at` para el retiro— y a decidir cuál manda;
  (2) sumaría una migración de datos y de consumidores sin aportar nada que la auditoría de 006
  (FR-019: autor y timestamp) no cubra; (3) un timestamp anulable admite tres estados
  (`TRUE`, `FALSE`, `NULL`) donde el negocio solo necesita dos; y (4) las consultas de reporte
  histórico no necesitan la fecha de baja: la fila y el `id` se conservan siempre.
- Q: ¿El SKU de un producto dado de baja queda libre para otro producto? → A: No. El SKU es único
  en todo el sistema, **incluidos los productos inactivos**: no se recicla la identidad de un
  producto histórico porque comprobantes, movimientos y reportes seguirían apuntando a ese SKU.
- Q: ¿El código de barras también es único frente a productos inactivos? → A: Sí, único entre
  todos los productos que lo informan, activos o no. La diferencia con el SKU es que el código es
  opcional: `NULL` puede repetirse indefinidamente.
- Q: «Validación de nombres únicos por categoría»: ¿único por categoría, por producto o global?
  → A: **Global**: el `nombre` de una categoría es único en todo el sistema, comparado sin
  distinguir mayúsculas y con los espacios de los extremos recortados. Es un supuesto: el pedido
  no aclara el alcance, y "por categoría" no tendría sentido porque una categoría no contiene
  categorías. La comparación no pliega acentos («Almacén» y «Almacen» son nombres distintos).
- Q: ¿Se permite reactivar un producto o una categoría dados de baja? → A: Sí, ambos, con
  endpoints dedicados e idempotentes. El requerimiento pide "activar/desactivar" para categorías y
  "baja lógica" para productos; la reactivación es el camino de vuelta y evita que un error de
  baja obligue a crear un registro nuevo (lo que duplicaría SKU, que es único).
- Q: ¿Se permite el borrado físico? → A: No, ni de productos ni de categorías. La v1 permitía
  borrar una categoría sin productos y un producto sin referencias; el requerimiento vigente dice
  "NUNCA" para ambos y esa redacción prevalece. La única vía de retiro es `activo = FALSE`. La API
  no expone `DELETE`: responde 405 (método no soportado) y la alternativa de baja lógica se ofrece
  como acción de la interfaz (`POST …/desactivar`). Se prefiere 405 a un 409 permanente porque no
  existe ninguna condición en la que el borrado físico sea válido, y un endpoint que siempre falla
  confunde a cualquier cliente.
- Q: ¿Qué ocurre al desactivar una categoría que tiene productos? → A: La desactivación es
  **posible y no tiene cascada**: los productos conservan su `categoria_id` y su propio `activo`.
  La categoría deja de ofrecerse para asignaciones nuevas, pero los productos existentes siguen
  operando y editables. Bloquear la baja obligaría a vaciar la categoría primero, lo que
  distorsionaría el catálogo histórico.
- Q: ¿Se puede editar un producto cuya categoría quedó inactiva? → A: Sí, en todos sus campos
  salvo la categoría: conservar la categoría actual no exige que esté activa, pero **cambiarla**
  sí exige una categoría activa. Sin esta excepción, un producto quedaría congelado por una
  decisión tomada sobre su categoría.
- Q: ¿El estado se cambia con `PUT` o con endpoints dedicados? → A: Solo con
  `POST /{id}/desactivar` y `POST /{id}/reactivar`, igual que en 002. `PUT` no acepta `activo`
  (responde 422 si se envía) para que exista una sola vía de transición y quede auditable.
- Q: ¿`GET /api/productos` y `GET /api/categorias` sin el parámetro `activo` devuelven todo o solo
  lo activo? → A: **Solo lo activo.** El requerimiento exige que los productos dados de baja no
  aparezcan en las búsquedas estándar; `activo=all` habilita el histórico y `activo=false` los
  inactivos. Esto difiere de `GET /api/usuarios`, donde la ausencia de filtro devuelve todas: la
  semántica del listado de catálogo es propia y no se hereda.
- Q: ¿La unidad de medida es texto libre o una lista? → A: Lista cerrada en v1: `UNIDAD`, `KG`,
  `METRO`, `CAJA`. Se agrega `METRO` a las tres del contrato anterior porque el requerimiento lo
  nombra explícitamente ("unidades, kg, metros"). No hay conversión entre unidades en este
  módulo; la lista se valida en la aplicación (no hay `CHECK` en la base) para poder ampliarla sin
  migración.

## Escenarios de Usuario y Pruebas *(obligatorio)*

### User Story 3 - Catálogo de productos y categorías (Prioridad: P2)

Como **Administrador**, quiero administrar el catálogo maestro de categorías y productos —con
SKU, código de barras, unidad de medida, costo, precio de venta y stock mínimo— y retirarlos con
baja lógica, para que caja y almacén usen una única fuente de verdad sin perder el historial.

**Por qué esta prioridad**: Sin catálogo no hay ventas ni movimientos; es la base sobre la que se
construyen los módulos posteriores (004, 005 y el de stock). La baja lógica protege la integridad de
los movimientos históricos que generará el módulo de stock.

**Prueba independiente**: Se prueba creando una categoría, creando un producto asociado,
filtrándolo por texto y por categoría, y dándolo de baja; entrega valor administrativo sin
depender de depósitos ni de movimientos de stock.

**Escenarios de aceptación**:

1. **Dado** un Administrador autenticado, **Cuando** crea una categoría y luego un producto con
   SKU único, categoría, unidad, costo, precio y stock mínimo, **Entonces** ambos quedan activos y
   el producto es localizable por el listado de gestión.
2. **Dado** un catálogo con productos activos e inactivos, **Cuando** el Administrador filtra por
   texto y por categoría, **Entonces** obtiene solo los productos activos que cumplen ambos
   criterios, con un orden estable.
3. **Dado** un producto activo, **Cuando** el Administrador lo da de baja, **Entonces** deja de
   aparecer en el listado estándar, sigue siendo consultable por detalle y conserva su fila,
   su SKU y sus referencias.

Los criterios detallados, caso de uso por caso de uso, están en la sección siguiente.

---

### Casos de Uso

Cada caso de uso tiene un ID estable (`CU-<ÁREA>-<NN>`) y al menos un criterio de aceptación en
formato Dado/Cuando/Entonces. Los códigos HTTP y los `codigo` de error son parte del criterio: un
test que no los verifique no cubre el caso.

#### CU-CAT-01 — Crear categoría

Actor: usuario autenticado de la familia **GESTION**.

**Criterios de aceptación**:

1. **Dado** un usuario de gestión autenticado, **Cuando** envía `POST /api/categorias` con
   `{ "nombre": "Almacén", "descripcion": "Productos secos" }`, **Entonces** responde 201 con
   `{ id, nombre: "Almacén", descripcion: "Productos secos", activo: true }` y la categoría
   aparece en el listado por defecto.
2. **Dado** que existe la categoría "Almacén", **Cuando** se intenta crear "  almacén  ",
   **Entonces** responde 409 `categoria_duplicada` con mensaje accionable y el total de categorías
   no cambia.
3. **Dado** un nombre vacío o compuesto solo por espacios, **Cuando** se crea la categoría,
   **Entonces** responde 422 `validacion` indicando que el nombre es obligatorio y no se persiste
   ninguna fila.
4. **Dado** un nombre de 81 caracteres o una descripción de 256 caracteres, **Cuando** se crea la
   categoría, **Entonces** responde 422 `validacion` con el campo y el límite incumplidos, y no se
   persiste ninguna fila.
5. **Dado** que la descripción se omite o se envía vacía, **Cuando** se crea la categoría,
   **Entonces** queda con `descripcion: null` y el alta es válida (201).
6. **Dado** un usuario de la familia OPERACION autenticado, **Cuando** intenta crear una
   categoría, **Entonces** responde 403 `sin_permisos` y no se persiste ninguna fila.

#### CU-CAT-02 — Listar y filtrar categorías

Actor: cualquier usuario autenticado (lectura); el filtro de inactivas se usa desde gestión.

**Criterios de aceptación**:

1. **Dado** un catálogo con 3 categorías activas y 1 inactiva, **Cuando** se consulta
   `GET /api/categorias` sin parámetros, **Entonces** responde 200 con las 3 activas ordenadas por
   nombre ascendente y sin la inactiva.
2. **Dado** el mismo catálogo, **Cuando** se consulta `?activo=false`, **Entonces** devuelve solo
   la inactiva; con `?activo=all`, las 4; con `?activo=otro`, responde 422 `validacion`.
3. **Dado** un usuario de la familia OPERACION autenticado, **Cuando** lista categorías,
   **Entonces** responde 200 con las activas (la lectura del catálogo es parte de su alcance).
4. **Dado** el listado, **Cuando** se inspecciona cada ítem, **Entonces** incluye `id`, `nombre`,
   `descripcion` y `activo`, sin campos internos ni conteos derivados.

#### CU-CAT-03 — Ver detalle de categoría

Actor: cualquier usuario autenticado.

**Criterios de aceptación**:

1. **Dado** una categoría activa existente, **Cuando** se consulta
   `GET /api/categorias/{id}`, **Entonces** responde 200 con todos sus campos.
2. **Dado** una categoría inactiva, **Cuando** se consulta por `id`, **Entonces** responde 200:
   el detalle sigue disponible para consulta histórica.
3. **Dado** un `id` inexistente, **Cuando** se consulta el detalle, **Entonces** responde 404
   `recurso_no_encontrado`.
4. **Dado** un cliente sin sesión, **Cuando** consulta el detalle, **Entonces** responde 401
   `no_autenticado`.

#### CU-CAT-04 — Editar categoría

Actor: usuario autenticado de la familia **GESTION**.

**Criterios de aceptación**:

1. **Dado** una categoría activa, **Cuando** se envía `PUT /api/categorias/{id}` con
   `{ "nombre": "Almacén seco" }`, **Entonces** responde 200 con el nuevo nombre y conserva
   `descripcion` y `activo` sin cambios.
2. **Dado** que existe la categoría "Bebidas", **Cuando** se renombra otra a "  BEBIDAS ",
   **Entonces** responde 409 `categoria_duplicada` y el nombre original de la categoría editada no
   cambia.
3. **Dado** un `PUT` con cuerpo `{}`, **Cuando** se envía, **Entonces** responde 422 `validacion`
   indicando que no hay campos para actualizar y no se modifica la fila.
4. **Dado** un `PUT` con `{ "activo": false }`, **Cuando** se envía, **Entonces** responde 422
   `validacion` (el estado se cambia con `/desactivar`) y la categoría sigue activa.
5. **Dado** un usuario de la familia OPERACION autenticado, **Cuando** intenta editar,
   **Entonces** responde 403 `sin_permisos`.
6. **Dado** un `id` inexistente, **Cuando** se intenta editar, **Entonces** responde 404
   `recurso_no_encontrado`.
7. **Dado** una categoría con descripción, **Cuando** se envía `PUT` con `descripcion: ""`,
   **Entonces** la descripción queda en `null` y el nombre no cambia.

#### CU-CAT-05 — Desactivar categoría (baja lógica)

Actor: usuario autenticado de la familia **GESTION**.

**Criterios de aceptación**:

1. **Dado** una categoría activa sin productos, **Cuando** se envía
   `POST /api/categorias/{id}/desactivar`, **Entonces** responde 204; el listado por defecto la
   excluye y `?activo=all` la muestra con `activo: false`.
2. **Dado** una categoría con 2 productos activos, **Cuando** se desactiva, **Entonces** responde
   204 y los 2 productos conservan su `categoria` y su `activo: true` (no hay cascada); el detalle
   de la categoría sigue respondiendo 200.
3. **Dado** una categoría ya inactiva, **Cuando** se vuelve a desactivar, **Entonces** responde
   204 (operación idempotente) y no se produce error.
4. **Dado** un `id` inexistente, **Cuando** se intenta desactivar, **Entonces** responde 404
   `recurso_no_encontrado`.
5. **Dado** un usuario de la familia OPERACION autenticado, **Cuando** intenta desactivar,
   **Entonces** responde 403 `sin_permisos`.

#### CU-CAT-06 — Reactivar categoría

Actor: usuario autenticado de la familia **GESTION**.

**Criterios de aceptación**:

1. **Dado** una categoría inactiva, **Cuando** se envía `POST /api/categorias/{id}/reactivar`,
   **Entonces** responde 204 y la categoría vuelve al listado por defecto con `activo: true`.
2. **Dado** una categoría inactiva que tiene productos, **Cuando** se reactiva, **Entonces** los
   productos no cambian de estado: solo se reactiva la categoría.
3. **Dado** una categoría ya activa, **Cuando** se vuelve a reactivar, **Entonces** responde 204
   (operación idempotente).
4. **Dado** un `id` inexistente o un usuario de la familia OPERACION, **Cuando** se intenta
   reactivar, **Entonces** responde 404 `recurso_no_encontrado` o 403 `sin_permisos`,
   respectivamente.

#### CU-PROD-01 — Crear producto

Actor: usuario autenticado de la familia **GESTION**.

**Criterios de aceptación**:

1. **Dado** una categoría activa "Almacén", **Cuando** se envía `POST /api/productos` con
   `{ "nombre": "Yerba 1 kg", "sku": " yerba-1k ", "categoria_id": <id>, "unidad_medida":
   "UNIDAD", "costo": 1200.50, "precio_venta": 1800, "stock_minimo": 6 }`, **Entonces** responde
   201 con `sku: "YERBA-1K"` (normalizado), `codigo_barras: null`, `activo: true` y
   `categoria: { id, nombre: "Almacén" }`.
2. **Dado** un producto activo con SKU "SKU-001", **Cuando** se crea otro con "  sku-001  ",
   **Entonces** responde 409 `sku_duplicado` y el total de productos no cambia.
3. **Dado** un producto **inactivo** con SKU "SKU-002", **Cuando** se crea otro con "SKU-002",
   **Entonces** responde 409 `sku_duplicado`: la unicidad es global, no solo entre activos.
4. **Dado** un SKU `AB` (2 caracteres), `bad!`, `-ABC` (guion inicial) o de 31 caracteres,
   **Cuando** se crea el producto, **Entonces** responde 422 `validacion` con el formato esperado
   y no se persiste la fila.
5. **Dado** un producto sin `codigo_barras`, **Cuando** se crea, **Entonces** responde 201 con
   `codigo_barras: null`; un segundo producto sin código también responde 201 (varios productos
   pueden no tener código).
6. **Dado** un producto con código de barras "7791234567890", aunque esté inactivo, **Cuando** se
   crea otro con el mismo código, **Entonces** responde 409 `codigo_barras_duplicado` y el total
   de productos no cambia.
7. **Dado** un `categoria_id` inexistente, **Cuando** se crea el producto, **Entonces** responde
   422 `validacion` con el mensaje "La categoría indicada no existe" y no se persiste la fila.
8. **Dado** una categoría inactiva, **Cuando** se intenta crear un producto en ella, **Entonces**
   responde 422 `validacion` con un mensaje accionable ("La categoría está inactiva; activala o
   elegí otra") y no se persiste la fila.
9. **Dado** un costo o precio negativo, con 3 decimales o mayor o igual a `10 000 000 000`,
   **Cuando** se crea el producto, **Entonces** responde 422 `validacion` y no se persiste la
   fila.
10. **Dado** un `stock_minimo` negativo o con decimales, **Cuando** se crea el producto,
    **Entonces** responde 422 `validacion`.
11. **Dado** una `unidad_medida` fuera de `UNIDAD`, `KG`, `METRO` o `CAJA`, **Cuando** se crea el
    producto, **Entonces** responde 422 `validacion`.
12. **Dado** un nombre vacío o de 161 caracteres, **Cuando** se crea el producto, **Entonces**
    responde 422 `validacion` y no se persiste la fila.
13. **Dado** un usuario de la familia OPERACION autenticado, **Cuando** intenta crear un producto,
    **Entonces** responde 403 `sin_permisos`.
14. **Dado** dos altas concurrentes con el mismo SKU, **Cuando** ambas se envían, **Entonces**
    exactamente una responde 201 y la otra 409 `sku_duplicado`: la unicidad la impone la base, no
    un pre-chequeo.

#### CU-PROD-02 — Listar y filtrar productos

Actor: usuario autenticado de la familia **GESTION** (el listado completo es de gestión; la caja
usa la búsqueda de 004).

**Criterios de aceptación**:

1. **Dado** un catálogo con 3 productos activos y 1 inactivo, **Cuando** se consulta
   `GET /api/productos` sin parámetros, **Entonces** responde 200 con
   `{ items, total: 3, pagina, por_pagina }`, solo activos y ordenados por nombre ascendente con
   desempate por `id`.
2. **Dado** el mismo catálogo, **Cuando** se consulta `?activo=false`, **Entonces** devuelve solo
   el inactivo; con `?activo=all`, los 4; con `?activo=otro`, responde 422 `validacion`.
3. **Dado** `?q=yerba`, **Cuando** se lista, **Entonces** devuelve los productos cuyo nombre
   contiene "yerba" sin distinguir mayúsculas **o** cuyo SKU contiene "YERBA"; los espacios de los
   extremos se recortan y un `q` vacío equivale a no enviar el filtro.
4. **Dado** `?categoria_id=<id>`, **Cuando** se lista, **Entonces** devuelve solo los productos de
   esa categoría.
5. **Dado** `?q=yerba&categoria_id=<id>`, **Cuando** se lista, **Entonces** aplica ambos filtros
   en conjunto (AND), nunca uno u otro.
6. **Dado** `?categoria_id=<id inexistente>`, **Cuando** se lista, **Entonces** responde 200 con
   `items: []` y `total: 0` (es un filtro, no un recurso).
7. **Dado** 3 productos activos, **Cuando** se consulta `?pagina=2&por_pagina=2`, **Entonces**
   devuelve 1 ítem con `pagina: 2`; con `por_pagina=0`, `por_pagina=101` o `pagina=0`, responde
   422 `validacion`; sin parámetros, `por_pagina` es 20.
8. **Dado** un usuario de la familia OPERACION autenticado, **Cuando** consulta
   `GET /api/productos`, **Entonces** responde 403 `sin_permisos`.

#### CU-PROD-03 — Ver detalle de producto

Actor: cualquier usuario autenticado.

**Criterios de aceptación**:

1. **Dado** un producto activo, **Cuando** se consulta `GET /api/productos/{id}`, **Entonces**
   responde 200 con `nombre`, `sku`, `codigo_barras`, `categoria: { id, nombre }`,
   `unidad_medida`, `costo`, `precio_venta`, `stock_minimo` y `activo`.
2. **Dado** un producto inactivo, **Cuando** se consulta por `id`, **Entonces** responde 200: el
   detalle es la vía de consulta histórica de un producto retirado.
3. **Dado** un usuario de la familia OPERACION autenticado, **Cuando** consulta el detalle,
   **Entonces** responde 200 (lectura permitida).
4. **Dado** un cliente sin sesión o un `id` inexistente, **Cuando** consulta, **Entonces** responde
   401 `no_autenticado` o 404 `recurso_no_encontrado`, respectivamente.

#### CU-PROD-04 — Editar producto

Actor: usuario autenticado de la familia **GESTION**.

**Criterios de aceptación**:

1. **Dado** un producto existente, **Cuando** se envía `PUT /api/productos/{id}` con
   `{ "precio_venta": 2000 }`, **Entonces** responde 200 y conserva el resto de los campos sin
   cambios.
2. **Dado** otro producto con SKU "ABC-001", **Cuando** se cambia el SKU del producto editado a
   "  abc-001  ", **Entonces** responde 409 `sku_duplicado` y el SKU original no cambia.
3. **Dado** un producto inactivo con SKU "HIST-001", **Cuando** se intenta asignar ese SKU a un
   producto activo, **Entonces** responde 409 `sku_duplicado`.
4. **Dado** un producto con código de barras, **Cuando** se envía `codigo_barras: ""`, **Entonces**
   el código queda en `null`; cuando el campo se omite, el código se conserva; cuando se envía uno
   ya usado por otro producto (activo o inactivo), responde 409 `codigo_barras_duplicado`.
5. **Dado** un `PUT` con `{ "activo": false }`, **Cuando** se envía, **Entonces** responde 422
   `validacion` y el producto sigue activo (el estado se cambia con `/desactivar`).
6. **Dado** un `PUT` con cuerpo `{}`, **Cuando** se envía, **Entonces** responde 422 `validacion`
   y no se modifica la fila.
7. **Dado** un cambio de `categoria_id` hacia una categoría inactiva, **Cuando** se envía,
   **Entonces** responde 422 `validacion`; si el producto ya pertenece a una categoría inactiva y
   se editan otros campos sin enviar `categoria_id`, responde 200.
8. **Dado** un monto negativo, con 3 decimales o fuera de rango, **Cuando** se envía el `PUT`,
   **Entonces** responde 422 `validacion` y **ningún** campo del producto se modifica (la
   actualización es atómica).
9. **Dado** un producto inactivo, **Cuando** se editan sus datos (sin reactivarlo), **Entonces**
   responde 200: la baja lógica no congela la corrección de datos.
10. **Dado** un usuario de la familia OPERACION o un `id` inexistente, **Cuando** intenta editar,
    **Entonces** responde 403 `sin_permisos` o 404 `recurso_no_encontrado`, respectivamente.

#### CU-PROD-05 — Dar de baja un producto (baja lógica)

Actor: usuario autenticado de la familia **GESTION**.

**Criterios de aceptación**:

1. **Dado** un producto activo, **Cuando** se envía `POST /api/productos/{id}/desactivar`,
   **Entonces** responde 204; el listado por defecto ya no lo muestra y `?activo=all` lo devuelve
   con `activo: false`.
2. **Dado** el producto recién dado de baja, **Cuando** se consulta
   `GET /api/productos/{id}`, **Entonces** responde 200 con todos sus datos.
3. **Dado** un producto inactivo, **Cuando** el módulo de movimientos (posterior a 003, aún sin
   spec) intenta registrar una entrada, salida, merma, ajuste o transferencia, **Entonces** la
   operación se rechaza con 409
   `producto_inactivo` y no se escribe ningún movimiento ni se altera el stock; la verificación de
   `activo = TRUE` ocurre dentro de la misma transacción del movimiento.
4. **Dado** un producto inactivo con movimientos históricos, **Cuando** un reporte histórico lo
   consulta, **Entonces** el producto sigue apareciendo con su `id`, SKU y nombre: la baja no
   borra filas ni cambia identificadores.
5. **Dado** un producto ya inactivo, **Cuando** se vuelve a desactivar, **Entonces** responde 204
   (operación idempotente).
6. **Dado** un producto con asignaciones de ubicación (005), **Cuando** se da de baja, **Entonces**
   las asignaciones se conservan; la baja solo cambia `activo`.
7. **Dado** un usuario de la familia OPERACION o un `id` inexistente, **Cuando** intenta dar de
   baja, **Entonces** responde 403 `sin_permisos` o 404 `recurso_no_encontrado`, respectivamente.

#### CU-PROD-06 — Reactivar producto

Actor: usuario autenticado de la familia **GESTION**.

**Criterios de aceptación**:

1. **Dado** un producto inactivo, **Cuando** se envía `POST /api/productos/{id}/reactivar`,
   **Entonces** responde 204; el producto vuelve al listado por defecto y conserva SKU, código de
   barras, precios, stock mínimo y categoría.
2. **Dado** un producto inactivo cuya categoría también está inactiva, **Cuando** se reactiva,
   **Entonces** responde 204: la reactivación no valida el estado de la categoría (solo la
   asignación de una categoría nueva lo hace).
3. **Dado** un producto ya activo, **Cuando** se vuelve a reactivar, **Entonces** responde 204
   (operación idempotente).
4. **Dado** un usuario de la familia OPERACION o un `id` inexistente, **Cuando** intenta
   reactivar, **Entonces** responde 403 `sin_permisos` o 404 `recurso_no_encontrado`,
   respectivamente.

#### CU-RET-01 — Retiro sin borrado físico

Actor: usuario autenticado de la familia **GESTION**. Es el contrato que protege el historial.

**Criterios de aceptación**:

1. **Dado** un producto activo o inactivo, **Cuando** se invoca `DELETE /api/productos/{id}`,
   **Entonces** responde 405 (método no permitido) y la fila sigue existiendo con el mismo `id` y
   el mismo `activo`.
2. **Dado** una categoría activa o inactiva, **Cuando** se invoca `DELETE /api/categorias/{id}`,
   **Entonces** responde 405 y la fila sigue existiendo con el mismo `id` y el mismo `activo`.
3. **Dado** una categoría con productos, **Cuando** se intenta retirarla, **Entonces** la única
   vía es `activo = false`; los productos no se borran ni se reasignan.
4. **Dado** un catálogo con N productos y M categorías, **Cuando** se ejecutan 50 bajas lógicas,
   **Entonces** el conteo de filas sigue siendo N y M y solo cambiaron los valores de `activo`.
5. **Dado** un producto o una categoría dados de baja, **Cuando** se consulta el histórico con
   `?activo=all`, **Entonces** aparecen con sus identificadores originales, sin huecos en la
   numeración.

### Casos Borde

- **SKU mal formado**: `bad!` (carácter inválido), `AB` (2 caracteres), `-ABC` (guion inicial) o 31
  caracteres → 422 con el formato esperado. **`bad` no es un SKU inválido**: se normaliza a `BAD`,
  que cumple el regex; el caso inválido es `bad!`. La búsqueda por SKU no debe devolver ambigüedad
  (FR-011, 004).
- **Código de barras vacío vs. ausente**: `""` o solo espacios equivalen a "sin código" (`NULL`)
  al crear; en la edición, `""` limpia el código y omitir el campo lo conserva. Un código de 7
  caracteres, con caracteres no alfanuméricos o de más de 64 → 422. Un código ya usado, aunque sea
  por un producto inactivo → 409.
- **Categoría con productos**: no se puede borrar físicamente (no existe la operación: 405); se
  puede desactivar sin cascada. El detalle y el listado con `?activo=all` siguen mostrando su
  relación con los productos.
- **Producto con referencias**: no hay borrado físico; las asignaciones de ubicación (005) y los
  movimientos históricos se conservan aunque el producto esté inactivo.
- **Montos**: negativo, con más de 2 decimales o mayor o igual a `10 000 000 000` → 422, sin
  persistir ni modificar datos parcialmente.
- **`stock_minimo`**: negativo o con decimales → 422.
- **Asignación a una categoría inactiva**: al crear o cambiar de categoría → 422; conservar la
  categoría inactiva actual al editar otros campos → permitido.
- **Tildes en nombres de categoría**: la comparación de unicidad usa `lower` sin plegado de
  acentos, por lo que "Almacén" y "Almacen" son categorías distintas. Si el negocio necesitara
  tratarlas como iguales, requiere una decisión y una extensión de esquema posteriores.
- **Concurrencia**: dos altas simultáneas con el mismo SKU o el mismo código de barras → una
  responde 201 y la otra 409; la unicidad se resuelve por constraint, no por `SELECT` previo.
- **Nombres de producto repetidos**: dos productos pueden llamarse igual; la identidad la da el
  SKU. No se agrega unicidad de nombre de producto.

## Requisitos *(obligatorio)*

### Requisitos Funcionales

- **FR-009**: El sistema DEBE permitir CRUD de productos con: nombre, SKU (obligatorio, único y con
  formato mayúsculas/dígitos/guiones, 3–30 caracteres, normalizado a mayúsculas), código de barras
  (opcional y único si se informa), categoría, unidad de medida, costo, precio de venta y stock
  mínimo de seguridad.
- **FR-010**: El sistema DEBE permitir el CRUD completo de categorías con **nombre** (obligatorio,
  1–80 caracteres, único en todo el sistema sin distinguir mayúsculas ni espacios sobrantes),
  **descripción** (opcional, hasta 255 caracteres) y **estado activo/inactivo**; además DEBE
  permitir listarlas (con filtro por estado), consultar su detalle, editarlas y
  activarlas/desactivarlas. El retiro del catálogo se realiza con la baja lógica (FR-012), nunca con
  borrado físico. Las categorías son una lista plana, sin jerarquía, en v1.

  > **FR-010 reemplaza** la redacción anterior ("El sistema DEBE soportar categorías de productos
  > (al menos alta/listado básicos) para organización y filtrado"), que no cubría descripción,
  > estado, edición ni baja lógica. La ampliación proviene del requerimiento del 2026-10-25 y
  > conserva el mismo ID y dueño (003) para no duplicar la partición de `specs/README.md`.

- **FR-012**: El sistema **NO DEBE** ofrecer ni ejecutar el borrado físico de productos ni de
  categorías. La única forma de retirar un producto o una categoría del catálogo activo es la baja
  lógica (`activo = FALSE`), que conserva la fila, su `id` y todas sus referencias. `DELETE` sobre
  `/api/productos/{id}` o `/api/categorias/{id}` DEBE responder 405 y no modificar datos.

  > **FR-012 amplía** su alcance anterior, que solo bloqueaba el borrado de productos *con
  > historial* y permitía borrar categorías vacías. El requerimiento vigente dice "NUNCA" para
  > ambos.

- **FR-014**: El cambio de precio de venta y la baja de productos DEBE restringirse al rol
  Administrador/Gerente.

  > Toda escritura del catálogo (productos y categorías) exige la familia GESTION por FR-003;
  > FR-014 acota además las dos operaciones sensibles. La lectura (`GET /api/categorias`,
  > `GET /api/productos/{id}`) queda disponible para ambas familias.

- **FR-020**: El sistema DEBE listar el catálogo de productos de forma paginada y filtrable por:
  texto libre `q` (comparado contra el nombre como subcadena y contra el SKU, ambos sin distinguir
  mayúsculas, con los extremos recortados); `categoria_id`; y estado
  (`activo=true|false|all`, por defecto solo activos). Los filtros DEBEN ser combinables (AND) y el
  orden DEBE ser estable (nombre ascendente, desempate por `id`).

  > FR-011 (004) sigue siendo la búsqueda de caja por SKU, código de barras, nombre y categoría con
  > los tiempos de SC-002. FR-020 es el listado de gestión, con contrato y filtros propios.

- **FR-021**: El sistema DEBE permitir reactivar productos y categorías dados de baja
  (`POST …/reactivar`), devolviéndolos al catálogo activo sin alterar sus datos ni sus
  referencias. La reactivación DEBE ser idempotente y no DEBE reactivar en cascada otros
  registros.

  > **Supuesto**: el requerimiento pide "activar/desactivar" para categorías y solo "baja lógica"
  > para productos. La reactivación de productos se adopta como camino de vuelta (ver «Supuestos»)
  > porque un SKU es único y un error de baja no debe obligar a crear un producto nuevo con otro
  > SKU, lo que rompería la trazabilidad.

- **FR-022**: Ningún producto con `activo = FALSE` DEBE admitir movimientos nuevos de entrada,
  salida, merma, ajuste o transferencia. El módulo que registre el movimiento DEBE verificar el
  estado del producto **dentro de la misma transacción** que escribe el stock y rechazar la
  operación con 409 `producto_inactivo` antes de modificar cualquier saldo.

- **FR-023**: Un producto DEBE crearse o cambiar de categoría solo hacia una categoría activa.
  Conservar la categoría actual no exige que esté activa: un producto cuya categoría fue
  desactivada sigue siendo editable en los demás campos.

> **Numeración**: FR-001 a FR-008a pertenecen a 001/002; FR-011 y FR-013 a 004; FR-015 a FR-018 a
> 005; y FR-019 a 006. Por eso los requisitos nuevos de esta spec empiezan en FR-020 y no se
> reutiliza ningún ID.

### Contrato de inactividad para módulos consumidores

Este contrato lo hará cumplir el módulo de stock (posterior a 003, aún sin spec) y 004 lo consume en
su búsqueda por el estado del producto; **no se implementa en 003**.

| Consumidor | Regla | Resultado esperado |
|---|---|---|
| Movimientos (módulo de stock, futuro) | Verificar `productos.activo = TRUE` dentro de la transacción del movimiento | 409 `producto_inactivo`; sin escritura de stock |
| Búsqueda de caja (004) | Excluir `activo = FALSE` por defecto (FR-013) | El producto inactivo no se ofrece para venta ni carga |
| Gestión (003) | Listados y detalle sin filtro de estado | Producto inactivo visible para consulta y corrección |
| Reportes históricos (stock / auditoría 006) | Consultar sin filtro de `activo` | El producto inactivo sigue apareciendo con su `id` y SKU |
| Asignación de ubicaciones (005) | No exige producto activo para conservar asignaciones; la baja no las borra | Asignaciones intactas |

### Entidades Clave

- **Producto**: artículo comercializable; atributos: nombre, SKU (único global), código de barras
  (opcional, único si se informa), categoría (obligatoria, activa al asignar), unidad de medida,
  costo, precio de venta, stock mínimo de seguridad y estado activo/inactivo. Su identidad
  histórica es el `id` y el SKU; nunca se borra.
- **Categoría**: agrupación plana de productos para organización, filtrado y reportes; atributos:
  nombre (único global sin distinguir mayúsculas), descripción opcional y estado activo/inactivo.
  Nunca se borra.

## Criterios de Éxito

### Resultados Medibles

- **SC-006**: Cero SKUs duplicados y cero códigos de barras duplicados en el catálogo, **contando
  también los productos inactivos**.

  > La redacción anterior decía "entre productos activos"; se amplía porque FR-009 exige unicidad
  > global y un SKU histórico no debe reciclarse (ver Clarifications).

- **SC-009**: El 100% de los intentos de crear o renombrar una categoría con un nombre ya usado
  —aunque difiera en mayúsculas o espacios— se rechaza sin dejar filas nuevas ni cambios
  parciales.
- **SC-010**: Ninguna operación de la API reduce el número de filas de `productos` o `categorias`:
  después de 50 bajas lógicas, ambos conteos se mantienen y solo cambia el valor de `activo`.
- **SC-011**: El 100% de los productos dados de baja sigue accesible por
  `GET /api/productos/{id}` y por `GET /api/productos?activo=all` inmediatamente después de la
  baja, y ninguno aparece en el listado por defecto.
- **SC-012**: El 95% de las consultas de listado con filtro de texto y de categoría responde en
  1 segundo o menos con 5.000 productos cargados en el entorno de desarrollo.

## Supuestos

- **Unidad de medida**: lista cerrada `UNIDAD`, `KG`, `METRO`, `CAJA`; no hay conversión entre
  unidades en este módulo. Se valida en la aplicación para poder ampliarla sin migración.
- **Categorías**: lista plana; el árbol de categorías anidadas no es requisito de v1.
- **Descripción de categoría**: opcional; el requerimiento la nombra como atributo, no como
  obligatoria.
- **Stock mínimo**: atributo de producto para alertas posteriores; este módulo solo lo persiste y
  muestra, no genera alertas ni compara contra stock.
- **Reactivación**: permitida para productos y categorías (ver Clarifications). Un producto
  reactivado no recupera stock ni movimientos: solo vuelve a estar disponible.
- **Edición de inactivos**: un producto inactivo puede corregirse en sus datos; la baja no lo
  congela.
- **Categoría inactiva con productos**: se permite; no hay cascada ni bloqueo de la baja.
- **Paginación y orden**: `pagina >= 1`; `1 <= por_pagina <= 100` con default 20; listado de
  productos por `nombre ASC, id ASC`; listado de categorías por `nombre ASC` y sin paginar (su
  volumen es de decenas o cientos de filas).
- **Formatos**: nombre de producto 1–160 caracteres; nombre de categoría 1–80; descripción de
  categoría hasta 255; código de barras 8–64 alfanuméricos; montos con hasta 2 decimales y menores
  a `10 000 000 000`.
- **Idioma de UI**: español (es-AR por defecto), alineado con el resto del proyecto.

**Fuera de alcance global**: ver [specs/README.md](../README.md).

## Fuera de alcance

- **Movimientos de stock** (entradas, salidas, mermas, ajustes, transferencias) y ventas: esta spec
  define el contrato de inactividad que esos módulos deben cumplir, pero no los implementa.
- **Búsqueda de caja** (`GET /api/productos/buscar`, índice trigram, tiempos de SC-002) → 004.
- **Depósitos, ubicaciones jerárquicas y asignación producto–ubicación** → 005.
- **Verificación transversal de la matriz RBAC y auditoría** → 006. Acá solo se declaran las filas
  de permisos de los endpoints del catálogo.
- **Reportes históricos** como pantalla o consulta: acá solo se garantiza que las filas y los
  identificadores se conserven.
- **Jerarquía de categorías**, imágenes de producto, variantes, precios por lista, impuestos,
  descripción o atributos extra de producto, importación/exportación CSV y etiquetas.

## Trazabilidad

| Elemento | Estado | Detalle |
|---|---|---|
| FR-009 | Se conserva | CRUD de productos, formato y unicidad de SKU, código de barras opcional |
| FR-010 | **Reemplazado / ampliado** | De "alta/listado básicos" a CRUD completo con descripción, estado y unicidad global del nombre |
| FR-012 | **Ampliado** | De "bloquear borrado con historial" a "nunca borrado físico, para productos y categorías" |
| FR-014 | Se conserva | Precio y baja de productos solo GESTION |
| FR-020 a FR-023 | **Nuevos** | Filtros del listado, reactivación (FR-021, apoyada en supuesto), contrato de inactividad para movimientos y asignación a categoría activa |
| SC-006 | **Ampliado** | La unicidad se mide sobre todo el catálogo, incluidos los inactivos |
| SC-009 a SC-012 | **Nuevos** | Unicidad de categorías, no borrado, visibilidad del histórico y latencia del filtro |
| Escenarios de SKU y código de barras | Se conservan | Con la precisión de unicidad global frente a inactivos |

**Retirado respecto de la v1 y del contrato anterior** (los archivos `contracts/*.md`, `plan.md` y
`quickstart.md` deben alinearse; esta spec no los edita):

| Artefacto | Situación anterior | Situación vigente |
|---|---|---|
| `DELETE /api/productos/{id}` | 204 sin referencias; 409 `producto_con_referencias` | No existe: 405. El retiro es `POST …/desactivar` |
| `DELETE /api/categorias/{id}` | 204 sin productos; 409 `categoria_con_productos` | No existe: 405. El retiro es `POST …/desactivar` |
| Códigos `producto_con_referencias` / `categoria_con_productos` | Parte del contrato | Dejan de usarse en 003 |
| `PUT` con `activo` | Aceptado en categorías | Rechazado con 422; el estado cambia por endpoints dedicados |
| `GET /api/productos?activo` sin parámetro | "incluye inactivos" (ambiguo) | Default explícito: solo activos |
| Migración `0004_productos_codigo_barras_null_distinct.sql` | Referenciada por la v1 | No se reutiliza el número; la corrección va en la migración `0006` (ver [data-model.md](./data-model.md)) |

**Agregado al contrato** (debe reflejarse en `contracts/api.md` y `contracts/validation.md`):

- `GET /api/categorias/{id}` → 200 / 404 (detalle de categoría, hoy inexistente).
- `POST /api/categorias/{id}/reactivar` y `POST /api/productos/{id}/reactivar` → 204.
- `q` en `GET /api/productos`: texto sobre nombre (subcadena) y SKU (subcadena).
- `descripcion` en el ítem y en el alta/edición de categoría.
- `activo` por defecto `true` en `GET /api/productos` y `GET /api/categorias`.
- `METRO` en la lista de unidades de medida.
- 409 `producto_inactivo` en los movimientos (lo emitirá el módulo de stock, aún sin spec; 003
  define el contrato).

**Pendiente en `plan.md` / `tasks.md`** (fuera de esta entrega): rehacer la tabla de endpoints, la
estructura de servicios (CRUD de categorías con descripción, baja lógica sin `DELETE`), la
migración `0006`, la actualización de `contracts/api.md` y `contracts/validation.md`, el ledger de
migraciones de `specs/README.md` (que además tiene desactualizadas las reservas de `0005` y
`0006`) y el paso 6 de `quickstart.md` (V6a).

## Preguntas abiertas

Ninguna. Las decisiones de producto que estaban abiertas —borrado físico, alcance de la unicidad
de categorías, reactivación, categoría inactiva, valor por defecto del filtro de estado— quedaron
resueltas en «Clarifications» con el requerimiento y el esquema vigente. Las diferencias con
`plan.md`, `contracts/*.md`, `quickstart.md` y `specs/README.md` son de alineación documental, no
de producto, y están listadas en «Trazabilidad».
