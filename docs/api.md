# API de Stockmante

Referencia navegable: **Swagger UI** en `http://localhost:3000/docs` y el contrato legible por
máquina en `http://localhost:3000/api-docs/openapi.json` (OpenAPI 3.1). El JSON se genera desde el
código (`utoipa`); esta página resume las decisiones que no se ven en el esquema.

## Autenticación

- El login crea una sesión única y setea la cookie HttpOnly `stockmante_sesion`
  (`SameSite=Lax`; `Secure` en producción).
- Swagger UI usa el esquema `cookieAuth`: después de `POST /api/auth/login`, el navegador envía la
  cookie automáticamente.
- Sin sesión: `401 no_autenticado`. Sesión vencida/revocada: `401 sesion_expirada`.
- Contraseña temporal vigente: `403 cambio_password_requerido` en toda la superficie autenticada
  salvo login/logout/sesión/cambio de contraseña.
- Familia `OPERACION` en una ruta de gestión: `403 sin_permisos`.

## Familias de rol (RBAC)

| Rol | Familia | Puede |
|---|---|---|
| `ADMIN`, `GERENTE` | `GESTION` | CRUD de usuarios, categorías, productos, depósitos, ubicaciones, proveedores y compras; todos los movimientos incluido `AJUSTE`; conteos de inventario físico |
| `OPERARIO`, `CAJERO` | `OPERACION` | Lectura de catálogo, búsqueda de caja, su propia sesión, consulta de stock e historial, y movimientos `ENTRADA`/`SALIDA`/`MERMA` |

Las rutas de escritura y los listados de administración exigen `RequiereGestion`. El backend es la
autoridad: ocultar un enlace en la UI no reemplaza el 403.

## Auditoría

Las altas, ediciones, bajas lógicas, reactivaciones, reseteos de contraseña temporal, movimientos
de stock, transferencias, compras, ventas e importaciones se registran en la tabla `auditoria`
(`tabla_ref`, `registro_id`, `accion`, `usuario_id`, `cuando`). La fila se escribe en la misma
transacción del cambio: un 401/403 o una validación fallida no deja registro.

La consulta es de solo lectura y exige familia `GESTION` (spec 015):

| Método | Ruta | Query | Éxito |
|---|---|---|---|
| GET | `/api/auditoria` | `tabla_ref`, `usuario_id`, `desde`, `hasta`, `pagina`, `por_pagina` | 200 `{ items, total, pagina, por_pagina }` |

`desde`/`hasta` aceptan `AAAA-MM-DD` o RFC 3339. `OPERACION` recibe `403 sin_permisos`. La
pantalla es `/admin/auditoria`.

## Sobre de error

Todas las respuestas 4xx/5xx usan:

```json
{ "error": { "codigo": "validacion", "mensaje": "El SKU debe tener entre 3 y 30 caracteres." } }
```

Códigos estables: `credenciales_invalidas`, `sesion_expirada`, `no_autenticado`, `sin_permisos`,
`recurso_no_encontrado`, `email_duplicado`, `sku_duplicado`, `codigo_barras_duplicado`,
`categoria_duplicada`, `deposito_con_dependencias`, `deposito_nombre_duplicado`,
`deposito_identificador_duplicado`, `ubicacion_huerfana`, `ubicacion_con_dependencias`,
`ubicacion_duplicada`, `proveedor_duplicado`, `cuit_duplicado`, `compra_ya_confirmada`, `compra_no_editable`,
`ultimo_administrador`, `cambio_password_requerido`, `stock_insuficiente`,
`producto_inactivo`, `validacion`, `interno`.

## Endpoints

### Autenticación

| Método | Ruta | Auth | Éxito |
|---|---|---|---|
| POST | `/api/auth/login` | pública | 204 + cookie |
| POST | `/api/auth/logout` | pública | 204 |
| GET | `/api/auth/sesion` | sesión | 200 `{ usuario }` |
| POST | `/api/auth/cambiar-password` | sesión | 204 |

### Usuarios (GESTION)

