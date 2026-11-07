# Plan de implementación: Proveedores y compras (012)

**Branch**: `012-proveedores-compras` | **Fecha**: 2026-11-07 | **Spec**: [spec.md](./spec.md)
**Migración**: `migrations/0013_proveedores_compras.sql` (aplicada; la fase 6 no agrega
migraciones) · **Research**: [research.md](./research.md) · **Modelo**: [data-model.md](./data-model.md)
· **Contratos**: [contracts/api.md](./contracts/api.md) · **Validación**: [quickstart.md](./quickstart.md)

## Summary

Proveedores con CRUD y baja lógica; compras en `BORRADOR` editables, cuya confirmación genera en
una transacción una `ENTRADA` (motivo `COMPRA`) y actualiza stock y costo, y cuya anulación revierte
con una `SALIDA` (motivo `DEVOLUCION`). Todo exige `GESTION`. T001–T023 ya están implementadas.

Este plan **reemplaza** al anterior, que describía tablas (`ordenes_compra`, `orden_compra_lineas`,
`recepciones`) y un endpoint (`/recibir`) que no existen. Además agrega la fase 6, que cierra los
hallazgos del análisis del 2026-11-07:

- C3: anulación concurrente sin bloqueo ni guarda condicional (puede terminar en 500).
- C4: la confirmación y la anulación no muestran el resultado concreto.
- U1/U4/U5: códigos de error y filtros definidos en las clarificaciones.
- R5: límites de `numero` y `costo_unitario`, que hoy pueden terminar en 500.

## Technical Context

**Language/Version**: Rust 2021, backend `stockmante-backend`; TypeScript + Svelte 5 (runes) en
`frontend/`.

**Primary Dependencies**: axum 0.8, sqlx 0.8, rust_decimal, chrono, utoipa (ya presentes).
**Ninguna dependencia nueva.**

**Storage**: PostgreSQL. Tablas `proveedores`, `compras`, `compra_lineas` y `movimientos.compra_id`
(0013), más `movimientos`, `movimiento_lineas` y `stock_por_deposito` (0009).

**Testing**: `cargo test` (unit + integración registrados en `[[test]]` de `backend/Cargo.toml`),
`cargo sqlx prepare --check`; frontend `vitest` + `npm run check`.

**Target Platform**: servidor con PostgreSQL; SPA en navegador.

**Project Type**: web application (backend Rust + frontend SvelteKit).

**Performance Goals**: sin objetivo de throughput; una compra tiene decenas de líneas.

**Constraints**: cero 500 por concurrencia o por datos fuera de rango (SC-021, R5); stock nunca
negativo; una confirmación genera exactamente una `ENTRADA` (SC-020).

**Scale/Scope**: decenas a cientos de compras por mes; ~10⁴ productos.

## Constitution Check

*GATE: pasa antes de Phase 0; se re-evalúa después de Phase 1.*

| Principio | Cumplimiento | Nota |
|---|---|---|
| I. Simplicidad | PASS | Sin capas nuevas. Se reusan `MovimientoDetalle` y `movimientos::obtener` de 007 en lugar de un DTO paralelo (R2). Sin migraciones ni dependencias. |
| II. Mantenibilidad | PASS | Lógica en `services::compras`; validaciones puras en `models::proveedor`; handlers delgados; frontend vía `$lib/api/compras.ts`. |
| III. Claridad de mensajes | **FAIL → corregido por T031–T034** | Hoy la UI confirma sin stock resultante (C4). Con `ResultadoCompra` la UI muestra depósito, producto y saldo. Códigos nuevos con mensaje que dice qué hacer (R3). |
| IV. Testabilidad | **FAIL → waiver W-003 pendiente (T025)** | T006/T020/T023 sin RED registrado (C1). Desde la fase 6, RED antes de GREEN en cada tarea; `parsear_fecha` y las validaciones quedan puras (R4, R5). |
| V. Integridad de datos | **FAIL → corregido por T026/T027** | `anular` valida sin bloqueo y descuenta sin guarda condicional (C3). Se adopta el orden de bloqueo de 007 y `cantidad >= n` (R1). Errores de dominio con variante propia (R3). |
| Verificación de esquema | PARCIAL | Registrada contra `migrations/` en `data-model.md`; T024 la repite contra la BD real antes de tocar SQL (C2). |
| Flujo SDD | PASS | Spec aclarada (5 respuestas, 2026-11-07) → plan → tasks → implementación. |

**Post-diseño (Phase 1)**: con T024–T036 aplicadas no quedan violaciones abiertas, salvo la
aprobación de W-003, que corresponde a `/speckit constitution`.

## Project Structure

### Documentation (this feature)

```text
specs/012-proveedores-compras/
├── spec.md          # aclarada 2026-11-07
├── plan.md          # este archivo
├── research.md      # R1–R8
├── data-model.md    # verificación de esquema, reglas, estados, DTOs
├── contracts/
│   └── api.md       # superficie completa + cambios de la fase 6
├── quickstart.md
└── tasks.md
```

### Source Code (repository root)

