# Tasks — Búsqueda y caja (004)

**Spec**: [spec.md](./spec.md) | **Plan**: [plan.md](./plan.md) | **Contratos**:
[contracts/api.md](./contracts/api.md) | **Creado**: 2026-10-26 | **Actualizado**: 2026-11-07
(fase 6)

**Tests**: sí. TDD en validación del servicio; integración HTTP para RBAC y orden; tests de
schema/cliente en el frontend.

## Fase 1 — Setup y datos

- [X] T000 (registro retroactivo 2026-11-07) Verificar el esquema real de `productos` y
  `categorias` contra `stockmante` y `stockmante_test`.
  **Evidencia**: 17 columnas revisadas; `productos.codigo_barras` y `categorias.descripcion`
  nullable, resto NOT NULL, coincide con `data-model.md`; `idx_productos_nombre_trgm` es
  `GIN (nombre gin_trgm_ops)`; `pg_trgm` presente. `stockmante_test` validado por T006.
  **Aceptación**: evidencia registrada en `progress.md`.
- [X] T001 Crear `migrations/0007_busqueda_trgm.sql` (`pg_trgm` + GIN sobre `productos.nombre`)
  y aplicarla en `stockmante` y `stockmante_test`. **Aceptación**: `\d productos` muestra el
  índice; `SELECT similarity('yerba','yerva')` funciona.
- [X] T002 Agregar `ListaBusqueda { items: Vec<Producto> }` en `models/producto.rs` con
  `ToSchema`. **Aceptación**: `cargo check` compila.
- [X] T003 RED: crear `backend/tests/unit/test_busqueda.rs` con parsing de `campo` (5 valores +
  inválido), longitudes mínimas, `limite` y `categoria_id` inválidos. **Aceptación**: falla antes
  de implementar.

## Fase 2 — Servicio

- [X] T004 Implementar `CampoBusqueda::parsear` y `BusquedaProductos` en
  `services/productos.rs`. **Aceptación**: T003 en verde.
- [X] T005 Implementar `buscar()` con una rama SQL estática por campo, escapado de `LIKE`,
  filtro de activos/categoría y límite. **Aceptación**: `cargo check`.
- [X] T006 RED/GREEN: `backend/tests/integration/test_busqueda_http.rs` — SKU exacto, código
  exacto, nombre con orden, categoría, `q` vacío + categoría, inactivos excluidos,
  `incluir_inactivos` GESTION/OPERACION, 401 y 422. **Aceptación**: verde.

## Fase 3 — Handler y contrato

- [X] T007 Registrar `GET /api/productos/buscar` en `handlers/productos.rs` (antes de `/{id}`),
  con `AuthUsuario`, validación de rol para inactivos y `#[utoipa::path]`.
- [X] T008 Agregar path y schema a `docs.rs`; extender `test_openapi` para la ruta nueva.
  **Aceptación**: `/api-docs/openapi.json` la lista y el test pasa.

## Fase 4 — Frontend

- [X] T009 RED: tests de `ListaBusquedaSchema` y de `buscarProductos` (query params y parseo).
- [X] T010 Implementar schema `ListaBusquedaSchema` y `buscarProductos()` en
  `#lib/api/productos.ts`. **Aceptación**: T009 verde.
- [X] T011 Crear `/caja/+page.svelte`: buscador con debounce 250 ms, selector de campo, filtro
  por categoría, checkbox de inactivos solo GESTION, tabla con precio/stock mínimo/estado y
  mensaje de sin resultados.
- [X] T012 Agregar el enlace `Caja` en `+layout.svelte` para ambas familias; los enlaces de
  administración siguen solo para GESTION.
- [X] T013 Ajustar `+page.svelte` para orientar a Caja/Administración según el rol.

## Fase 5 — Cierre

- [X] T014 Ejecutar `cargo test`, `cargo sqlx prepare --check` (OK, sin diferencias;
  2026-11-07), `npm test`, `npm run check` y `npm run build`; registrar resultados.
- [X] T015 Correr los pasos de `quickstart.md` contra el backend real y registrar evidencia
  (incluido el tiempo de SC-002 para SKU/código exacto y nombre/categoría como peor caso).
- [X] T016 Actualizar `specs/README.md` (estado de 004) y `docs/api.md` si el contrato final
  difiere; escribir `progress.md`.