| Método | Ruta | Éxito |
|---|---|---|
| GET | `/api/usuarios?activo=&pagina=&por_pagina=` | 200 página |
| POST | `/api/usuarios` | 201 usuario (temporal por correo) |
| GET | `/api/usuarios/{id}` | 200 |
| PUT | `/api/usuarios/{id}` | 200 |
| POST | `/api/usuarios/{id}/desactivar` | 204 |
| POST | `/api/usuarios/{id}/reactivar` | 204 |
| POST | `/api/usuarios/{id}/password-temporal` | 204 |

### Categorías

| Método | Ruta | Auth | Éxito |
|---|---|---|---|
| GET | `/api/categorias?activo=true\|false\|all` | sesión | 200 |
| POST | `/api/categorias` | GESTION | 201 |
| GET | `/api/categorias/{id}` | sesión | 200 |
| PUT | `/api/categorias/{id}` | GESTION | 200 |
| POST | `/api/categorias/{id}/desactivar` | GESTION | 204 |
| POST | `/api/categorias/{id}/reactivar` | GESTION | 204 |

No existe `DELETE`: la baja es lógica (`activo = false`).

### Productos

| Método | Ruta | Auth | Éxito |
|---|---|---|---|
| GET | `/api/productos?q=&categoria_id=&activo=&pagina=&por_pagina=` | GESTION | 200 página |
| POST | `/api/productos` | GESTION | 201 |
| GET | `/api/productos/{id}` | sesión | 200 |
| PUT | `/api/productos/{id}` | GESTION | 200 |
| POST | `/api/productos/{id}/desactivar` | GESTION | 204 |
| POST | `/api/productos/{id}/reactivar` | GESTION | 204 |
| GET | `/api/productos/buscar?q=&campo=&incluir_inactivos=&categoria_id=&limite=` | sesión | 200 `{ items }` |

La búsqueda de caja (`/buscar`) excluye inactivos por defecto; `incluir_inactivos=true` exige
GESTION. `campo=nombre|categoria` exige `q` de al menos 2 caracteres; `sku|codigo` buscan desde el
primer carácter. `campo` ausente equivale a `todos` (SKU/código exactos primero, luego nombre).
`nombre` y `categoria` comparan igual: trigramas (`%`) o subcadena (`ILIKE '%q%'`) sobre
`productos.nombre` y `categorias.nombre`, ordenado por similitud. En `todos` el SKU se compara
normalizado (trim + mayúsculas), igual que en `campo=sku`. La respuesta no informa si hubo más
coincidencias que `limite`: el cliente lo infiere cuando `items.length == limite` (en `/caja`, aviso
«Se muestran los primeros N resultados…»). Contrato completo en
[`specs/004-busqueda-caja/contracts/api.md`](../specs/004-busqueda-caja/contracts/api.md).

### Depósitos y ubicaciones (GESTION)

| Método | Ruta | Éxito |
|---|---|---|
| GET | `/api/depositos?activo=` | 200 |
| POST | `/api/depositos` | 201 |
| GET | `/api/depositos/{id}` | 200 |
| PUT | `/api/depositos/{id}` | 200 |
| DELETE | `/api/depositos/{id}` | 204, o 409 `deposito_con_dependencias` |
| POST | `/api/depositos/{id}/desactivar` | 204 |
| POST | `/api/depositos/{id}/reactivar` | 204 |
| GET | `/api/depositos/{id}/ubicaciones` | 200 |
| POST | `/api/ubicaciones` | 201 |
| GET | `/api/ubicaciones/{id}` | 200 |
| PUT | `/api/ubicaciones/{id}` | 200 |
| DELETE | `/api/ubicaciones/{id}` | 204, o 409 `ubicacion_con_dependencias` |
| POST | `/api/ubicaciones/{id}/desactivar` | 204 |
| POST | `/api/ubicaciones/{id}/reactivar` | 204 |
| GET | `/api/productos/{id}/ubicaciones` | 200 |
| PUT | `/api/productos/{id}/ubicaciones` | 204 (reemplaza el conjunto) |
| GET | `/api/productos/ubicaciones?producto_ids=` | 200 (lote; `[]` por producto sin asignaciones) |

