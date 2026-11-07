# Tasks: 013-ventas-caja

**Input**: Design documents from `/specs/013-ventas-caja/`

**Prerequisites**: plan.md ✓, spec.md ✓ (clarificada 2026-11-07), research.md ✓, data-model.md ✓,
contracts/api.md ✓

**Tests**: obligatorios (Constitución, Principio IV). Escribir y ver fallar (RED) antes de
implementar; el gate final exige `cargo test`, `cargo sqlx prepare --check`, `npm test`,
`npm run check` y `npm run build` en verde.

**Organización**: una sola historia (US13, P1). Las tareas de backend van antes que las de
frontend porque el POS consume `$lib/api/ventas.ts`.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: puede ejecutarse en paralelo (archivo distinto, sin dependencias pendientes)
- **[Story]**: US13 (única historia de la spec)
- Cada tarea cita el FR/SC que cubre y una ruta de archivo concreta

## Path Conventions

- Backend: `backend/src/`, `backend/tests/`, `migrations/`
- Frontend: `frontend/src/`
- Documentación: `docs/`

---

## Phase 1: Setup

- [ ] T001 Verificar contra la BD real (MCP de PostgreSQL o `psql`) que `clientes`, `ventas`,
  `venta_lineas`, la secuencia `ventas_numero_seq` y `movimientos.venta_id` coinciden con
  `migrations/0014_ventas.sql`, en ambas bases; registrar el resultado en
  `specs/013-ventas-caja/data-model.md` (Principio V, precondición de todo SQL)
- [ ] T002 [P] Verificar que `backend/tests/unit/common/mod.rs` expone `app_de_test`,
  `sesion_gestion`, `sesion_operacion` y `sku_unico` y que los nuevos targets de test se declaran
  como `[[test]]` en `backend/Cargo.toml` (el proyecto declara cada target explícitamente)

## Phase 2: Foundational (bloqueantes)

**⚠️ CRÍTICO**: sin esta fase no se pueden escribir los tests ni completar los servicios.

- [ ] T003 [P] Crear `backend/src/models/cliente.rs` moviendo `Cliente` y `ListaClientes` desde
  `backend/src/models/venta.rs` (con `validar_nombre`, `validar_documento`, `normalizar_opcional`,
  `NOMBRE_MAX` a `DIRECCION_MAX`), y registrar `pub mod cliente;` en `backend/src/models/mod.rs`
  (FR-049, R5)
- [ ] T004 [P] En `backend/src/models/venta.rs` agregar `stock_resultante: Option<i32>` a
  `VentaLinea` (FR-051, Escenario 13.1, R9); en `backend/src/handlers/ventas.rs` conservar
  `precio_unitario: Option<Decimal>` en `VentaLineaDto` solo para poder rechazarlo (FR-050, R1);
  conservar `validar_medio_pago` y `validar_observacion`
- [ ] T005 Agregar `AppError::DocumentoDuplicado` en `backend/src/error.rs` (mensaje `"Ya existe un
  cliente con ese documento."`) mapeada a 409 y al código `documento_duplicado` (FR-049, R4)
- [ ] T006 Usar `DocumentoDuplicado` en el chequeo de duplicados de `backend/src/services/clientes.rs`
  (`crear` y `actualizar`) en lugar de `Validacion` (FR-049, R4)

## Phase 3: User Story 13 — Vender en la caja y descontar stock (Priority: P1) 🎯 MVP

**Goal**: registrar ventas atómicas con stock descontado, anularlas con reposición y administrar
clientes, con permisos correctos y respuesta con stock resultante.

**Independent Test**: `cargo test` de `test_ventas_http`, `test_clientes_http`, `test_rbac_matriz`
y `test_venta_validacion`; y la secuencia curl de `quickstart.md` (13.1–13.9).

### Tests for User Story 13 ⚠️ (escribir primero y ver fallar)

- [ ] T007 [P] [US13] Crear `backend/tests/unit/test_venta_validacion.rs` con casos puros de
  `agregar_lineas` (2+1=3, orden de primera aparición, una sola entrada por producto) y de
  validación de medio de pago y observación; declarar el target en `backend/Cargo.toml`
  (FR-050, FR-051, SC-021, R2)
