# Los 10 casos de uso del TP

**Sistema**: Stockmante — gestión y control de inventario.
**Alcance**: esta es la lista de los **10 casos de uso funcionales** propuestos para el Trabajo
Práctico, con el estado real de implementación y la evidencia de cada uno.

> **Criterio de aprobación**: el sistema web (backend + frontend) debe resolver de manera
> interactiva y sin errores críticos **un mínimo de 7 de estos 10 casos de uso**. El estado
> actual es **9/10 completos y 1 parcial** (CU-09), por encima del mínimo.

## Resumen

| ID | Caso de uso | Actor | Specs | Pantalla / API | Estado |
|---|---|---|---|---|---|
| CU-01 | Registrar/Editar Productos en Catálogo, Ubicaciones y Depósitos | Administrador / Gerente | 003, 005 | `/admin/productos`, `/admin/categorias`, `/admin/depositos` | Completo |
| CU-02 | Registrar Entrada de Stock (Compra / Reabastecimiento) | Administrador / Gerente | 007, 012, 016 | `/movimientos`, `/admin/compras`, `/admin/importacion` | Completo |
| CU-03 | Configurar Alertas de Stock Mínimo | Administrador / Gerente | 003, 011 | `/admin/productos`, `/admin/stock`, `/alertas` | Completo |
| CU-04 | Ajustar Stock Manualmente (Auditoría / Mermas) | Administrador / Gerente | 007 | `/movimientos` | Completo |
| CU-05 | Consultar Valorización de Inventario y Reportes | Administrador / Gerente | 010, 014 | `/admin/reportes`, `/admin/stock` | Completo |
| CU-06 | Gestionar Usuarios y Asignar Roles | Administrador / Gerente | 001, 002 | `/admin/usuarios` | Completo |
| CU-07 | Registrar Movimientos de Stock | Operario / Cajero | 007 | `/movimientos` | Completo |
| CU-08 | Consultar Disponibilidad y Ubicación de Producto | Operario / Cajero | 004, 005, 010 | `/caja`, `/admin/stock` | Completo |
| CU-09 | Generar / Escanear Código de Barras o SKU | Operario / Cajero | 003, 004 | `/caja`, `/admin/productos` | **Parcial** |
| CU-10 | Notificación Automática de Inventario Crítico | Sistema | 011 | `/alertas` + correo | Completo |

**Resultado**: 9 completos + 1 parcial = **cumple el mínimo de 7/10**.

---

## CU-01 — Registrar/Editar Productos en Catálogo, Ubicaciones y Depósitos

**Actor**: Administrador / Gerente de Inventario.
**Objetivo**: dar de alta y mantener productos con sus especificaciones, precio, costo y stock
mínimo, junto con las ubicaciones y depósitos a los que se asignan.

**Flujo principal**

1. En `/admin/categorias` se crean las categorías del catálogo.
2. En `/admin/productos` se crea un producto con nombre, SKU, categoría, unidad de medida,
   costo, precio de venta y stock mínimo.
3. En `/admin/depositos` se crean depósitos y ubicaciones jerárquicas.
4. Desde la ficha del producto o de la ubicación se asignan las ubicaciones permitidas.
5. El listado de productos muestra las ubicaciones asignadas o «Sin ubicación».

**Flujos alternativos y errores**

- SKU duplicado → `409 sku_duplicado`; código de barras duplicado →
  `409 codigo_barras_duplicado`; categoría duplicada → `409 categoria_duplicada`.
- Categoría inactiva o inexistente → `422 validacion`.
- Montos negativos, con más de 2 decimales o fuera de rango → `422 validacion`.
- Depósito con ubicaciones o movimientos → `409 deposito_con_dependencias`; se ofrece baja
  lógica.
- Ubicación con depósito inexistente o padre de otro depósito → `422 ubicacion_huerfana`.

**Criterios de aceptación**

1. **Dado** un gestor autenticado, **Cuando** crea una categoría, un producto y un depósito con
   una ubicación, **Entonces** los tres quedan activos y visibles en sus listados.
2. **Dado** un producto con SKU repetido, **Cuando** se intenta crear otro, **Entonces** responde
   `409 sku_duplicado` sin cambios parciales.