Crear una ubicación con `deposito_id` inexistente o con un `padre_id` de otro depósito es
`422 ubicacion_huerfana`. Mover una ubicación con hijos o productos asignados es
`409 ubicacion_con_dependencias`. La lectura masiva acepta 1–100 ids separados por coma; ausente,
más de 100, `0` o un valor no entero → `422 validacion`; los ids inexistentes se omiten.

### Movimientos de stock (Módulo 2, spec 007)

| Método | Ruta | Auth | Éxito |
|---|---|---|---|
| POST | `/api/movimientos` | sesión¹ | 201 movimiento con stock resultante |
| GET | `/api/movimientos?tipo=&deposito_id=&producto_id=&desde=&hasta=&pagina=&por_pagina=` | sesión | 200 página |
| GET | `/api/movimientos/{id}` | sesión | 200 con líneas |
| GET | `/api/stock?q=&deposito_id=&categoria_id=&bajo_minimo=&activo=&pagina=&por_pagina=` | sesión | 200 página |
| GET | `/api/stock/{producto_id}` | sesión | 200 desglose por depósito |

¹ `ENTRADA`, `SALIDA` y `MERMA` para `OPERACION` y `GESTION`; `AJUSTE` solo `GESTION`
(403 `sin_permisos`).

Un movimiento tiene cabecera (tipo, depósito, motivo, observación opcional) y una o más
líneas con `cantidad_delta` con signo: `ENTRADA > 0`, `SALIDA`/`MERMA < 0`, `AJUSTE ≠ 0`.
La validación completa ocurre en una única transacción: si una línea falla, no se escribe
nada (cabecera, líneas, auditoría ni stock). Errores nuevos: `409 stock_insuficiente`
(`"Stock insuficiente en Principal: 3 disponibles, 5 solicitadas. Reducí la cantidad o registrá
antes una entrada en ese depósito."`), `409 producto_inactivo` (`"El producto está inactivo y no
admite movimientos. Reactivalo desde Productos o quitalo de la operación."`) y `422 validacion`
para depósito/ubicación inválidos. Motivo `OTRO` exige observación; **todo `AJUSTE` exige una
justificación en `observacion`** (aunque el motivo sea `AJUSTE`), porque es la justificación de
auditoría de la corrección.

**Matriz de motivos del alta manual** (007, 2026-11-07): `ENTRADA` → `COMPRA`, `DEVOLUCION`,
`OTRO`; `SALIDA` → `VENTA`, `DEVOLUCION`, `OTRO`; `MERMA` → `MERMA`, `OTRO`; `AJUSTE` → `AJUSTE`,
`OTRO`. Otra combinación → 422 `"El motivo {motivo} no corresponde a una {tipo}. Motivos válidos:
{lista}."`. `TRANSFERENCIA`, `INVENTARIO` e `IMPORTACION` están reservados a sus flujos (422 que
nombra el flujo). `cantidad_delta` no entera → 422 `"La cantidad de cada línea debe ser un número
entero distinto de cero."`; fuera de `i32` → `"La cantidad de cada línea supera el máximo admitido.
Revisá el valor."`.

**Orden de evaluación** (gana el primer error): JSON mal formado y tipo inválido → 403 si `AJUSTE`
sin `GESTION` → motivo, coherencia tipo ↔ motivo, observación, líneas y cantidades → dentro de la
transacción: depósito (`"El depósito {id} no existe…"`, inactivo), productos (`"El producto {id} no
existe…"`, luego 409 `producto_inactivo`), ubicaciones y stock (409 `stock_insuficiente`).

`stock_por_deposito` es la fuente de verdad (con ubicación opcional);
`productos.stock_actual` es el cache global recalculado en la misma transacción.

**Consulta de stock (010)**: `GET /api/stock` acepta además `producto_ids` (ids separados por
coma, 1–50; 422 `"producto_ids admite hasta 50 ids."` o `"producto_ids debe ser una lista de
enteros mayores a 0 separados por coma."`). Cada item trae `faltante` (`max(stock_minimo −
stock_actual, 0)`), y `costo`/`valorizado` (`costo × (stock_deposito ?? stock_actual)`, 2
decimales) **solo para `GESTION`**: para `OPERACION` llegan en `null`. `GET /api/stock/{id}` agrega
`faltante`. `GET /api/depositos/activos` (cualquier sesión) devuelve `[{ id, nombre }]` de los
depósitos activos: es la lista que usan caja, movimientos y transferencias.