- [ ] T008 [P] [US13] Crear `backend/tests/integration/test_clientes_http.rs`: alta 201, listado y
  detalle con sesión `OPERACION` 200, alta/edición/baja/reactivación con `OPERACION` 403, documento
  duplicado 409 `documento_duplicado`, baja lógica conserva ventas históricas; declarar el target
  en `backend/Cargo.toml` (FR-049, FR-055)
- [ ] T009 [P] [US13] Crear `backend/tests/integration/test_ventas_http.rs` cubriendo Escenarios
  13.1–13.9: venta 201 con `stock_resultante`, stock insuficiente 409 con el mensaje exacto,
  atomicidad multi-línea, producto inactivo 422, precio congelado, anulación con
  `ENTRADA/DEVOLUCION`, doble anulación 409, descuento, cliente opcional y líneas repetidas;
  declarar el target en `backend/Cargo.toml` (FR-050–FR-054, SC-021)
- [ ] T010 [US13] Agregar a `backend/tests/integration/test_rbac_matriz.rs` la matriz de ventas y
  clientes: anular venta y escribir clientes con `OPERACION` → 403; vender y leer → permitido
  (FR-055)

### Implementation for User Story 13

- [ ] T011 [P] [US13] En `backend/src/services/ventas.rs`: agregar la función pura
  `agregar_lineas`, tomar el precio de `productos.precio_venta` y rechazar `precio_unitario` con
  422 (mensaje exacto de FR-050), validar stock sobre la cantidad agregada y completar
  `stock_resultante` en `crear`, `obtener` y `anular` con el movimiento más reciente de la venta
  (FR-050–FR-052, SC-021, R1, R2, R9)
- [ ] T012 [P] [US13] En `backend/src/handlers/ventas.rs`: `anular_venta` con `RequiereGestion` y
  respuestas 403/409 documentadas en `utoipa`; crear/listar/ver siguen con `AuthUsuario`
  (FR-055, R3)
- [ ] T013 [P] [US13] En `backend/src/handlers/clientes.rs`: POST, PUT, activar y desactivar con
  `RequiereGestion`; GET y GET/{id} con `AuthUsuario`; documentar 403 (FR-049, FR-055, R3)
- [ ] T014 [US13] Agregar las rutas de ventas y clientes a la lista esperada de
  `backend/tests/unit/test_openapi.rs` y verificar que `backend/src/docs.rs` registre paths y
  schemas (`VentaDetalle`, `ListaVentas`, `Cliente`, `ListaClientes`), incluidas las respuestas
  403/409 (FR-049, FR-054, FR-055). Depende de T012 y T013

**Checkpoint**: backend completo y en verde; `cargo sqlx prepare --check` sin diferencias.

## Phase 4: User Story 13 — Punto de venta y pantallas de administración

**Goal**: POS en `/caja` y administración en `/admin/ventas` y `/admin/clientes` con API tipada.

**Independent Test**: `npm test`, `npm run check` y la secuencia de UI de `quickstart.md`
(carrito con producto repetido, descuento, medio de pago, anulación y CRUD de clientes).

- [ ] T015 [P] [US13] Crear `frontend/src/lib/schemas/ventas.ts` con Zod:
  `VentaSchema`, `VentaLineaSchema` (incluye `stock_resultante`), `VentaDetalleSchema`,
  `ListaVentasSchema` y el schema de entrada `NuevaVentaSchema` sin `precio_unitario`
  (FR-050, FR-054)
- [ ] T016 [P] [US13] Crear `frontend/src/lib/schemas/clientes.ts` con `ClienteSchema`,
  `ListaClientesSchema` y `NuevoClienteSchema` (FR-049)
- [ ] T017 [P] [US13] Crear `frontend/src/lib/api/ventas.ts` con `crearVenta`, `listarVentas`,
  `obtenerVenta` y `anularVenta` usando `api()` y validando con los schemas de T015
  (FR-054, R6)
- [ ] T018 [P] [US13] Crear `frontend/src/lib/api/clientes.ts` con `listarClientes`,
  `obtenerCliente`, `crearCliente`, `actualizarCliente`, `activarCliente` y `desactivarCliente`
  usando los schemas de T016 (FR-049, R6)
- [ ] T019 [P] [US13] Crear `frontend/src/lib/api/ventas.test.ts` y
  `frontend/src/lib/api/clientes.test.ts` (mock de `fetch`) cubriendo rutas, cuerpos, mapeo de
  errores `ApiError` y `stock_resultante` (FR-049, FR-054, Principio IV)