## Fase 6 — Ampliación 2026-11-07 (analyze + clarify)

**Objetivo**: cerrar los hallazgos D1–D3, A1, E1, F1 y C2 del análisis del 2026-11-07 con las
clarificaciones de la spec (sesión 2026-11-07). **Prueba independiente**: `npm test` cubre la
búsqueda diferida y el aviso de truncados; `quickstart.md` V2/V3 prueba el aviso en `/caja` y el
p95 de SC-002 sobre 10.000 productos. **RED obligatorio** en T018 (sin waiver preventivo).

### Gobierno

- [X] T017 Registrar el waiver **W-006** en `.specify/memory/constitution.md` con
  `/speckit constitution` (PATCH 1.3.2 → 1.3.3, Sync Impact Report). Alcance: P-V por T000
  registrado el 2026-11-07 después de implementar, y P-IV por RED no registrado en T003, T006 y
  T009 de este archivo. Sin evidencia reconstruida. Condición de cierre: T026 en verde. W-003 y
  W-004 siguen reservados, por eso el número es W-006. **Aceptación**: W-006 figura en «Waivers
  registrados» y la versión es 1.3.3.

### Frontend — búsqueda diferida y truncados [US4]

- [X] T018 [P] [US4] RED: crear `frontend/src/lib/utils/busquedaCaja.test.ts` con
  `vi.useFakeTimers()`: (a) la tarea no corre antes de 250 ms y sí al cumplirse; (b) dos
  `programar` seguidos ejecutan solo la segunda; (c) `esVigente()` de la primera tarea pasa a
  `false` si se programa otra mientras espera su respuesta; (d) `cancelar()` evita la ejecución;
  (e) `mensajeTruncados` devuelve `null` con 0, con `cantidad < limite` y con `limite <= 0`, y
  «Se muestran los primeros 20 resultados. Refiná la búsqueda para encontrar el producto.» con
  `cantidad === limite === 20`; (f) `LIMITE_CAJA === 20`. **Aceptación**: la suite falla porque el
  módulo no existe; el fallo se registra en `specs/004-busqueda-caja/progress.md`.
- [X] T019 [US4] Implementar `frontend/src/lib/utils/busquedaCaja.ts` con `LIMITE_CAJA`,
  `crearBusquedaDiferida(esperaMs)` y `mensajeTruncados(cantidad, limite)` según
  `data-model.md` (modelo de frontend). **Aceptación**: T018 en verde.
- [X] T020 [US4] Refactorizar `frontend/src/routes/caja/+page.svelte`: reemplazar
  `secuencia`/`temporizador` por `crearBusquedaDiferida(250)`, el literal `limite: 20` por
  `LIMITE_CAJA` y mostrar `mensajeTruncados(total, LIMITE_CAJA)` debajo del contador «N
  resultados · X ms». Sin cambiar el resto del comportamiento (descarte de respuestas viejas, `ms`
  medido desde el request hasta el parseo). **Aceptación**: `npm run check` sin errores; T018 sigue
  verde; quickstart V2 verificado a mano.

### SC-002 — catálogo de referencia y medición [US4]

- [X] T021 [US4] Verificar el esquema real de `productos` y `categorias` en la base de
  desarrollo (MCP de PostgreSQL o `sqlx-cli`; solo lectura) antes de escribir el seed: columnas,
  tipos, nullables, `CHECK` y `UNIQUE`, incluidas las columnas que hayan agregado 0009–0014.
  **Aceptación**: evidencia en `progress.md`; si hay columnas NOT NULL sin default que no estén en
  `data-model.md`, se actualiza el data-model antes de T022.
