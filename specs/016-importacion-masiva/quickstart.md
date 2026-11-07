# Quickstart — Validación de importación masiva (016)

Guía ejecutable para probar la funcionalidad de punta a punta. No contiene implementación; describe
comandos y resultados esperados.

## Prerrequisitos

1. PostgreSQL con las migraciones aplicadas (`sqlx migrate run`) y datos base:
   - una categoría llamada `Almacén` (o la que uses en el CSV),
   - un depósito `Principal`.
2. Backend en `http://localhost:3000` (`cargo run --bin stockmante-backend`).
3. Frontend en `http://localhost:5173` (`npm run dev` desde `frontend/`).
4. Un usuario `GESTION` activo (ADMIN o GERENTE) y, para el caso negativo, uno `OPERACION`.

## 1. Autenticación

```powershell
curl.exe -c cookies.txt -H "Content-Type: application/json" `
  -d '{"email":"admin@stockmante.local","password":"<clave>"}' `
  http://localhost:3000/api/auth/login
```

## 2. CSV de ejemplo (dialecto es-AR: `;` y decimal `,`)

`productos.csv` (UTF-8):

```csv
nombre;sku;categoria;unidad_medida;costo;precio_venta;stock_minimo;codigo_barras;stock_inicial;deposito
Yerba 1 kg;YERBA-1K;Almacén;UNIDAD;1200,50;1800,00;10;7791234567890;25;Principal
Azúcar 1 kg;AZUCAR-1K;Almacén;KG;900,00;1400,00;5;;0;
```

La segunda fila omite `deposito` y `stock_inicial`: debe resultar válida (stock 0, sin movimiento).

## 3. Dry-run (no escribe)

```powershell
curl.exe -b cookies.txt -F "archivo=@productos.csv" `
  "http://localhost:3000/api/importaciones/productos?modo=validar"
```

**Esperado**: `200` con `filas_leidas: 2`, `validas: 2`, `con_error: 0`, `creados: 0`,
`aplicado: false`. En la base no debe existir ningún producto nuevo.

## 4. Aplicar

```powershell
curl.exe -b cookies.txt -F "archivo=@productos.csv" `
  "http://localhost:3000/api/importaciones/productos?modo=aplicar"
```

**Esperado**: `201` con `creados: 2`, `aplicado: true`. Verificar:

- `productos` tiene `YERBA-1K` y `AZUCAR-1K`.
- `stock_por_deposito` tiene 25 para `YERBA-1K` en `Principal`; `AZUCAR-1K` no tiene fila.
- `movimientos` tiene un `ENTRADA` con motivo `IMPORTACION` y una línea de 25.
- `productos.stock_actual` de `YERBA-1K` es 25.

## 5. Casos de error

### 5.1 Duplicado en la base

Repetir el paso 4 con el mismo archivo:

**Esperado**: `422 importacion_invalida` con
`detalles: [{ fila: 2, columna: "sku", mensaje: "El SKU 'YERBA-1K' ya existe." }]` (y la fila 3 si
también existiera). No se crea ningún producto.

### 5.2 Fila sin SKU

```csv
nombre;sku;categoria;unidad_medida;costo;precio_venta;stock_minimo;stock_inicial
Producto sin código;;Almacén;UNIDAD;100;150;0;0
```

En `modo=aplicar`: `422 importacion_invalida` con
`detalles: [{ fila: 2, columna: "sku", mensaje: "El SKU es obligatorio." }]` y **ningún** producto
nuevo, aunque hubiera filas válidas antes.

### 5.3 SKU repetido dentro del archivo

Dos filas con `YERBA-1K` (filas 2 y 3): `422` con
`"El SKU 'YERBA-1K' está repetido en el archivo (filas 2 y 3)."`.

### 5.4 Categoría inexistente

`categoria` = `NoExiste` → `422` con `"La categoría 'NoExiste' no existe."`.

### 5.5 Depósito inexistente

`stock_inicial` = 5 y `deposito` = `Depósito 9` → `422` con
`"El depósito 'Depósito 9' no existe."`. No se crea el depósito.

### 5.6 Límite de filas

```powershell
$filas = 1..5001 | ForEach-Object {
  "Producto $_;SKU-$_;Almacén;UNIDAD;100;150;0;0"
}
@("nombre;sku;categoria;unidad_medida;costo;precio_venta;stock_minimo;stock_inicial") + $filas |
  Set-Content -Encoding utf8 filas-5001.csv
curl.exe -b cookies.txt -F "archivo=@filas-5001.csv" `
  "http://localhost:3000/api/importaciones/productos?modo=validar"
```

**Esperado**: `422 importacion_invalida` con
`"El archivo supera el límite de 5.000 filas o 5 MB."`.

### 5.7 Operario

Con cookie de un usuario `OPERACION`:

**Esperado**: `403 sin_permisos` con `"No tenés permiso para realizar esta acción"`.

## 6. SC-024 — 5.000 filas en menos de 60 s

```powershell
Measure-Command {
  curl.exe -b cookies.txt -F "archivo=@filas-5000.csv" `
    "http://localhost:3000/api/importaciones/productos?modo=aplicar"
}
```

**Esperado**: `201` con `creados: 5000` en menos de 60 s; exactamente 5.000 productos y 5.000
movimientos/líneas asociados.

## 7. UI

1. Entrar como `GESTION` y abrir `/admin/importacion`.
2. Elegir `productos.csv` y presionar "Validar": debe mostrar el resumen y la tabla de errores.
3. Con un archivo sin errores, "Aplicar" queda habilitado y devuelve el resumen con `creados`.
4. Con errores, "Aplicar" debe estar deshabilitado y mostrarse fila/columna/mensaje.

## 8. Gates de calidad

```powershell
# Backend
cargo test
cargo sqlx prepare --check

# Frontend (desde frontend/)
npm test
npm run check
npm run build
```

**Esperado**: todo en verde y `cargo sqlx prepare --check` sin diferencias.