3. **Dado** un producto activo, **Cuando** se da de baja, **Entonces** conserva su fila y su SKU,
   deja de aparecer en el listado estándar y sigue consultable por `id`.
4. **Dado** un producto, **Cuando** se le asignan ubicaciones permitidas, **Entonces** el listado
   las muestra en una sola lectura masiva y «Sin ubicación» cuando no tiene ninguna.

**Evidencia**

- Tests: `test_productos_http`, `test_categorias_http`, `test_catalogo_baja_logica`,
  `test_depositos_http`, `test_deposito_validacion`.
- Capturas: `captures/12-admin-productos.png`, `11-admin-categorias.png`,
  `13-admin-depositos.png`.
- Specs: [003-catalogo-productos](../specs/003-catalogo-productos/spec.md),
  [005-depositos-ubicaciones](../specs/005-depositos-ubicaciones/spec.md).

---

## CU-02 — Registrar Entrada de Stock (Compra / Reabastecimiento)

**Actor**: Administrador / Gerente de Inventario.
**Objetivo**: incrementar el stock existente o inicializarlo, asociándolo a un proveedor y a una
factura o remito, con un destino (depósito y ubicación).

**Flujo principal**

1. En `/admin/proveedores` se da de alta el proveedor.
2. En `/admin/compras` se crea una compra en borrador con `numero` (factura/remito), proveedor,
   depósito destino y líneas de producto con cantidad y costo.
3. Al confirmar, el backend escribe una única transacción: movimiento `ENTRADA` motivo `COMPRA`,
   stock por depósito, cache global, costo del producto y auditoría.
4. La UI muestra el movimiento y el stock resultante.
5. Alternativa masiva: `/admin/importacion` permite importar un CSV con stock inicial, que genera
   un movimiento `ENTRADA` motivo `IMPORTACION`.

**Flujos alternativos y errores**

- Compra ya confirmada → `409 compra_ya_confirmada`; compra no editable → `409 compra_no_editable`.
- Proveedor inactivo → `409 proveedor_inactivo`.
- Anulación sin stock suficiente → `409 stock_insuficiente`; no cambia la compra.
- Importación con una fila inválida → `422 importacion_invalida` con el detalle por celda; no se
  crea nada.
- Producto inactivo o depósito inválido → `422 validacion` / `409 producto_inactivo`.

**Criterios de aceptación**

1. **Dado** un proveedor activo y un producto, **Cuando** se confirma una compra, **Entonces** se
   crea una `ENTRADA` con motivo `COMPRA`, sube el stock y queda la auditoría.
2. **Dado** una compra confirmada, **Cuando** se intenta confirmar de nuevo, **Entonces** responde
   `409 compra_ya_confirmada` sin duplicar stock.
3. **Dado** una compra confirmada, **Cuando** se anula, **Entonces** se revierte el stock con una
   `SALIDA` motivo `DEVOLUCION` y queda auditada.
4. **Dado** un CSV válido, **Cuando** se importa en modo aplicar, **Entonces** se crean productos,
   stock inicial y movimiento `ENTRADA` motivo `IMPORTACION` en una transacción; con un error no
   se crea nada.

**Evidencia**

- Tests: `test_compras_http`, `test_compras_concurrencia`, `test_movimientos_http`,
  `test_importacion_http`, `test_importacion_validacion`.
- Capturas: `captures/15-admin-proveedores.png`, `16-admin-compras.png`.
- Specs: [007-movimientos-stock](../specs/007-movimientos-stock/spec.md),
  [012-proveedores-compras](../specs/012-proveedores-compras/spec.md),
  [016-importacion-masiva](../specs/016-importacion-masiva/spec.md).

---

## CU-03 — Configurar Alertas de Stock Mínimo

**Actor**: Administrador / Gerente de Inventario.
**Objetivo**: establecer umbrales de reabastecimiento para que el sistema avise cuando el stock
caiga por debajo del límite.

**Flujo principal**

1. En el alta/edición de producto se define `stock_minimo`.
2. En `/admin/stock` se filtran los productos bajo mínimo y se ve el `faltante`.
3. El sistema evalúa periódicamente (scheduler cada `ALERTAS_INTERVALO_MINUTOS`, por defecto 60)
   y también bajo demanda con `POST /api/alertas/enviar`.
