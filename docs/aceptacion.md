# Aceptación manual — los 10 casos de uso

**Objetivo**: dejar registro de la verificación **interactiva** (backend + frontend + base reales)
de los 10 casos de uso del TP, complementando los tests automáticos.
**Fecha de la corrida**: pendiente de completar en la sesión de demo.
**Entorno**: PostgreSQL 18 local, backend Axum, frontend SvelteKit, asistente Node 24 + Groq.

> Los tests automáticos ya están en verde (ver [`README.md`](../README.md#calidad)). Este
> documento es la evidencia del recorrido humano; se completa marcando cada paso y anotando el
> resultado observado.

## Preparación

1. Base creada y migrada:
   ```bash
   sqlx migrate run
   cd backend && cargo run --bin bootstrap
   ```
2. Backend:
   ```bash
   cd backend && cargo run --bin stockmante-backend
   ```
3. Frontend:
   ```bash
   cd frontend && npm run dev
   ```
4. Asistente (solo para CU-10 / criterio 5):
   ```bash
   cd asistente && npm start
   ```
   Con `ASISTENTE_URL=http://127.0.0.1:3100` en `.env` y `LLM_PROVIDER`/`LLM_MODEL`/`LLM_API_KEY`
   configurados.
5. Usuario de prueba: un `GESTION` sin cambio de contraseña pendiente y, si se quiere probar RBAC,
   un `OPERACION`.

## Checklist

### CU-01 — Productos, ubicaciones y depósitos

- [ ] Crear una categoría en `/admin/categorias`.
- [ ] Crear un producto con SKU único, costo, precio y stock mínimo en `/admin/productos`.
- [ ] Crear un depósito y una ubicación hija en `/admin/depositos`.
- [ ] Asignar la ubicación al producto y verla en el listado de productos.
- [ ] Intentar crear otro producto con el mismo SKU y verificar `409 sku_duplicado`.
- [ ] Dar de baja el producto y verificar que sigue consultable por `id` y no aparece en el listado estándar.

**Resultado observado**: _pendiente_.
**Capturas**: `captures/11-admin-categorias.png`, `12-admin-productos.png`, `13-admin-depositos.png`.

### CU-02 — Entrada de stock (compra/reabastecimiento)

- [ ] Crear un proveedor en `/admin/proveedores`.
- [ ] Crear una compra en borrador con número de factura/remito y una línea.
- [ ] Confirmar la compra y verificar que subió el stock y que el movimiento es `ENTRADA`/`COMPRA`.
- [ ] Intentar confirmar de nuevo y verificar `409 compra_ya_confirmada`.
- [ ] Anular la compra y verificar la reversa (`SALIDA`/`DEVOLUCION`).
- [ ] (Opcional) Importar un CSV en `/admin/importacion` y verificar el stock inicial.

**Resultado observado**: _pendiente_.
**Capturas**: `captures/15-admin-proveedores.png`, `16-admin-compras.png`.

### CU-03 — Alertas de stock mínimo

- [ ] Editar un producto con `stock_minimo = 10` y dejar el stock en 4.
- [ ] Verlo en `/admin/stock` con `faltante = 6`.
- [ ] Disparar `POST /api/alertas/enviar` (o esperar el ciclo) y verificar el digest.
- [ ] Verificar que una segunda ejecución el mismo día no duplica el correo.
- [ ] Con `OPERACION`, verificar `403 sin_permisos` en `/api/alertas`.

**Resultado observado**: _pendiente_.
**Capturas**: `captures/14-admin-stock.png`.

### CU-04 — Ajuste manual con justificación

- [ ] En `/movimientos`, elegir `AJUSTE`, depósito y cantidad.
- [ ] Intentar confirmar sin observación y verificar el error de validación (la UI la exige).
- [ ] Completar la justificación y confirmar; verificar el stock resultante y la auditoría.
- [ ] Repetir con `MERMA` + motivo `OTRO` sin observación y verificar `422`.
- [ ] Con `OPERACION`, verificar que `AJUSTE` no está disponible y la API responde `403`.

**Resultado observado**: _pendiente_.
**Capturas**: `captures/06-movimientos.png`.

### CU-05 — Valorización y reportes

- [ ] En `/admin/reportes`, abrir kardex de un producto y verificar el saldo corrido.
- [ ] Abrir stock valorizado y verificar cantidad × costo.
- [ ] Abrir rotación con un rango de fechas y verificar el índice.
- [ ] Abrir ventas y descargar el CSV; verificar el archivo.
- [ ] Con `OPERACION`, verificar `403` en los cuatro reportes (también en CSV).

**Resultado observado**: _pendiente_.
**Capturas**: `captures/14-admin-stock.png` y captura de `/admin/reportes`.

### CU-06 — Usuarios y roles

- [ ] Crear un usuario `OPERACION` y ver las credenciales en pantalla.
- [ ] Iniciar sesión con la temporal; verificar el cambio obligatorio.
- [ ] Volver como `GESTION`, desactivar el usuario y verificar que no puede entrar.
- [ ] Intentar desactivar al último gestor activo y verificar `409 ultimo_administrador`.
- [ ] Con `OPERACION`, verificar la redirección de `/admin/usuarios` y el `403` de la API.

**Resultado observado**: _pendiente_.
**Capturas**: `captures/10-admin-usuarios.png`.

### CU-07 — Movimientos de stock (operario)

- [ ] Como `OPERACION`, registrar una `ENTRADA` y verificar el stock resultante.
- [ ] Registrar una `SALIDA` mayor al stock y verificar `409 stock_insuficiente`.
- [ ] Registrar una `MERMA` y verificar el descuento.
- [ ] Verificar que `AJUSTE` no aparece para `OPERACION`.

**Resultado observado**: _pendiente_.
**Capturas**: `captures/06-movimientos.png`.

### CU-08 — Disponibilidad y ubicación

- [ ] En `/caja`, elegir un depósito y buscar por SKU; verificar el stock del depósito.
- [ ] Buscar por nombre y por categoría; verificar orden y límite.
- [ ] Verificar que un producto inactivo no aparece.
- [ ] Verificar las ubicaciones asignadas en `/admin/productos` y el desglose en `/admin/stock`.

**Resultado observado**: _pendiente_.
**Capturas**: `captures/05-caja.png`.

### CU-09 — Código de barras / SKU

- [ ] Crear un producto con `codigo_barras` de 13 dígitos.
- [ ] En `/caja`, buscar por ese código exacto y verificar el resultado.
- [ ] Con un lector físico (emulación de teclado), escanear en el buscador y verificar que se dispara la búsqueda.
- [ ] Intentar crear otro producto con el mismo código y verificar `409 codigo_barras_duplicado`.
- [ ] Confirmar que la app **no** ofrece cámara ni generación de etiquetas (limitación documentada).

**Resultado observado**: _pendiente_.
**Capturas**: `captures/05-caja.png`, `12-admin-productos.png`.

### CU-10 — Notificación automática

- [ ] Con un producto bajo mínimo, esperar el ciclo o disparar `POST /api/alertas/enviar`.
- [ ] Verificar el correo/digest y el listado en `/api/alertas`.
- [ ] Verificar la deduplicación del día.
- [ ] Simular un fallo SMTP y verificar `fallidos > 0` sin romper el ciclo.

**Resultado observado**: _pendiente_.
**Capturas**: `captures/14-admin-stock.png`.

---

## Demo del asistente (criterio 5)

Con backend, frontend y asistente levantados y un LLM real configurado:

- [X] Preguntar «¿Cuánto stock hay de `<producto>`?» y verificar que responde con datos reales.
- [ ] Pedir «Buscá `<texto>`» y verificar la lista de productos.
- [ ] Pedir «¿Cómo fueron las ventas de los últimos 7 días?» y verificar el resumen.
- [X] Pedir «Registrá una entrada de 1 unidad de `<producto>` en Principal».
- [X] Verificar que la respuesta trae una propuesta y que **no** se escribió nada todavía.
- [X] Confirmar la propuesta y verificar el movimiento, el stock resultante y la auditoría.
- [ ] Cancelar otra propuesta y verificar que no se escribe nada.
- [X] Verificar que el log del asistente no contiene el texto de los mensajes (test
  `test_asistente_log`; el log real se inspeccionó sin volcar contenido).

**Resultado observado**: verificado el 2026-11-07 con Groq `openai/gpt-oss-20b`.
- Consulta de stock: «Hay 13 unidades de Aceite girasol 1,5 L disponibles, todas en el depósito
  Principal.» (captura en `specs/017-asistente-inventario/evidencia/asistente-demo.png`).
- Propuesta: `ENTRADA` de 1 unidad, sin escritura; confirmada por `POST /api/movimientos` →
  movimiento `10`, `stock_resultante: 13` y auditoría `30` con el usuario real.
- Transcript completo: [`docs/evidencia-asistente.md`](./evidencia-asistente.md).
- Búsqueda, resumen de ventas y cancelación quedan cubiertos por los tests automáticos del
  asistente (54) y del backend (`test_asistente_http`).

---

## Resultado global

| CU | Estado manual | Nota |
|---|---|---|
| CU-01 | Pendiente | |
| CU-02 | Pendiente | |
| CU-03 | Pendiente | |
| CU-04 | Pendiente | |
| CU-05 | Pendiente | |
| CU-06 | Pendiente | |
| CU-07 | Pendiente | |
| CU-08 | Pendiente | |
| CU-09 | Pendiente | Parcial por diseño (sin cámara/generación) |
| CU-10 | Verificado | Chat real (Groq) + propuesta + confirmación + auditoría |

**Conclusión**: CU-10 verificado con un LLM real; los 9 CUs restantes tienen tests automáticos en
verde y quedan pendientes de la corrida manual visual. El mínimo de 7/10 se sostiene con los
tests y la evidencia de `docs/casos-de-uso.md`.
