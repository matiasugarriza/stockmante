# Quickstart: validación de ventas de caja y clientes (013)

Guía de validación end-to-end. Los contratos exactos están en
[`contracts/api.md`](./contracts/api.md) y el esquema en [`data-model.md`](./data-model.md).

## Prerrequisitos

- PostgreSQL en ejecución con `.env` (`DATABASE_URL`) y migraciones aplicadas hasta
  `migrations/0014_ventas.sql`.
- Usuario `GESTION` (por ejemplo el admin de `cargo run --bin bootstrap`) y un usuario
  `OPERACION`/`CAJERO` para probar permisos.
- Al menos un depósito activo ("Principal") con dos productos activos con stock y
  `precio_venta`.

## Levantar la aplicación

```bash
# backend
cargo run --bin stockmante-backend

# frontend (otra terminal)
cd frontend
npm install          # solo la primera vez
npm run dev
```

Backend en `http://localhost:3000`, frontend en `http://localhost:5173` (o el puerto que reporte
Vite).

## Escenario 1 — Venta completa y precio congelado (13.1, 13.5)

```bash
B=http://localhost:3000
curl -c gestor.txt -s -X POST $B/api/auth/login \
  -H 'Content-Type: application/json' \
  -d '{"email":"admin@stockmante.local","password":"<password>"}'

# crear un cliente (GESTION)
curl -b gestor.txt -s -X POST $B/api/clientes -H 'Content-Type: application/json' \
  -d '{"nombre":"Ana Pérez","documento":"30111222"}'

# vender (cajero u operario; el número de venta debe ser V-000001, V-000002…)
curl -b cajero.txt -s -X POST $B/api/ventas -H 'Content-Type: application/json' \
  -d '{"deposito_id":1,"medio_pago":"EFECTIVO","lineas":[{"producto_id":12,"cantidad":2}]}'
```

Esperado: **201** con `numero`, `total` y `lineas[].stock_resultante`; `GET /api/ventas/{id}`
sigue mostrando el `precio_unitario` original aunque después cambie `productos.precio_venta`.

## Escenario 2 — Stock insuficiente y atomicidad (13.2, 13.3)

Con un producto de 2 unidades, vender 3 devuelve **409** `stock_insuficiente` con
`"Stock insuficiente en Principal: 2 disponibles, 3 solicitadas. Reducí la cantidad o registrá antes una entrada en ese depósito."`. Repetir con una venta de tres
líneas donde la última no alcanza: el 409 no deja venta, líneas, movimiento ni cambio de stock.

## Escenario 3 — Producto repetido en el carrito (13.9)

```bash
# stock 2: dos líneas de 1 + 1 → 201, una sola línea de cantidad 2, stock 0
curl -b cajero.txt -s -X POST $B/api/ventas -H 'Content-Type: application/json' \
  -d '{"deposito_id":1,"medio_pago":"EFECTIVO","lineas":[{"producto_id":12,"cantidad":1},{"producto_id":12,"cantidad":1}]}'

# stock 2: dos líneas de 2 + 1 → 409 stock_insuficiente "2 disponibles, 3 solicitadas."
```

## Escenario 4 — Precio enviado por el cliente (FR-050)

```bash
curl -b cajero.txt -s -X POST $B/api/ventas -H 'Content-Type: application/json' \
  -d '{"deposito_id":1,"medio_pago":"EFECTIVO","lineas":[{"producto_id":12,"cantidad":1,"precio_unitario":1}]}'
```

Esperado: **422** `validacion` con
`"El precio unitario se toma del catálogo; no se puede enviar en la venta."`.

## Escenario 5 — Anulación y permisos (13.6, FR-055)

```bash
# con sesión OPERACION → 403 sin_permisos
curl -b cajero.txt -s -o /dev/null -w '%{http_code}\n' -X POST $B/api/ventas/1/anular

# con sesión GESTION → 200, estado ANULADA y movimiento ENTRADA/DEVOLUCION
curl -b gestor.txt -s -X POST $B/api/ventas/1/anular

# repetir → 409 venta_ya_anulada "La venta ya está anulada; no se puede anular dos veces."
```

Escribir clientes con sesión `OPERACION` debe dar 403; listarlos, 200. Un documento repetido
responde **409** `documento_duplicado` con `"Ya existe un cliente con ese documento."`.

## Escenario 6 — Interfaz (FR-054)

1. `/caja`: buscar un producto, agregarlo al carrito dos veces (debe fusionarse en una línea),
   aplicar descuento, elegir medio de pago y confirmar; el ticket muestra número, total y stock
   resultante.
2. `/admin/ventas`: listar con filtros, abrir el detalle y anular con un usuario `GESTION`.
3. `/admin/clientes`: alta, edición, baja lógica y reactivación; un documento duplicado muestra
   el mensaje 409.
4. Los enlaces "Ventas" y "Clientes" aparecen solo para `GESTION`.

## Pruebas automatizadas (quality gates)

```bash
cargo test
cargo sqlx prepare --check

cd frontend
npm test
npm run check
npm run build
```

Esperado: suite completa en verde; sin `unwrap`/`panic!` en rutas de producción; `cargo sqlx
prepare --check` sin diferencias.