4. Envía un correo por gestor activo con sus productos pendientes del día y registra cada par
   producto–gestor para no duplicar el aviso.

**Flujos alternativos y errores**

- `stock_minimo` negativo o no entero → `422 validacion`.
- Sin productos bajo mínimo → no se envía correo y la respuesta informa `productos: 0`.
- Falla de transporte SMTP → responde `200` con `fallidos > 0` y se reintenta en el ciclo
  siguiente.
- `OPERACION` en alertas → `403 sin_permisos`.

**Criterios de aceptación**

1. **Dado** un producto con `stock_minimo = 10` y stock 4, **Cuando** se consulta `/api/alertas`,
   **Entonces** aparece con `faltante = 6`.
2. **Dado** un producto bajo mínimo, **Cuando** corre el ciclo, **Entonces** se envía un solo
   correo por gestor y producto en el día local.
3. **Dado** el mismo producto y el mismo día, **Cuando** corre el ciclo otra vez, **Entonces** no
   se duplica el correo.
4. **Dado** un usuario `OPERACION`, **Cuando** consulta alertas, **Entonces** recibe
   `403 sin_permisos`.

**Evidencia**

- Tests: `test_alertas_http`, `test_alertas_validacion`, `test_stock_http`.
- Captura: `captures/14-admin-stock.png`.
- Specs: [003-catalogo-productos](../specs/003-catalogo-productos/spec.md),
  [011-alertas-stock-minimo](../specs/011-alertas-stock-minimo/spec.md).

---

## CU-04 — Ajustar Stock Manualmente (Auditoría / Mermas)

**Actor**: Administrador / Gerente de Inventario.
**Objetivo**: corregir discrepancias entre el stock físico y el digital por rotura, robo o error
de conteo, dejando una justificación obligatoria.

**Flujo principal**

1. En `/movimientos` el gestor elige tipo `AJUSTE` (o `MERMA`), depósito, motivo, cantidad y una
   **observación obligatoria** que justifica la corrección.
2. El backend valida permisos (`AJUSTE` solo `GESTION`), motivo, justificación, cantidades y
   stock.
3. En una transacción escribe el movimiento, actualiza `stock_por_deposito`, el cache global y la
   auditoría.
4. La UI muestra el stock resultante.

**Flujos alternativos y errores**

- `AJUSTE` sin observación → `422 validacion` (“Un ajuste requiere una justificación…”).
- Motivo `OTRO` sin observación → `422 validacion`.
- Ajuste negativo mayor al stock → `409 stock_insuficiente`.
- `OPERACION` intentando `AJUSTE` → `403 sin_permisos`.
- Tipo ↔ motivo incoherente → `422 validacion` con los motivos válidos.

**Criterios de aceptación**

1. **Dado** un producto con stock 10, **Cuando** `GESTION` registra un `AJUSTE` con justificación
   y delta −3, **Entonces** el stock queda en 7 y la auditoría registra el autor.
2. **Dado** un `AJUSTE` sin observación, **Cuando** se envía, **Entonces** responde
   `422 validacion` y no se escribe nada.
3. **Dado** una `MERMA` con motivo `OTRO`, **Cuando** se envía sin observación, **Entonces**
   responde `422 validacion`.
4. **Dado** un usuario `OPERACION`, **Cuando** intenta un `AJUSTE`, **Entonces** recibe
   `403 sin_permisos`.

**Evidencia**

- Tests: `test_movimientos_http`, `test_movimiento_validacion`, `test_rbac_matriz`.
- Captura: `captures/06-movimientos.png`.
- Spec: [007-movimientos-stock](../specs/007-movimientos-stock/spec.md).

---

## CU-05 — Consultar Valorización de Inventario y Reportes

**Actor**: Administrador / Gerente de Inventario.
**Objetivo**: ver métricas financieras del inventario (capital invertido, valorización, rotación
y productos de baja rotación) y reportes de ventas.

**Flujo principal**

1. En `/admin/reportes` se eligen kardex, stock valorizado, rotación o ventas.
2. Se filtran por producto, depósito y rango de fechas.
3. El backend calcula con `stock_por_deposito`, movimientos y ventas confirmadas.
4. Se puede descargar el resultado en CSV (RFC 4180, BOM, `\r\n`) o verlo en JSON.

