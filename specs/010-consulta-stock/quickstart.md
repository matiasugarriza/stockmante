# Quickstart: validar la consulta de stock (010)

## Prerrequisitos

- BD con migraciones aplicadas hasta la última (`sqlx migrate run`), backend y frontend corriendo.
- Un usuario `GESTION` (ADMIN o GERENTE) y uno `OPERACION` (OPERARIO o CAJERO); dos depósitos
  activos; un producto con `stock_minimo = 10` y stock 3 repartido en los dos depósitos.

## Automático

```bash
cd backend && cargo test --test test_stock_http && cargo test --test test_stock_item
cd backend && cargo sqlx prepare --check
cd frontend && npm test && npm run check && npm run build
cd asistente && npm test
```

SC-018 (lento, a demanda):

```bash
cd backend && cargo test --test test_stock_http -- --ignored sc018 --nocapture
```

Esperado: cada variante (sin `q`, `q` por nombre, `q` por SKU) por debajo de 1 s; si no, ver
research R6.

## Manual

1. Como `GESTION`, `GET /api/stock?bajo_minimo=true` → el producto aparece con `faltante: 7`,
   `costo` y `valorizado` numéricos ([contrato](contracts/api.md)).
2. Como `OPERACION`, la misma consulta → `costo: null`, `valorizado: null`.
3. `/admin/stock` como `GESTION`: filtrar por depósito, categoría, bajo mínimo y activo; ver
   faltante y valorizado; abrir el desglose del producto y comprobar que suma el total.
4. `/caja` como `OPERACION`: elegir un depósito, buscar el producto y ver "Stock en <depósito>";
   cambiar de depósito y ver el valor nuevo; no hay columnas de costo.
5. Cortar el backend y recargar `/admin/stock`: se ve el mensaje con qué hacer y el botón
   Reintentar.
