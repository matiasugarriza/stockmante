# Progreso de implementación — Ventas de caja (013)

**Spec**: [spec.md](./spec.md) | **Plan**: [plan.md](./plan.md) | **Actualizado**: 2026-11-07 |
**Estado**: implementado (T001–T024); T025 (gates) en el cierre transversal.

## Esquema (T001)

Ver `data-model.md`: esquema real idéntico a `0014_ventas.sql`; la migración quedó registrada.

## Evidencia RED (2026-11-07)

| Test | Antes de implementar |
|---|---|
| `test_venta_validacion` | no compila: `agregar_lineas` no existía y `NuevaVentaLinea` tenía `precio_unitario` |
| `test_ventas_http` | 4/6 FAILED: líneas repetidas no agregadas (stock validado por línea), sin `stock_resultante`, precio enviado aceptado, OPERACION podía anular |
| `test_clientes_http::alta_lectura_permisos_y_documento_duplicado` | FAILED: OPERACION escribía clientes y el duplicado era 422 |
| `test_rbac_matriz` (matriz) | FAILED: anular venta y escribir clientes no respondían 403 a OPERACION |
| `api/ventas.test.ts`, `api/clientes.test.ts` | FAILED: módulos inexistentes |

Verdes en la primera corrida: `precio_congelado_al_confirmar`, `descuento_medio_de_pago_y_cliente`
y `baja_logica_conserva_las_ventas` (comportamiento ya implementado). `utils/carrito.test.ts` se
escribió junto con el módulo (sin RED registrado).

## Cambios

- `models/cliente.rs` (movido desde `venta.rs`); `AppError::DocumentoDuplicado` (409).
- `services/ventas::crear`: `agregar_lineas`, precio del catálogo dentro de la transacción, orden de
  bloqueo de 007 (depósito → productos) y descuento condicional (`UPDATE … AND cantidad >= $n`): sin
  saldo no se escribe nada y nunca hay 500 por el `CHECK` de stock.
- `stock_resultante` por línea = saldo del movimiento más reciente de la venta (venta o anulación).
- `anular_venta` y escrituras de clientes con `RequiereGestion`; OpenAPI con 403/409.
- UI: POS en `/caja` (carrito con fusión por producto, descuento validado, medio de pago, cliente
  opcional, resultado con stock restante), `/admin/ventas` (filtros, detalle, anulación) y
  `/admin/clientes` (alta, edición, baja y reactivación); enlaces en la navegación de GESTION.

## Resultado

`test_ventas_http` 6/6, `test_clientes_http` 2/2, `test_venta_validacion` 4/4, `test_rbac_matriz`
11/11, `test_openapi` 10/10; frontend 319/319, `svelte-check` 0/0.