**Flujos alternativos y errores**

- Fechas faltantes o invertidas → `422 validacion` con el parámetro.
- `formato` distinto de `json`/`csv` → `422 validacion`.
- Producto inexistente en kardex → `404 recurso_no_encontrado`.
- `OPERACION` → `403 sin_permisos` (también en CSV).

**Criterios de aceptación**

1. **Dado** stock y movimientos, **Cuando** `GESTION` consulta el kardex, **Entonces** el saldo
   corrido reconstruye el stock actual.
2. **Dado** el reporte de stock valorizado, **Cuando** se consulta, **Entonces** suma el stock
   físico por costo unitario.
3. **Dado** un rango de fechas, **Cuando** se consulta rotación, **Entonces** cuenta solo
   `SALIDA` sin `TRANSFERENCIA` y calcula el índice.
4. **Dado** el reporte de ventas, **Cuando** se pide CSV, **Entonces** descarga un CSV con el
   `Content-Disposition` correcto.

**Evidencia**

- Tests: `test_reportes_http`, `test_reportes_calculo`, `test_stock_http`.
- Capturas: `captures/14-admin-stock.png` y `/admin/reportes` (captura a generar).
- Specs: [010-consulta-stock](../specs/010-consulta-stock/spec.md),
  [014-reportes-kardex](../specs/014-reportes-kardex/spec.md).

---

## CU-06 — Gestionar Usuarios y Asignar Roles

**Actor**: Administrador / Gerente de Inventario.
**Objetivo**: crear cuentas para empleados y definir sus permisos dentro del sistema.

**Flujo principal**

1. En `/admin/usuarios` el gestor lista, filtra y pagina usuarios.
2. Crea un usuario con nombre, correo y rol (`ADMIN`, `GERENTE`, `OPERARIO`, `CAJERO`).
3. El sistema muestra en pantalla las credenciales (correo + contraseña temporal).
4. El usuario cambia la contraseña en el primer login.
5. El gestor puede editar, desactivar, reactivar y regenerar la temporal.

**Flujos alternativos y errores**

- Correo duplicado → `409 email_duplicado`; formato inválido → `422 validacion`.
- Último gestor activo → `409 ultimo_administrador`.
- Usuario desactivado → no puede iniciar sesión; sus sesiones se revocan.
- `OPERACION` en la gestión → `403 sin_permisos`.

**Criterios de aceptación**

1. **Dado** un gestor, **Cuando** crea un usuario válido, **Entonces** recibe `201` con las
   credenciales y el usuario puede entrar con la temporal.
2. **Dado** un correo repetido, **Cuando** se crea otro usuario, **Entonces** responde
   `409 email_duplicado`.
3. **Dado** el último gestor activo, **Cuando** se intenta desactivar, **Entonces** responde
   `409 ultimo_administrador`.
4. **Dado** un usuario `OPERACION`, **Cuando** abre `/admin/usuarios`, **Entonces** el guard lo
   redirige y la API responde `403 sin_permisos`.

**Evidencia**

- Tests: `test_usuarios_http`, `test_usuarios_alta`, `test_usuarios_email`,
  `test_ultimo_admin`, `test_auth_http`.
- Captura: `captures/10-admin-usuarios.png`.
- Specs: [001-autenticacion-sesion](../specs/001-autenticacion-sesion/spec.md),
  [002-gestion-usuarios](../specs/002-gestion-usuarios/spec.md).

---

## CU-07 — Registrar Movimientos de Stock

**Actor**: Operario / Cajero.
**Objetivo**: confirmar el stock inicial registrado por el administrador y descontar o incrementar
unidades de forma atómica al procesar un ingreso o egreso de un depósito.

**Flujo principal**

1. En `/movimientos` el operador elige tipo (`ENTRADA`, `SALIDA` o `MERMA`), depósito, motivo y
   líneas con producto, ubicación opcional y cantidad.
2. El backend valida el tipo, el motivo, las cantidades enteras y el stock disponible.
3. En una transacción actualiza `stock_por_deposito`, el cache global y la auditoría.
4. La UI muestra el resultado concreto (producto, depósito, cantidad y stock resultante).