### Transferencias entre depósitos (Módulo 2, spec 008)

| Método | Ruta | Auth | Éxito |
|---|---|---|---|
| POST | `/api/transferencias` | sesión² | 201 transferencia con stock resultante en origen y destino |
| GET | `/api/transferencias?producto_id=&deposito_origen_id=&deposito_destino_id=&desde=&hasta=&pagina=&por_pagina=` | sesión | 200 página |
| GET | `/api/transferencias/{id}` | sesión | 200 con los ids de los dos movimientos generados (`movimiento_salida_id`, `movimiento_entrada_id`) |

² `OPERACION` y `GESTION`. Errores: `409 stock_insuficiente`, `409 producto_inactivo`,
`422 transferencia_mismo_deposito` (`"El depósito de origen y el de destino deben ser distintos."`),
`422 validacion` para depósito/ubicación inválidos o inactivos.

La operación es inmediata y atómica: descuenta origen, incrementa destino, escribe dos
movimientos (`SALIDA` y `ENTRADA`, motivo `TRANSFERENCIA`) enlazados a la transferencia
y audita (`tabla_ref = 'transferencias'`, acción `CREAR`). La suma global de stock se
conserva (SC-016).

El stock se valida y se informa **por fila de ubicación**, igual que en 007: sin
`ubicacion_origen_id` se usa la fila del depósito (ubicación nula), aunque el producto tenga
unidades en otras ubicaciones de ese depósito. `stock_origen_resultante` y
`stock_destino_resultante` son el saldo de esas filas y coinciden con
`movimiento_lineas.stock_resultante` de cada movimiento. El detalle no embebe los movimientos:
se consultan con `GET /api/movimientos/{id}`. Un destino que desbordaría `i32` responde 422
`validacion`; un id inexistente responde 404 (`"No existe la transferencia {id}. Revisá el número
o buscala en el listado de transferencias."`).

### Inventario físico (Módulo 2, spec 009)

Toda la superficie exige familia `GESTION`; `OPERACION` recibe `403 sin_permisos`.

| Método | Ruta | Éxito |
|---|---|---|
| POST | `/api/conteos` (`{ deposito_id, observacion? }`) | 201 conteo `ABIERTO` con una línea por fila de stock del depósito |
| GET | `/api/conteos?deposito_id=&estado=&desde=&hasta=&pagina=&por_pagina=` | 200 página |
| GET | `/api/conteos/{id}` | 200 con líneas |
| PATCH | `/api/conteos/{id}/lineas` (`{ lineas: [{ linea_id, cantidad_contada }] }`) | 200; `diferencia` contra el snapshot de apertura |
| POST | `/api/conteos/{id}/cerrar` | 200 `CERRADO`; un `AJUSTE` (motivo `INVENTARIO`) solo por las líneas con diferencia |
| POST | `/api/conteos/{id}/cancelar` | 200 `CANCELADO`, sin ajuste |

El cierre recalcula contra el stock **vigente** y deja cada fila igual a lo contado; si nada
difiere, `movimiento_ajuste_id` queda en `null`. Errores: `422 deposito_inactivo`, `422 validacion`
(depósito sin stock registrado: no se crea el conteo), `422 lineas_sin_contar`,
`409 conteo_cerrado`, `409 conteo_cancelado`, `404 conteo_no_encontrado`.

### Alertas de stock mínimo (Módulo 2, spec 011)

Toda la superficie exige familia `GESTION`; `OPERACION` recibe `403 sin_permisos`.

| Método | Ruta | Éxito |
|---|---|---|
| GET | `/api/alertas?pagina=&por_pagina=` (default 50, máx. 100) | 200 `{ items: [{ producto_id, nombre, sku, stock_actual, stock_minimo, faltante }], total, pagina, por_pagina }` |
| POST | `/api/alertas/enviar` (sin cuerpo) | 200 `{ fecha, productos, destinatarios, enviados, fallidos }` |