- [X] T022 [US4] Crear `scripts/seed_catalogo_10000.sql` (depende de T021): en una transacción,
  `DELETE FROM productos WHERE sku LIKE 'SEED4-%'`, después `DELETE FROM categorias WHERE nombre
  LIKE 'SEED4-CAT-%'`; insertar 20 categorías `SEED4-CAT-01`…`SEED4-CAT-20` (`nombre VARCHAR(80)
  NOT NULL UNIQUE`, único también por `lower(btrim(nombre))`) y 10.000 productos activos con
  `sku` `SEED4-00001`…`SEED4-10000` (`VARCHAR(30)`, `CHECK (sku ~ '^[A-Z0-9][A-Z0-9-]{2,29}$')`),
  `codigo_barras` de 13 dígitos únicos `779…` (`VARCHAR(64)`, `UNIQUE`), `nombre` variado de rubro
  + marca + presentación + número (`VARCHAR(160) NOT NULL`), `categoria_id` repartido en las 20
  categorías, `unidad_medida 'UNIDAD'`, `costo`/`precio_venta` `NUMERIC(12,2) >= 0` y
  `stock_minimo INT >= 0`. Sin movimientos, stock ni auditoría. Encabezado: «Solo desarrollo; nunca
  en stockmante_test». **Aceptación**: dos corridas seguidas dejan exactamente 10.000 productos y 20
  categorías `SEED4-`.
- [X] T023 [P] [US4] Crear `scripts/medir_sc002.mjs` (Node 18+, sin dependencias): lee
  `STOCKMANTE_URL`, `STOCKMANTE_EMAIL` y `STOCKMANTE_PASSWORD` del entorno (aborta con un mensaje
  claro si falta alguna; nunca las imprime), hace login y conserva la cookie `stockmante_sesion`,
  ejecuta 40 búsquedas por tipo (`campo=sku`, `codigo`, `nombre`, `categoria`) con términos del
  seed `SEED4-`, mide `performance.now()` desde el `fetch` hasta terminar `response.json()`, e
  imprime p50/p95/máximo por tipo y si cumple p95 ≤ 1000 ms. Sale con código distinto de 0 si
  alguna respuesta no es 200. **Aceptación**: corre contra el backend local y produce la tabla.
- [X] T024 [US4] Ejecutar `quickstart.md` V3 (pasos 16–20) contra la base de desarrollo: seed,
  medición, `EXPLAIN (ANALYZE, BUFFERS)` de la búsqueda por nombre y limpieza. Registrar en
  `progress.md` p50/p95/máximo por tipo, el plan de ejecución (si usa
  `idx_productos_nombre_trgm`) y la confirmación de que no quedan filas `SEED4-`. **Aceptación**:
  p95 ≤ 1000 ms en los cuatro tipos, o desvío documentado con decisión (SC-002 no se da por
  cumplido en ese caso).

### Cierre de la ampliación

- [X] T025 [P] Actualizar `docs/api.md` (operador de `campo=categoria`, SKU normalizado en
  `todos`, inferencia de truncado del lado del cliente) y la fila de 004 en `specs/README.md`
  («Implementado; ampliación 2026-11-07 …»). **Aceptación**: coinciden con
  `contracts/api.md`.
- [X] T026 Ejecutar los gates: `cargo test`, `cargo sqlx prepare --check`, `npm test`,
  `npm run check` y `npm run build`; actualizar `progress.md` (fase 6, RED de T018, evidencia de
  T021/T024 y condición de cierre de W-006). **Aceptación**: todo en verde y `progress.md` al día.

## Dependencias de la ampliación

- T017 es independiente y puede ir primero o en paralelo; bloquea solo el cierre (T026).
- T018 → T019 → T020 (RED → GREEN → integración en la pantalla).
- T021 → T022 → T024; T023 en paralelo con T018–T022; T024 necesita T022 y T023.
- T025 en paralelo con T024; T026 al final.

**En paralelo**: T018 y T023 (archivos distintos, sin dependencias), y luego T025 con T024.

**Estrategia**: primero T018–T020 (aviso visible en caja, valor inmediato para el usuario),
después la medición T021–T024 y por último T025/T026 con el waiver ya aprobado.

## Trazabilidad

| Requisito | Tareas |
|---|---|
| Constitución V (verificación de esquema) | T000, T021, T017 (W-006) |
| Constitución IV (RED y lógica testeable) | T017 (W-006), T018–T020 |
| FR-011 | T001–T012, T020 |
| FR-013 (alcance 004: búsqueda) | T005, T006, T011 |
| SC-002 (API vista desde cliente, 10.000 productos) | T001, T005, T015, T021–T024 |
| Caso borde: resultados truncados | T018–T020 |
| Soporte UX (home por rol) | T013 |
| Quality gates | T014, T026 |
| Documentación y cierre | T016, T025 |