**Flujos alternativos y errores**

- Salida o merma sin stock → `409 stock_insuficiente`; no se escribe nada.
- Producto inactivo → `409 producto_inactivo`.
- Cantidad no entera, cero o fuera de rango → `422 validacion`.
- Motivo incoherente o reservado → `422 validacion` con la lista de motivos válidos.
- Movimiento multilínea con una línea inválida → no se escribe ninguna.

**Criterios de aceptación**

1. **Dado** un producto con stock, **Cuando** el operario registra una `ENTRADA`, **Entonces** el
   stock por depósito y el cache suben y la UI muestra el saldo.
2. **Dado** un producto sin stock suficiente, **Cuando** registra una `SALIDA`, **Entonces**
   responde `409 stock_insuficiente` y el stock no cambia.
3. **Dado** un movimiento con una línea inválida, **Cuando** se envía, **Entonces** no se escribe
   cabecera, línea, auditoría ni cambio de stock.
4. **Dado** un usuario `OPERACION`, **Cuando** intenta un `AJUSTE`, **Entonces** recibe
   `403 sin_permisos` (el ajuste es del CU-04).

**Evidencia**

- Tests: `test_movimientos_http`, `test_movimiento_validacion`, `test_rbac_matriz`.
- Captura: `captures/06-movimientos.png`.
- Spec: [007-movimientos-stock](../specs/007-movimientos-stock/spec.md).

---

## CU-08 — Consultar Disponibilidad y Ubicación de Producto

**Actor**: Operario / Cajero.
**Objetivo**: buscar rápido por SKU, nombre o categoría y verificar la cantidad disponible en
tiempo real y dónde está.

**Flujo principal**

1. En `/caja` el operador elige el depósito y escribe SKU, código, nombre o categoría.
2. El frontend aplica debounce y llama a `GET /api/productos/buscar`.
3. El backend prioriza coincidencias exactas de SKU/código y luego similitud de nombre.
4. Para los resultados visibles, el frontend consulta `GET /api/stock` con el depósito elegido y
   muestra el stock real.
5. En `/admin/productos` se ven las ubicaciones asignadas a cada producto; en `/admin/stock`
   (GESTION) se ve el desglose por depósito.

**Flujos alternativos y errores**

- Texto de menos de 2 caracteres para nombre/categoría → `422 validacion`.
- Productos inactivos: excluidos por defecto; `incluir_inactivos=true` exige `GESTION`.
- Si hay más resultados que el límite, la UI avisa «Se muestran los primeros N resultados…».
- Error al cargar depósitos o stock → aviso visible y la búsqueda sigue funcionando.

**Criterios de aceptación**

1. **Dado** un producto activo con stock en el depósito elegido, **Cuando** se busca por SKU,
   **Entonces** aparece con la cantidad disponible.
2. **Dado** un producto inactivo, **Cuando** se busca sin `incluir_inactivos`, **Entonces** no
   aparece.
3. **Dado** un término de nombre o categoría, **Cuando** se busca, **Entonces** se ordena por
   similitud y se respeta el límite.
4. **Dado** un producto con ubicaciones asignadas, **Cuando** se abre el catálogo, **Entonces** se
   muestran sus ubicaciones o «Sin ubicación».

**Evidencia**

- Tests: `test_busqueda_http`, `test_busqueda`, `test_stock_http`, `busquedaCaja.test.ts`,
  `inventario.test.ts`.
- Captura: `captures/05-caja.png`.
- Specs: [004-busqueda-caja](../specs/004-busqueda-caja/spec.md),
  [005-depositos-ubicaciones](../specs/005-depositos-ubicaciones/spec.md),
  [010-consulta-stock](../specs/010-consulta-stock/spec.md).

---

## CU-09 — Generar / Escanear Código de Barras o SKU

**Actor**: Operario / Cajero.
**Objetivo**: agilizar la lectura e identificación de productos en el almacén usando un lector de
código de barras o la cámara del dispositivo.