Una alerta es un producto activo con `stock_minimo > 0` y `stock_actual < stock_minimo`. El ciclo
(scheduler cada `ALERTAS_INTERVALO_MINUTOS`, default 60, `0` lo apaga; o el `POST`) manda **un
correo por gestor activo** con sus productos pendientes del día local (UTC−03:00) y registra cada
par producto–gestor en `notificaciones_stock`: es idempotente en el día. Un fallo de transporte
responde 200 con `fallidos > 0` y se reintenta en el ciclo siguiente. Errores: 401, 403, 422
(paginación).

### Proveedores y compras (Módulo 3, spec 012)

Toda la superficie exige familia `GESTION`; `OPERACION` recibe `403 sin_permisos`.

| Método | Ruta | Éxito |
|---|---|---|
| GET | `/api/proveedores?activo=true\|false\|all&pagina=&por_pagina=` | 200 página |
| POST | `/api/proveedores` | 201 proveedor |
| GET | `/api/proveedores/{id}` | 200 |
| PUT | `/api/proveedores/{id}` | 200 (DTO completo) |
| POST | `/api/proveedores/{id}/desactivar` | 200 proveedor con `activo=false` |
| POST | `/api/proveedores/{id}/activar` | 200 proveedor con `activo=true` |
| GET | `/api/compras?proveedor_id=&estado=&desde=&hasta=&pagina=&por_pagina=` | 200 página |
| POST | `/api/compras` | 201 `{ compra, lineas }` en `BORRADOR` |
| GET | `/api/compras/{id}` | 200 `{ compra, lineas }` |
| PUT | `/api/compras/{id}` | 200 (solo `BORRADOR`) |
| DELETE | `/api/compras/{id}` | 204 (solo `BORRADOR`) |
| POST | `/api/compras/{id}/confirmar` | 200 `{ compra, lineas }` en `CONFIRMADA` |
| POST | `/api/compras/{id}/anular` | 200 `{ compra, lineas }` en `ANULADA` |

Confirmar ejecuta una única transacción: escribe un movimiento `ENTRADA` con motivo `COMPRA`,
actualiza `stock_por_deposito`, el costo del producto y el cache `stock_actual`. Confirmar dos
veces responde `409 compra_ya_confirmada` sin duplicar stock. Editar/eliminar una compra que no
está en `BORRADOR` responde `409 compra_no_editable`. Anular revierte el stock con un movimiento
`SALIDA` (motivo `DEVOLUCION`) y falla con `409 stock_insuficiente` si ya no alcanza.

**Fase 6 (2026-11-07)**: `POST /api/compras/{id}/confirmar` y `/anular` devuelven
`ResultadoCompra` = `{ compra, lineas, movimiento }`, donde `movimiento` es el `MovimientoDetalle`
de 007 (`ENTRADA`/`COMPRA` al confirmar; `SALIDA`/`DEVOLUCION` con `cantidad_delta` negativo al
anular). Errores nuevos: `409 proveedor_inactivo` (`"El proveedor «<nombre>» está dado de baja;
reactivalo o elegí otro proveedor."`, al crear, editar y confirmar) y `409 compra_no_anulable`
(borrador o ya anulada). `GET /api/compras` responde 422 si `estado` no es BORRADOR/CONFIRMADA/
ANULADA o si `desde`/`hasta` no son `AAAA-MM-DD`/RFC 3339 (el mensaje nombra el parámetro). Límites:
`numero` ≤ 40 caracteres, `costo_unitario` entre 0 y 9.999.999.999,99 (422 que nombra el campo).
La anulación bloquea depósito y productos en el orden de 007 y descuenta con un `UPDATE`
condicional: frente a una salida concurrente una gana y la otra recibe 409 `stock_insuficiente`.


### Ventas de caja y clientes (Módulo 3, spec 013)

