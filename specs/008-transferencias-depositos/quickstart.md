# Quickstart: Transferencias entre depósitos (008)

## Prerrequisitos

- Base de datos `stockmante` y `stockmante_test` con migración `0010` aplicada.
- Backend compilado (`cargo build`).
- Frontend con dependencias instaladas (`npm install`).
- Al menos dos depósitos activos con stock de un producto activo.

## Aplicar migración

```bash
cd backend
cargo sqlx migrate run --source ../migrations
DATABASE_URL=postgres://.../stockmante_test cargo sqlx migrate run --source ../migrations
```

## Escenarios de validación

### 1. Transferencia exitosa (Escenario 8.1)

```bash
# POST /api/transferencias
curl -X POST http://localhost:3000/api/transferencias \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "producto_id": 1,
    "deposito_origen_id": 1,
    "deposito_destino_id": 2,
    "cantidad": 8
  }'
```

**Esperado**: 201 con `stock_origen_resultante` y `stock_destino_resultante` correctos.

### 2. Stock insuficiente (Escenario 8.2)

```bash
curl -X POST http://localhost:3000/api/transferencias \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "producto_id": 1,
    "deposito_origen_id": 1,
    "deposito_destino_id": 2,
    "cantidad": 5
  }'
```

**Esperado**: 409 `stock_insuficiente` con mensaje literal; ningún depósito cambia.

### 3. Mismo depósito (Escenario 8.3)

```bash
curl -X POST http://localhost:3000/api/transferencias \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "producto_id": 1,
    "deposito_origen_id": 1,
    "deposito_destino_id": 1,
    "cantidad": 5
  }'
```

**Esperado**: 422 `transferencia_mismo_deposito`.

### 4. Depósito destino inactivo (Escenario 8.4)

```bash
curl -X POST http://localhost:3000/api/transferencias \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "producto_id": 1,
    "deposito_origen_id": 1,
    "deposito_destino_id": 3,
    "cantidad": 5
  }'
```

**Esperado**: 422 `validacion`; nada escrito.

### 5. Consulta (Escenario 8.5)

```bash
# Detalle
curl http://localhost:3000/api/transferencias/1 \
  -H "Authorization: Bearer $TOKEN"

# Listado con filtros
curl "http://localhost:3000/api/transferencias?producto_id=1&deposito_origen_id=1" \
  -H "Authorization: Bearer $TOKEN"
```

**Esperado**: 200 con origen, destino, cantidad, autor, fecha y los dos movimientos generados.

## Tests

```bash
# Backend
cd backend
cargo test

# Frontend
cd frontend
npm test
```

## Quality gates

```bash
cd backend
cargo sqlx prepare --check

cd ../frontend
npm run check
npm run build
```
