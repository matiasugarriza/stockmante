# Quickstart: Inventario físico por conteo (009)

## Prerrequisitos

- Bases `stockmante` y `stockmante_test` con la migración `migrations/0011_inventario_fisico.sql`
  aplicada.
- Backend compilado (`cd backend && cargo build`).
- Un usuario de familia `GESTION` y otro de familia `OPERACION`.
- Un depósito activo con stock de al menos un producto activo.

## Escenarios de validación

### 1. Abrir un conteo (Escenario 9.1)

```bash
curl -X POST http://localhost:3000/api/conteos \
  -H "Authorization: Bearer $TOKEN_GESTION" \
  -H "Content-Type: application/json" \
  -d '{ "deposito_id": 1, "observacion": "Conteo mensual" }'
```

**Esperado**: 201, estado `ABIERTO`, líneas con `cantidad_sistema` y `cantidad_contada: null`.

### 2. Cargar cantidades (Escenario 9.2)

```bash
curl -X PATCH http://localhost:3000/api/conteos/1/lineas \
  -H "Authorization: Bearer $TOKEN_GESTION" \
  -H "Content-Type: application/json" \
  -d '{ "lineas": [ { "linea_id": 1, "cantidad_contada": 8 } ] }'
```

**Esperado**: 200, `diferencia = cantidad_contada − cantidad_sistema(snapshot)`, estado sigue
`ABIERTO`.

### 3. Cerrar con diferencias (Escenario 9.3)

```bash
curl -X POST http://localhost:3000/api/conteos/1/cerrar \
  -H "Authorization: Bearer $TOKEN_GESTION"
```

**Esperado**: 200, estado `CERRADO`, un movimiento `AJUSTE` (motivo `INVENTARIO`) con líneas solo
para diff ≠ 0, y stock del depósito = lo contado (SC-017).

### 4. Cerrar sin diferencias (Escenario 9.4)

**Esperado**: 200, estado `CERRADO`, `movimiento_ajuste_id` null y sin movimiento nuevo.

### 5. Conteo cerrado (Escenario 9.5)

**Esperado**: cargar o cerrar de nuevo → 409 `conteo_cerrado`.

### 6. Cancelar y operar sobre cancelado (Escenario 9.6)

```bash
curl -X POST http://localhost:3000/api/conteos/1/cancelar \
  -H "Authorization: Bearer $TOKEN_GESTION"
```

**Esperado**: 200, estado `CANCELADO`, sin ajuste. Cargar, cerrar o cancelar de nuevo → 409
`conteo_cancelado`.

### 7. Operario (Escenario 9.7)

**Esperado**: 403 `sin_permisos` en las 6 rutas con `$TOKEN_OPERACION`.

### 8. Líneas sin contar al cerrar

Crear un conteo y cerrarlo sin cargar cantidades.

**Esperado**: 422 `lineas_sin_contar`, estado sigue `ABIERTO`.

### 9. Alta sobre depósito sin stock

```bash
curl -X POST http://localhost:3000/api/conteos \
  -H "Authorization: Bearer $TOKEN_GESTION" \
  -H "Content-Type: application/json" \
  -d '{ "deposito_id": 999 }'
```

**Esperado**: 422 `validacion` con mensaje "El depósito no tiene stock registrado para contar." y
sin conteo creado.

## Tests

```bash
cd backend && cargo test
cd frontend && npm test
```

Archivos esperados: `backend/tests/unit/test_conteo_validacion.rs`,
`backend/tests/integration/test_conteos_http.rs`, `frontend/src/lib/api/conteos.test.ts`,
`frontend/src/lib/schemas/conteos.test.ts`.

## Quality gates

```bash
cd backend && cargo test && cargo sqlx prepare --check
cd frontend && npm test && npm run check && npm run build
```
