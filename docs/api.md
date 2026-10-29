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
| `ADMIN`, `GERENTE` | `GESTION` | CRUD de usuarios, categorías, productos, depósitos, ubicaciones y asignaciones; todos los movimientos incluido `AJUSTE` |
| `OPERARIO`, `CAJERO` | `OPERACION` | Lectura de catálogo, búsqueda de caja, su propia sesión, consulta de stock e historial, y movimientos `ENTRADA`/`SALIDA`/`MERMA` |

Las rutas de escritura y los listados de administración exigen `RequiereGestion`. El backend es la
autoridad: ocultar un enlace en la UI no reemplaza el 403.

## Auditoría

Las altas, ediciones, bajas lógicas, reactivaciones y el reseteo de contraseña temporal de
usuarios y productos se registran en la tabla `auditoria` (`tabla_ref`, `registro_id`, `accion`,
`usuario_id`, `cuando`). La fila se escribe en la misma transacción del cambio: un 401/403 o una
validación fallida no deja registro. No hay endpoint de consulta en el Módulo 1.

## Sobre de error

Todas las respuestas 4xx/5xx usan:

```json
{ "error": { "codigo": "validacion", "mensaje": "El SKU debe tener entre 3 y 30 caracteres." } }
```

Códigos estables: `credenciales_invalidas`, `sesion_expirada`, `no_autenticado`, `sin_permisos`,
`recurso_no_encontrado`, `email_duplicado`, `sku_duplicado`, `codigo_barras_duplicado`,
`categoria_duplicada`, `deposito_con_dependencias`, `deposito_nombre_duplicado`,
`deposito_identificador_duplicado`, `ubicacion_huerfana`, `ubicacion_con_dependencias`,
`ubicacion_duplicada`,
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

Crear una ubicación con `deposito_id` inexistente o con un `padre_id` de otro depósito es
`422 ubicacion_huerfana`. Mover una ubicación con hijos o productos asignados es
`409 ubicacion_con_dependencias`.

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
(`"Stock insuficiente en Principal: 3 disponibles, 5 solicitadas."`), `409 producto_inactivo`
y `422 validacion` para depósito/ubicación inválidos. Motivo `OTRO` exige observación.

`stock_por_deposito` es la fuente de verdad (con ubicación opcional);
`productos.stock_actual` es el cache global recalculado en la misma transacción.

### Transferencias entre depósitos (Módulo 2, spec 008)

| Método | Ruta | Auth | Éxito |
|---|---|---|---|
| POST | `/api/transferencias` | sesión² | 201 transferencia con stock resultante en origen y destino |
| GET | `/api/transferencias?producto_id=&deposito_origen_id=&deposito_destino_id=&desde=&hasta=&pagina=&por_pagina=` | sesión | 200 página |
| GET | `/api/transferencias/{id}` | sesión | 200 con los dos movimientos generados |

² `OPERACION` y `GESTION`. Errores: `409 stock_insuficiente`, `409 producto_inactivo`,
`422 transferencia_mismo_deposito` (`"El depósito de origen y el de destino deben ser distintos."`),
`422 validacion` para depósito/ubicación inválidos o inactivos.

La operación es inmediata y atómica: descuenta origen, incrementa destino, escribe dos
movimientos (`SALIDA` y `ENTRADA`, motivo `TRANSFERENCIA`) enlazados a la transferencia
y audita (`tabla_ref = 'transferencias'`, acción `CREAR`). La suma global de stock se
conserva (SC-016).

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
