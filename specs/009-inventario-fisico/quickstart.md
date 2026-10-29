# Quickstart: Inventario físico por conteo (009)

## Prerrequisitos

- Base de datos `stockmante` y `stockmante_test` con migración `0011` aplicada.
- Backend compilado.
- Al menos un depósito activo con stock de un producto activo.

## Escenarios de validación

### 1. Abrir un conteo (Escenario 9.1)

```bash
curl -X POST http://localhost:3000/api/conteos \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{ "deposito_id": 1 }'
```

**Esperado**: 201 con estado `ABIERTO` y líneas con `cantidad_sistema` y `cantidad_contada: null`.

### 2. Cargar cantidades (Escenario 9.2)

```bash
curl -X PATCH http://localhost:3000/api/conteos/1/lineas \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{ "lineas": [ { "linea_id": 1, "cantidad_contada": 8 } ] }'
```

**Esperado**: 200 con diferencia calculada y conteo sigue `ABIERTO`.

### 3. Cerrar con diferencias (Escenario 9.3)

```bash
curl -X POST http://localhost:3000/api/conteos/1/cerrar \
  -H "Authorization: Bearer $TOKEN"
```

**Esperado**: 200, estado `CERRADO`, un movimiento `AJUSTE` generado, stock = contado.

### 4. Cerrar sin diferencias (Escenario 9.4)

**Esperado**: 200, estado `CERRADO`, sin movimiento.

### 5. Conteo cerrado (Escenario 9.5)

**Esperado**: 409 `conteo_cerrado`.

### 6. Cancelar (Escenario 9.6)

```bash
curl -X POST http://localhost:3000/api/conteos/1/cancelar \
  -H "Authorization: Bearer $TOKEN"
```

**Esperado**: 200, estado `CANCELADO`, sin ajuste.

### 7. Operario (Escenario 9.7)

**Esperado**: 403 `sin_permisos`.

## Tests

```bash
cd backend && cargo test
cd frontend && npm test
```

## Quality gates

```bash
cd backend && cargo sqlx prepare --check
cd frontend && npm run check && npm run build
```