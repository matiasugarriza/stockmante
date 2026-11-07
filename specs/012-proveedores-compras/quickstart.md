# Quickstart: validar proveedores y compras (012)

Guía de validación de punta a punta. Contratos en [contracts/api.md](./contracts/api.md); reglas y
estados en [data-model.md](./data-model.md).

## Prerrequisitos

- PostgreSQL con migraciones hasta `0013` aplicadas (`DATABASE_URL` en `.env`).
- Backend y frontend levantados; un usuario `GERENTE` (o `ADMIN`) y uno `OPERARIO`.
- Si el backend está corriendo, usar un `CARGO_TARGET_DIR` aparte para `cargo test` (el binario en
  uso bloquea `target/`).

## Gates

```bash
cd backend && cargo test --test test_compras_http --test test_rbac_matriz
```

```bash
cd backend && cargo sqlx prepare --check
```

```bash
cd frontend && npm test -- compras proveedores && npm run check && npm run build
```

## Escenarios manuales (UI `/admin/proveedores` y `/admin/compras`)

| # | Pasos | Resultado esperado |
|---|---|---|
| 12.1 | Crear el proveedor «Ferretería Norte»; repetir el nombre; repetir el CUIT en otro proveedor | 201; 409 `proveedor_duplicado`; 409 `cuit_duplicado` |
| 12.2 | Compra con 2 líneas y nº de remito | `BORRADOR`, «Compra #id», total = Σ subtotales, stock sin cambios |
| 12.2 b | Dar de baja el proveedor e intentar confirmar | 409 `proveedor_inactivo` con el nombre del proveedor |
| 12.3 | Reactivar y confirmar | `CONFIRMADA`; el aviso muestra depósito, cada producto y su stock resultante |
| 12.4 | Confirmar otra vez (API) | 409 `compra_ya_confirmada`, stock igual |
| 12.5 | Anular | `ANULADA`; el aviso muestra el stock resultante tras la devolución |
| 12.5 b | Anular un borrador; anular dos veces | 409 `compra_no_anulable` con el mensaje de cada caso |
| 12.6 | Editar y eliminar un borrador; editar una confirmada | permitido; 409 `compra_no_editable` |
| 12.7 | Entrar con `OPERARIO` | 403 `sin_permisos` |
| 12.8 | `GET /api/compras?desde=07-11-2026` y `?estado=PAGADA` | 422 `validacion` nombrando el parámetro |
| R5 | Línea con costo 12345678901,00 o nº de remito de 41 caracteres | 422 `validacion`, nunca 500 |

## Concurrencia (SC-021)

Lo cubre el test de integración de T026: una compra confirmada de 10 unidades sin otro stock en el
depósito, y en paralelo una anulación y una `SALIDA` de 007 por 5 unidades. Se espera que una de las
dos responda 200 y la otra 409 `stock_insuficiente`, sin ningún 500, y que el saldo final sea ≥ 0.