| Método | Ruta | Permiso | Éxito |
|---|---|---|---|
| POST | `/api/ventas` | sesión (OPERACION o GESTION) | 201 `VentaDetalle` con `lineas[].stock_resultante` |
| GET | `/api/ventas?deposito_id=&cliente_id=&estado=&desde=&hasta=&pagina=&por_pagina=` | sesión | 200 página |
| GET | `/api/ventas/{id}` | sesión | 200 `VentaDetalle` |
| POST | `/api/ventas/{id}/anular` | **GESTION** | 200 `VentaDetalle` `ANULADA`; `ENTRADA` motivo `DEVOLUCION` |
| GET | `/api/clientes?activo=&pagina=&por_pagina=`, `/api/clientes/{id}` | sesión | 200 |
| POST/PUT | `/api/clientes`, `/api/clientes/{id}`, `/{id}/activar`, `/{id}/desactivar` | **GESTION** | 201/200 `Cliente` |

Cuerpo de la venta: `{ deposito_id, cliente_id?, medio_pago (EFECTIVO|TARJETA|TRANSFERENCIA),
descuento?, observacion?, lineas: [{ producto_id, cantidad }] }`. Las líneas del mismo producto se
**agregan** (2 + 1 = 3) y el precio es el de `productos.precio_venta` congelado al confirmar;
mandar `precio_unitario` es 422 (`"El precio unitario se toma del catálogo; no se puede enviar en
la venta."`). Errores: `409 stock_insuficiente` (sobre la cantidad agregada; no se registra nada),
`422 validacion` (producto inactivo `"El producto no está disponible para la venta."`, descuento
mayor al subtotal, medio de pago, cantidad), `409 venta_ya_anulada`, `409 documento_duplicado`
(`"Ya existe un cliente con ese documento."`), `403 sin_permisos`. Ejemplo de línea:
`{ "producto_id": 12, "cantidad": 3, "precio_unitario": 3000, "subtotal": 9000, "stock_resultante": 7 }`.

### Reportes (Módulo 3, spec 014)

Toda la superficie exige familia `GESTION` (403 `sin_permisos` para `OPERACION`, también en CSV).

| Método | Ruta | Éxito |
|---|---|---|
| GET | `/api/reportes/kardex?producto_id=&deposito_id=&desde=&hasta=&formato=` | 200 `{ producto_id, producto_nombre, deposito_id, desde, hasta, saldo_inicial, lineas[{ movimiento_id, creado_en, tipo, motivo, deposito_id, deposito_nombre, cantidad, saldo }], saldo_final }` |
| GET | `/api/reportes/stock-valorizado?formato=` | 200 `{ items[{ producto_id, nombre, sku, cantidad, costo_unitario, valor }], total }` |
| GET | `/api/reportes/rotacion?desde=&hasta=&formato=` | 200 `{ desde, hasta, items[{ producto_id, nombre, unidades_salidas, stock_inicial, stock_final, stock_promedio, indice_rotacion }] }` |
| GET | `/api/reportes/ventas?desde=&hasta=&formato=&agrupar=` | 200 `{ desde, hasta, cantidad_ventas, total, por_dia[{ dia, cantidad_ventas, total }], por_producto[{ producto_id, nombre, unidades, total }] }` |

`desde`/`hasta` son días locales `AAAA-MM-DD` (UTC−03:00) **inclusivos** (también se acepta RFC
3339, del que se toma la fecha local); rotación y ventas los exigen. El stock valorizado sale de
`stock_por_deposito` (activos o no, solo con stock); la rotación cuenta `SALIDA` sin
`TRANSFERENCIA`; ventas solo `CONFIRMADA` y por día local. `formato=csv` devuelve `text/csv;
charset=utf-8` (RFC 4180, BOM, `\r\n`) con `Content-Disposition: attachment;
filename="<reporte>_<desde>_<hasta>.csv"`; en ventas `agrupar=dia|producto` elige la tabla.
Errores 422: `"La fecha desde no puede ser posterior a la fecha hasta."`, `"Indicá las fechas desde
y hasta del período (formato AAAA-MM-DD)."`, `"El parámetro desde debe estar en formato ISO 8601
(AAAA-MM-DD)."`, `"El parámetro formato acepta json o csv."`, `"El parámetro agrupar acepta dia o
producto."`; 404 `"El producto no existe."` en kardex. Sin paginación.