- [ ] T020 [US13] Reemplazar `frontend/src/routes/caja/+page.svelte` por el POS: búsqueda de 004,
  carrito con fusión de líneas por producto, descuento con validación, medio de pago y
  confirmación; muestra número, total y stock resultante (FR-050, FR-053, FR-054, 13.1/13.7/13.8).
  Depende de T017
- [ ] T021 [US13] Crear `frontend/src/routes/admin/ventas/+page.svelte`: filtros por fecha,
  depósito, cliente y estado; detalle con líneas; anulación con confirmación y mensaje de error
  (FR-052, FR-054). Depende de T017
- [ ] T022 [US13] Crear `frontend/src/routes/admin/clientes/+page.svelte`: alta, edición, baja
  lógica y reactivación, con el mensaje de 409 `documento_duplicado` (FR-049, FR-054). Depende de
  T018
- [ ] T023 [US13] Agregar los enlaces "Ventas" y "Clientes" al bloque `GESTION` de
  `frontend/src/routes/+layout.svelte`, apuntando a `/admin/ventas` y `/admin/clientes`
  (FR-054, R6)

**Checkpoint**: POS y administración funcionando contra el backend; suite frontend en verde.

## Phase 5: Polish & Cross-Cutting Concerns

- [ ] T024 Documentar en `docs/api.md` las secciones `/api/ventas` y `/api/clientes` con permisos,
  códigos de error y ejemplos de `stock_resultante` (FR-049–FR-055, R8)
- [ ] T025 Ejecutar los quality gates y validar `quickstart.md`: `cargo test`,
  `cargo sqlx prepare --check`, `npm test`, `npm run check`, `npm run build`; sin
  `unwrap`/`panic!` en rutas de producción (Constitución, sección Quality Gates)

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: puede empezar ya; T001 bloquea cualquier cambio de SQL
- **Foundational (Phase 2)**: depende de Setup; T003–T006 bloquean los tests y los servicios
- **US13 backend (Phase 3)**: depende de Phase 2; tests (T007–T010) antes de implementación
  (T011–T014)
- **US13 frontend (Phase 4)**: depende de T011 y T012 (contrato con `stock_resultante` y
  anulación); T017/T018 antes de las pantallas
- **Polish (Phase 5)**: depende de todo lo anterior

### Within User Story 13

- T007–T010 (RED) → T011–T014 (GREEN)
- T015/T016 → T017/T018 → T019
- T017 → T020/T021; T018 → T022 → T023
- T014 depende de T012/T013

### Parallel Opportunities

- T003 y T004: archivos distintos (`models/cliente.rs`, `models/venta.rs`)
- T007, T008 y T009: targets de test distintos
- T011, T012 y T013: servicios y handlers distintos (tras el RED)
- T015, T016, T017, T018 y T019: archivos de frontend distintos

### Parallel Example: User Story 13

```bash
# RED en paralelo
Task: "Unit tests de agregación en backend/tests/unit/test_venta_validacion.rs"
Task: "Integración de clientes en backend/tests/integration/test_clientes_http.rs"
Task: "Integración de ventas en backend/tests/integration/test_ventas_http.rs"

# GREEN en paralelo
Task: "Agregación y precio de catálogo en backend/src/services/ventas.rs"
Task: "RequiereGestion en backend/src/handlers/ventas.rs"
Task: "RequiereGestion en backend/src/handlers/clientes.rs"
```

## Implementation Strategy

### MVP First

1. Phase 1 (T001–T002) y Phase 2 (T003–T006).
2. Phase 3 completa: tests RED y backend en verde.
3. Validar con `quickstart.md` (curl) antes de tocar el frontend.

### Incremental Delivery

1. Backend corregido y testeado → valor inmediato por API.
2. POS en `/caja` → primera entrega usable por el cajero.
3. `/admin/ventas` y `/admin/clientes` + enlaces → cierre de FR-049/FR-052/FR-054.
4. `docs/api.md` y gates → cierre de la tarea.

## Notes

- Las tareas del `tasks.md` anterior marcadas `[X]` **no estaban hechas**: tests, POS, admin y
  `$lib/api` no existían. Todas se reabren acá con entregables verificables.
- No se agregan migraciones: `0014_ventas.sql` ya está aplicada.
- Commit después de cada tarea o grupo lógico; no cerrar una tarea sin su test en verde.
