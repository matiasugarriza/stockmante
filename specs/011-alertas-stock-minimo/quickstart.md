# Quickstart: validar alertas de stock mínimo (011)

Guía de validación end-to-end. Los detalles de contrato están en `contracts/api.md` y el esquema en
`data-model.md`; no se repiten acá.

## Prerrequisitos

- PostgreSQL con las migraciones 0001–0014 aplicadas y `DATABASE_URL` exportada.
- `backend/` compila: `cargo build`.
- Al menos un usuario `GESTION` activo (`ADMIN` o `GERENTE`) y un `OPERACION` activo.
- `ALERTAS_INTERVALO_MINUTOS` sin definir (= 60) o explícita.

## 1. Migración

```bash
sqlx migrate run
psql "$DATABASE_URL" -c "\d notificaciones_stock"
```

Esperado: la tabla existe con `uq_notificacion_producto_usuario_fecha`, `ck_notificacion_enviada_con_fecha`
e `idx_notificaciones_pendientes`.

## 2. Detección (11.1, 11.2, 11.4)

Con sesión `GESTION`:

```bash
curl -s -b "stockmante_sesion=$COOKIE" "$API/api/alertas" | jq
```

Esperado, para un producto con `stock_minimo = 10` y `stock_actual = 4`:
`{"producto_id":…,"stock_actual":4,"stock_minimo":10,"faltante":6}`.

- Subir el stock a 12 (movimiento de entrada): el producto desaparece de la lista.
- Un producto con `stock_minimo = 0` y `stock_actual = 0` nunca aparece.

## 3. Permisos (11.7)

```bash
curl -s -o /dev/null -w '%{http_code}\n' -b "stockmante_sesion=$COOKIE_OPERACION" "$API/api/alertas"
```

Esperado: `403`. Igual para `POST /api/alertas/enviar`.

## 4. Ciclo manual (11.5, 11.3)

```bash
curl -s -b "stockmante_sesion=$COOKIE" -X POST "$API/api/alertas/enviar" | jq
```

Esperado: `productos` = cantidad de productos bajo mínimo, `destinatarios` = usuarios `GESTION`
activos, `enviados` = productos × destinatarios.

Con `CorreoDev` (sin `SMTP_URL`) revisar la bandeja del proceso o los logs: **un** correo por
usuario con todos los productos.

Repetir el `POST` el mismo día: `enviados: 0` y ningún correo nuevo (SC-019).

## 5. Fallo de correo (11.6)

Levantar el backend apuntando a un SMTP inválido (`SMTP_URL=smtp://127.0.0.1:1`) o usar un test con
`CorreoQueFalla`:

```bash
curl -s -b "stockmante_sesion=$COOKIE" -X POST "$API/api/alertas/enviar" | jq
```

Esperado: HTTP 200 con `fallidos > 0`; en la BD las filas quedan `estado = 'fallida'`,
`error` no nulo e `intentos = 1`. Restaurado el SMTP, el ciclo siguiente envía solo esas filas y
las pasa a `enviada`, sin duplicar las ya enviadas.

## 6. Scheduler

```bash
ALERTAS_INTERVALO_MINUTOS=1 cargo run
```

Esperado en logs: un ciclo por minuto. Con `ALERTAS_INTERVALO_MINUTOS=0` no corre ningún ciclo
automático (el `POST` manual sigue funcionando). Un valor no numérico impide arrancar con error de
configuración.

## 7. Frontend

```bash
cd frontend && npm run dev
```

- Con sesión `GESTION`, el badge de `Stock` en el encabezado muestra el conteo de alertas y se
  actualiza al navegar (sin polling).
- `/admin/stock` con "Bajo mínimo" activo muestra mínimo, stock y faltante.
- Con sesión `OPERACION` no hay badge ni enlace, y ninguna llamada a `/api/alertas`.

## 8. Suite

```bash
cd backend && cargo test
cd backend && cargo sqlx prepare --check
cd frontend && npm test && npm run check && npm run build
```

Esperado: todo en verde; `sqlx prepare --check` sin diferencias (incluye las queries nuevas de
`services/alertas.rs`).