```text
migrations/
└── 0013_proveedores_compras.sql          # existente, sin cambios

backend/src/
├── app.rs                                # nest /api/proveedores y /api/compras (existente)
├── error.rs                              # + CompraNoAnulable, ProveedorInactivo
├── docs.rs                               # + schema ResultadoCompra
├── models/
│   ├── proveedor.rs                      # Proveedor, Compra, CompraLinea, CompraDetalle, ListaCompras
│   │                                     # + ResultadoCompra, validar_numero, validar_costo, validar_estado
│   └── stock.rs                          # MovimientoDetalle (007, reutilizado)
├── services/
│   ├── proveedores.rs                    # CRUD (existente)
│   └── compras.rs                        # crear/actualizar/eliminar/confirmar/anular (cambia anular, confirmar, validar_destinos)
└── handlers/
    ├── proveedores.rs                    # RequiereGestion (existente)
    └── compras.rs                        # parsear_fecha → Result; confirmar/anular → ResultadoCompra

backend/tests/
├── unit/test_compras_validacion.rs       # nueva: numero, costo, estado, fechas
├── integration/test_compras_http.rs      # + 12.2b, 12.5b, 12.8, ResultadoCompra
├── integration/test_compras_concurrencia.rs  # nueva: SC-021
└── unit/test_openapi.rs                  # + ResultadoCompra

frontend/src/
├── lib/schemas/compras.ts (+ test)       # + ResultadoCompraSchema (reusa el schema de movimiento)
├── lib/api/compras.ts (+ test)           # confirmar/anular → ResultadoCompra
├── lib/components/compras/               # CompraForm (rótulo nº factura/remito), CompraTabla («Compra #id»)
└── routes/admin/compras/+page.svelte     # aviso con depósito, producto y stock resultante

docs/api.md                               # códigos nuevos y forma de respuesta
```

**Structure Decision**: capas `models → services → handlers` como en 007–016. Los modelos de compra
viven en `models/proveedor.rs` (no existe `compra.rs`, como decía el plan anterior); no se dividen
ahora porque el archivo sigue acotado al dominio de abastecimiento.

## Decisiones de diseño

1. **Esquema real** (I1): `proveedores`, `compras`, `compra_lineas`, `movimientos.compra_id`. Sin
   tabla de recepciones: la confirmación es la recepción.
2. **Endpoints reales** (I2): ver [contracts/api.md](./contracts/api.md); `confirmar` y `anular` en
   lugar de `recibir`.
3. **Último costo** (YAGNI): `productos.costo = costo_unitario` al confirmar; la anulación no lo
   revierte.
4. **Orden de bloqueo único** (R1, A1): `compras` → `depositos` → `productos ORDER BY id` → filas
   de `stock_por_deposito`, igual que `movimientos::crear`. Descuento con `cantidad >= n`.
5. **`ResultadoCompra`** (R2): respuesta de confirmar y anular con el `MovimientoDetalle` de 007.
6. **Errores** (R3): `compra_no_anulable` y `proveedor_inactivo` (409); `compra_no_editable` solo
   para editar y eliminar.
7. **Validación temprana** (R4, R5): filtros, `numero` y `costo_unitario` se validan antes de
   abrir la transacción.
8. **Sin ubicación** (R8): la recepción imputa al depósito.

## Estrategia de pruebas

Principio IV: **RED antes de GREEN** en cada tarea de la fase 6; cada test nuevo se corre primero
contra el código actual y se registra su fallo en `progress.md`.

| Caso | Tipo | Assert |
|---|---|---|
| `numero` de 41 caracteres, costo 10¹⁰ | unit + HTTP | 422 `validacion` (hoy 500) |
| `estado=PAGADA`, `desde=07-11-2026` | unit + HTTP | 422 `validacion` (hoy 200 sin filtrar) |
| proveedor inactivo al crear/editar/confirmar | HTTP | 409 `proveedor_inactivo` (hoy 422) |
| anular un borrador / anular dos veces | HTTP | 409 `compra_no_anulable` (hoy `compra_no_editable`) |
| confirmar / anular | HTTP | 200 con `movimiento.lineas[].stock_resultante` correcto |
| anulación ‖ `SALIDA` de 007 | integración concurrente | una 200 y otra 409, ningún 500, saldo ≥ 0 (SC-021) |
| SC-020 | HTTP | exactamente 1 `ENTRADA` con `compra_id` tras confirmar |
| frontend | vitest | parseo de `ResultadoCompra`; el aviso arma texto con depósito, producto y saldo |

## Riesgos

- **Interbloqueos** si `anular` bloquea en otro orden que 007. Mitigado con el orden único (decisión
  4) y con el test concurrente.
- **Clientes del JSON de confirmar/anular**: el cambio es aditivo (`movimiento` se agrega); el
  asistente (017) no confirma compras.
- **Waiver W-003**: si no se aprueba, el Principio IV sigue en falta para T006/T020/T023; no se
  fabrica un RED artificial.
- **Verificación contra la BD real** pendiente (T024): las migraciones podrían no reflejar un cambio
  manual en la base.

## Complexity Tracking

Sin violaciones que justificar. `ResultadoCompra` es un DTO nuevo, pero compone tipos existentes en
lugar de duplicarlos.

## Fuera de alcance (confirmado por la spec)

Órdenes con aprobación, facturación e impuestos, recepción parcial, costo promedio o FIFO, y
elegir ubicación al recibir.
