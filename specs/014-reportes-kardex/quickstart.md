# Quickstart: validación de reportes y kardex (014)

Guía de validación end-to-end. Contratos en [`contracts/api.md`](./contracts/api.md); tipos y
errores en [`data-model.md`](./data-model.md).

## Prerrequisitos

- PostgreSQL con `.env` (`DATABASE_URL`) y migraciones aplicadas hasta
  `migrations/0014_ventas.sql`.
- Usuario `GESTION` (admin de `cargo run --bin bootstrap`) y un usuario `OPERACION`.
- Un producto con, en octubre 2026: una entrada por compra (012), una venta (013), una merma, un
  ajuste y una transferencia entre dos depósitos (008); una venta anulada.

## Levantar la aplicación

```bash
cargo run --bin stockmante-backend
```

```bash
cd frontend && npm run dev
```

## Escenarios (curl con la cookie de sesión en `$C`)

| # | Comando | Resultado esperado |
|---|---|---|
| 14.1 | `curl -b "$C" "localhost:3000/api/reportes/kardex?producto_id=1&desde=2026-10-01&hasta=2026-10-31"` | `saldo_inicial + Σ cantidad = saldo_final`; incluye movimientos del 31/10 hasta las 23:59 locales |
| SC-022 | igual, sin `desde`/`hasta` | `saldo_final` = `stock_actual` del producto (`GET /api/productos/1`) |
| 14.2 | `curl -b "$C" localhost:3000/api/reportes/stock-valorizado` | sin productos en cero; `total` = Σ `valor` |
| 14.3 | `curl -b "$C" "localhost:3000/api/reportes/rotacion?desde=2026-10-01&hasta=2026-10-31"` | `unidades_salidas` no cuenta la transferencia ni la merma |
| 14.4 | `curl -b "$C" "localhost:3000/api/reportes/ventas?desde=2026-10-01&hasta=2026-10-31"` | la venta anulada no aparece; días en hora local |
| 14.5 | agregar `&formato=csv` a cada uno (ventas también con `&agrupar=producto`) | `text/csv; charset=utf-8`, descarga con nombre, abre bien en Excel |
| 14.6 | `...rotacion?desde=2026-11-01&hasta=2026-10-01` | 422 `La fecha desde no puede ser posterior a la fecha hasta.` |
| borde | `...rotacion?desde=2026-10-01` | 422 `Indicá las fechas desde y hasta…` |
| borde | `...ventas?desde=01/10/2026&hasta=2026-10-31` | 422 formato ISO 8601 |
| 14.7 | cualquiera con la cookie de `OPERACION` | 403 `sin_permisos` |

## UI

1. Entrar como `GESTION` → enlace **Reportes** → `/admin/reportes`.
2. Elegir cada reporte, completar fechas, ver la tabla y descargar el CSV.
3. Con `desde > hasta` la pantalla muestra el mensaje del backend.
4. Como `OPERACION`, el enlace no aparece y `/admin/reportes` muestra el 403.

## Rendimiento

`EXPLAIN ANALYZE` de las consultas de kardex y rotación sobre la BD de desarrollo; agregar un
índice solo si aparece un `Seq Scan` costoso sobre `movimiento_lineas` o `movimientos`.

## Gates

```bash
cargo test
```

```bash
cargo sqlx prepare --check
```

```bash
cd frontend && npm test && npm run check && npm run build
```