**Estado: parcial.** La aplicación **sí** almacena un `codigo_barras` único por producto (8–64
caracteres alfanuméricos), lo muestra en el catálogo y lo busca por igualdad exacta desde el
primer carácter en `/caja`. Un **lector físico de código de barras** que emula teclado funciona
sin cambios: el operador escanea y el código se escribe en el campo de búsqueda. Lo que **no**
está implementado es la lectura con **cámara** ni la **generación/impresión de etiquetas**; la
spec 019 lo excluye explícitamente (la búsqueda es por texto o selección).

**Flujo principal (soportado)**

1. En `/caja` el operador pone el foco en el buscador y escanea con el lector físico (o pega el
   código).
2. `GET /api/productos/buscar?campo=codigo&q=<código>` devuelve el producto exacto.
3. El operador lo agrega a la operación.

**Flujos alternativos y errores**

- Código inexistente → lista vacía; la UI muestra «sin resultados».
- Código con formato inválido al crear un producto → `422 validacion` (8–64 alfanuméricos).
- Código repetido → `409 codigo_barras_duplicado`.
- Cámara no disponible o no implementada → usar lector físico o entrada manual.

**Criterios de aceptación**

1. **Dado** un producto con `codigo_barras`, **Cuando** se busca por ese código exacto,
   **Entonces** aparece el producto.
2. **Dado** un lector de código de barras conectado que emula teclado, **Cuando** escanea en el
   buscador de `/caja`, **Entonces** el código se ingresa y la búsqueda se dispara.
3. **Dado** un código repetido, **Cuando** se crea otro producto, **Entonces** responde
   `409 codigo_barras_duplicado`.
4. **Dado** un código inválido, **Cuando** se guarda el producto, **Entonces** responde
   `422 validacion`.
5. **Dado** un navegador sin cámara o sin permisos, **Cuando** se intenta escanear con cámara,
   **Entonces** la aplicación no lo ofrece y sugiere el lector físico o la entrada manual
   (funcionalidad no incluida).

**Evidencia**

- Tests: `test_busqueda_http` (campo `codigo`), `test_productos_http` (unicidad y validación),
  `test_producto_validacion`.
- Captura: `captures/05-caja.png`, `captures/12-admin-productos.png`.
- Specs: [003-catalogo-productos](../specs/003-catalogo-productos/spec.md),
  [004-busqueda-caja](../specs/004-busqueda-caja/spec.md).
- **Spec de la mejora**: [020-escaneo-codigos-barra](../specs/020-escaneo-codigos-barra/spec.md)
  (especificada 2026-11-07; la implementación de cámara/etiquetas queda delegada a otro agente).
  No forma parte del mínimo de 7/10.

---

## CU-10 — Notificación Automática de Inventario Crítico

**Actor**: Sistema (proceso automatizado).
**Objetivo**: evaluar periódicamente si algún producto alcanzó su límite inferior y enviar una
alerta por correo o dashboard.

**Flujo principal**

1. Cada `ALERTAS_INTERVALO_MINUTOS` (default 60; `0` apaga el scheduler) el backend busca
   productos activos con `stock_minimo > 0` y `stock_actual < stock_minimo`.
2. Para cada gestor activo arma el digest de productos pendientes del día local (UTC−03:00).
3. Envía un correo por gestor, registra la deduplicación en `notificaciones_stock` y deja el
   listado disponible en `GET /api/alertas`.
4. También se puede disparar a mano con `POST /api/alertas/enviar`.

**Flujos alternativos y errores**

- Sin productos bajo mínimo → no se envía correo (`productos: 0`).
- Falla SMTP → `fallidos > 0` y reintento en el ciclo siguiente.
- Sin `SMTP_URL` en desarrollo → stub de consola; en producción el arranque exige SMTP.
- `OPERACION` → `403 sin_permisos`.

**Criterios de aceptación**

1. **Dado** un producto bajo mínimo, **Cuando** corre el ciclo, **Entonces** se envía un correo
   por gestor con el producto y su faltante.
2. **Dado** el mismo producto y gestor en el mismo día, **Cuando** corre el ciclo de nuevo,
   **Entonces** no se duplica el correo.
3. **Dado** un fallo de transporte, **Cuando** corre el ciclo, **Entonces** la respuesta informa
   `fallidos > 0` y el próximo ciclo reintenta.
4. **Dado** un usuario `OPERACION`, **Cuando** consulta `/api/alertas`, **Entonces** recibe
   `403 sin_permisos`.