### Importación masiva (Módulo 4, spec 016)

| Método | Ruta | Permiso | Éxito |
|---|---|---|---|
| POST | `/api/importaciones/productos?modo=validar\|aplicar` (`multipart/form-data`, campo `archivo`) | **GESTION** | 200 (validar) / 201 (aplicar) `{ filas_leidas, validas, con_error, errores[{ fila, columna, mensaje }], creados, aplicado }` |

CSV UTF-8 (se tolera BOM) de hasta 5.000 filas y 5 MB. Columnas: `nombre`, `sku`, `categoria`,
`unidad_medida`, `costo`, `precio_venta`, `stock_minimo` y opcionales `codigo_barras`,
`stock_inicial` (vacío = 0) y `deposito` (vacío = «Principal»; solo se exige con stock > 0). El
delimitador se detecta (`;` → decimal `,`; `,` → decimal `.`). `validar` nunca escribe y devuelve los
errores por celda con 200; `aplicar` es **todo o nada**: con cualquier error responde
`422 importacion_invalida` con `error.detalles` y no crea nada; sin errores crea productos,
stock inicial (movimiento `ENTRADA` motivo `IMPORTACION`) y auditoría en una transacción. Errores
globales (también `importacion_invalida`, `fila` 0/1): límite superado, archivo vacío, columna
obligatoria faltante, CSV ilegible y falta del campo `archivo`. `modo` inválido → `422 validacion`.
La importación nunca crea categorías ni depósitos ni actualiza productos existentes.

`error.detalles` aparece **solo** en `importacion_invalida`; el resto de los errores conserva el
sobre `{ error: { codigo, mensaje } }`.

### Asistente de inventario (Módulo 4, spec 017)

| Método | Ruta | Auth | Éxito |
|---|---|---|---|
| POST | `/api/asistente/consulta` | sesión | 200 `{ respuesta, accion_propuesta }` |

Body: `{ mensaje, historial? }`. `mensaje` 1–2000 caracteres; `historial` es
`[{ rol: "usuario" | "asistente", texto }]` y se reenvían solo los 10 más recientes. El backend
reenvía la consulta al servicio LangChain (`asistente/`, `ASISTENTE_URL`) junto con la cookie de
sesión del usuario: las tools leen esta misma API con sus permisos.

El asistente **nunca escribe**. Para un movimiento devuelve `accion_propuesta`
(`{ tipo, deposito_id, deposito_nombre, producto_id, producto_nombre, sku, cantidad_delta, motivo,
observacion }`); si el usuario la confirma, el frontend la manda a `POST /api/movimientos`, que
valida permisos, ejecuta la transacción y audita con el usuario real. `AJUSTE` solo se propone a
GESTION. Las conversaciones no se persisten y el log guarda solo metadatos.

Errores: `422 validacion` (mensaje vacío o largo), `429 asistente_limite_excedido` (más de 20
consultas por minuto por usuario), `503 asistente_no_disponible` (sin `ASISTENTE_URL`, sin LLM
configurado en el servicio, o servicio caído) y `504 asistente_tiempo_agotado` (más de 30 s).

### Salud y documentación

| Método | Ruta | Auth | Éxito |
|---|---|---|---|
| GET | `/api/salud` | pública | 200 `{ ok: true }` |
| GET | `/docs` | pública | Swagger UI |
| GET | `/api-docs/openapi.json` | pública | OpenAPI 3.1 |

`/docs` y `/api-docs/openapi.json` responden 404 con `DOCS_ENABLED=false`.

## Convenciones

- Montos: `NUMERIC(12,2)` como número JSON (no string), `>= 0`, hasta 2 decimales.
- Fechas: ISO-8601 UTC (`creado_en`, `cuando`).
- Paginación: `pagina` desde 1, `por_pagina` 1–100 (default 20).
- Idioma de mensajes: español (es-AR).
- Los contratos de negocio detallados viven en `specs/*/contracts/api.md`.