**Evidencia**

- Tests: `test_alertas_http`, `test_alertas_validacion`.
- Captura: `captures/14-admin-stock.png`.
- Spec: [011-alertas-stock-minimo](../specs/011-alertas-stock-minimo/spec.md).

---

## Matriz de trazabilidad

| CU | Specs dueñas | FR principales | Tests de backend | Tests de frontend/asistente | Captura |
|---|---|---|---|---|---|
| CU-01 | 003, 005 | FR-009…FR-018, FR-020…FR-023, FR-074, FR-075 | `test_productos_http`, `test_categorias_http`, `test_catalogo_baja_logica`, `test_depositos_http`, `test_deposito_validacion` | `productos*.test.ts`, `categorias*.test.ts`, `depositos*.test.ts` | 11, 12, 13 |
| CU-02 | 007, 012, 016 | FR-024…FR-030, FR-044…FR-048, FR-063…FR-065 | `test_compras_http`, `test_compras_concurrencia`, `test_importacion_http`, `test_movimientos_http` | `compras*.test.ts`, `importaciones*.test.ts` | 15, 16 |
| CU-03 | 003, 011 | FR-041…FR-043 | `test_alertas_http`, `test_alertas_validacion`, `test_stock_http` | `alertas*.test.ts`, `stock*.test.ts` | 14 |
| CU-04 | 007 | FR-024…FR-028 | `test_movimientos_http`, `test_movimiento_validacion` | `movimientos*.test.ts` | 06 |
| CU-05 | 010, 014 | FR-038…FR-040, FR-056…FR-059 | `test_reportes_http`, `test_reportes_calculo`, `test_stock_http` | `reportes*.test.ts` | 14 + nuevas |
| CU-06 | 001, 002 | FR-001…FR-008b | `test_usuarios_http`, `test_ultimo_admin`, `test_auth_http` | `usuarios*.test.ts`, `auth.test.ts` | 10 |
| CU-07 | 007 | FR-024…FR-030 | `test_movimientos_http`, `test_movimiento_validacion` | `movimientos*.test.ts` | 06 |
| CU-08 | 004, 005, 010 | FR-011, FR-013, FR-038…FR-040, FR-075 | `test_busqueda_http`, `test_stock_http` | `busquedaCaja.test.ts`, `inventario.test.ts`, `stock*.test.ts` | 05 |
| CU-09 | 003, 004 | FR-009, FR-011 | `test_busqueda_http`, `test_productos_http`, `test_producto_validacion` | `productos*.test.ts`, `busquedaCaja.test.ts` | 05, 12 |
| CU-10 | 011 | FR-041…FR-043 | `test_alertas_http`, `test_alertas_validacion` | `alertas*.test.ts` | 14 |

## Alcance adicional (fuera de los 10)

| Spec | Funcionalidad |
|---|---|
| 001/002 | Registro público, SSO Google/Microsoft, sesión única, contraseña temporal |
| 006 | RBAC transversal y auditoría de todas las escrituras de gestión |
| 008 | Transferencias atómicas entre depósitos |
| 009 | Inventario físico: conteo y ajuste automático |
| 013 | Ventas de caja, clientes y anulación con reversa |
| 015 | Auditoría consultable |
| 017 | Asistente de inventario con LangChain (criterio 5 del TP) |
| 018 | Rediseño visual de escritorio y tema claro/oscuro |
| 019 | Rediseño visual móvil (implementación parcial) |

## Cómo se verifica

1. **Automático**: `cargo test`, `cargo sqlx prepare --check`, `npm test`, `npm run check`,
   `npm run build` y los tests de `asistente/`.
2. **Interactivo**: recorrido manual de los 10 CUs con backend, frontend y base reales; se
   registra en [`docs/aceptacion.md`](./aceptacion.md).
3. **Evidencia visual**: `captures/` y
   [`specs/018-rediseno-visual-desktop/evidencia/`](../specs/018-rediseno-visual-desktop/evidencia/).

> **Nota sobre el origen**: esta lista es la propuesta original del TP. Cada CU tiene una spec
> aprobada antes de su implementación (Spec-Driven Development); el historial de `specs/` y los
> commits de especificación respaldan ese orden.
