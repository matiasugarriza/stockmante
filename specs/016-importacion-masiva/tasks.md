# Tasks: 016-importacion-masiva

**Input**: Design documents from `/specs/016-importacion-masiva/`

**Prerequisites**: [plan.md](./plan.md) ✓, [spec.md](./spec.md) ✓ (clarificada 2026-11-07),
[research.md](./research.md) ✓, [data-model.md](./data-model.md) ✓, [contracts/api.md](./contracts/api.md) ✓,
[quickstart.md](./quickstart.md) ✓

**Tests**: obligatorios (Constitución P-IV y quality gates). Escribir y ver fallar (RED) antes de
implementar. Gate final: `cargo test`, `cargo sqlx prepare --check`, `npm test`, `npm run check` y
`npm run build` en verde.

**Organización**: una sola historia (US16, P3). El backend va antes que el frontend porque la pantalla
consume `$lib/api/importaciones.ts`. Los tests unitarios e HTTP de backend se escriben primero.

**Esquema verificado** (P-V): `migrations/0002_categorias_productos.sql` y
`migrations/0009_stock_movimientos.sql`; ver detalle en [research.md](./research.md#verificación-de-esquema-p-v).
No hay migración nueva.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: puede ejecutarse en paralelo (archivo distinto, sin dependencias pendientes)
- **[Story]**: US16 (única historia de la spec)
- Cada tarea cita el FR/SC que cubre y una ruta de archivo concreta

## Path Conventions

- Backend: `backend/src/`, `backend/tests/`, `backend/Cargo.toml`
- Frontend: `frontend/src/`
- Documentación: `docs/`, `specs/016-importacion-masiva/`

---

## Phase 1: Setup

**Purpose**: preparar los targets de test y la base tipada del frontend.

- [X] T001 Registrar los targets de test en `backend/Cargo.toml`: `test_importacion_validacion`
  (`tests/unit/test_importacion_validacion.rs`) y `test_importacion_http`
  (`tests/integration/test_importacion_http.rs`), siguiendo el patrón de los `[[test]]` existentes.
- [X] T002 [P] Crear `frontend/src/lib/schemas/importaciones.ts` con esquemas Zod de
  `DetalleImportacion` (`fila: number`, `columna: string`, `mensaje: string`),
  `ResumenImportacion` (`filas_leidas`, `validas`, `con_error`, `errores`, `creados`, `aplicado`) y
  `ErrorImportacion` (`error.codigo`, `error.mensaje`, `error.detalles?`).

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: contrato de error y transporte multipart que consumen backend y frontend.

**⚠️ CRITICAL**: sin estas tareas no se puede implementar US16.

- [X] T003 Implementar en `backend/src/error.rs` la variante
  `ImportacionInvalida(Vec<DetalleImportacion>)` con
  `DetalleImportacion { fila: usize, columna: String, mensaje: String }`; mapear a **422**, código
  `importacion_invalida` y agregar `detalles` al sobre `{ error: { codigo, mensaje, detalles } }`
  solo para esa variante. Ampliar `ErrorRespuesta`/`ErrorDetalle` en `backend/src/docs.rs` con
  `detalles` opcional. Cubrir status/código en `backend/tests/unit/test_apperror.rs`.
- [X] T004 [P] Ampliar `frontend/src/lib/api/client.ts`: `ErrorBody.error.detalles?`,
  `ApiError.detalles` y helper `apiFormData<T>(path, formData)` que haga `fetch` con
  `credentials: 'include'` **sin** fijar `Content-Type` (el navegador agrega el boundary). Cubrir con
  `frontend/src/lib/api/client.test.ts`.

**Checkpoint**: el sobre de error y la subida multipart están listos.

---

## Phase 3: User Story 16 — Importar catálogo y stock desde CSV (Priority: P3)

**Goal**: el Gerente sube un CSV, valida sin escribir, ve errores por fila/columna y aplica de forma
atómica; el stock inicial queda como movimiento `ENTRADA` motivo `IMPORTACION`.

**Independent Test**: `quickstart.md` pasos 3–6 (dry-run 200, apply 201, duplicado 422 atómico,
límites 422, operario 403, 5.000 filas < 60 s) y la pantalla `/admin/importacion`.

### Tests for User Story 16 (RED primero) ⚠️

- [X] T005 [P] [US16] Unit tests en `backend/tests/unit/test_importacion_validacion.rs`: autodetección
  de delimitador `,`/`;`, decimal `.`/`,` según delimitador, BOM UTF-8, comillas dobles, encabezado
  faltante (`Falta la columna 'sku'.`), fila con más columnas que el encabezado, archivo sin filas,
  `stock_inicial` vacío = 0, `deposito` vacío = "Principal", duplicado intra-archivo con filas
  (`(filas 3 y 8)`), SKU inválido (3–30, `^[A-Z0-9][A-Z0-9-]{2,29}$`), unidad fuera de
  `UNIDAD|KG|METRO|CAJA`, montos con `validar_monto` y código de barras 8–64 alfanumérico. Deben
  fallar antes de T008.
- [X] T006 [P] [US16] Integration tests HTTP en `backend/tests/integration/test_importacion_http.rs`:
  `validar` 200 sin escrituras; `aplicar` 201 con `creados`, `stock_por_deposito.cantidad`,
  `productos.stock_actual` y movimiento `ENTRADA`/`IMPORTACION`; una fila inválida en `aplicar` → 422
  y **cero** productos; SKU existente y repetido con filas; categoría inexistente; depósito
  inexistente y falta de "Principal" (solo con stock > 0); 5.001 filas y archivo > 5 MB → 422
  `importacion_invalida` con `"El archivo supera el límite de 5.000 filas o 5 MB."`; `modo` inválido
  → 422 `validacion`; falta `archivo`; `OPERACION` → 403 `sin_permisos`. Deben fallar antes de
  T009–T011.
- [X] T007 [P] [US16] Frontend tests en `frontend/src/lib/api/importaciones.test.ts`: construye
  `FormData` con `archivo`, agrega `?modo=`, parsea `ResumenImportacion` con Zod y propaga
  `ApiError.detalles` en 422. Deben fallar antes de T012.

### Implementation for User Story 16

- [X] T008 [US16] Reescribir `backend/src/services/importaciones.rs` separando parser/validación pura
  de la I/O: `detectar_dialecto`, `parsear_filas`, `validar_fila`, `ResumenImportacion` y
  `DetalleImportacion`. Aplicar D1/D4/D5/D6 de [research.md](./research.md): `csv` con
  `flexible(true)`, límites 5.000 filas/5 MB, `validar_nombre_producto`, `validar_sku`,
  `validar_unidad`, `validar_monto`, `validar_codigo_barras`, `validar_stock_minimo`; duplicados con
  la primera fila; depósito solo si `stock_inicial` > 0; sin `unwrap/expect/panic`.
- [X] T009 [US16] Implementar la persistencia atómica en `backend/src/services/importaciones.rs`
  (depende de T008): una sola `pool.begin()`; `INSERT INTO productos ... RETURNING id`;
  `INSERT ... ON CONFLICT (uq_stock_producto_deposito_ubicacion) DO NOTHING` en
  `stock_por_deposito`; `UPDATE` de cantidad con `FOR UPDATE`; `ENTRADA`/`IMPORTACION` en
  `movimientos` + `movimiento_lineas` (`cantidad_delta = stock_inicial`, `stock_resultante ≥ 0`);
  `UPDATE productos SET stock_actual = <resultante> ...`; `auditoria::registrar` en la misma
  transacción; `rollback` ante cualquier error.
- [X] T010 [US16] Ajustar `backend/src/handlers/importaciones.rs` (depende de T003/T009): `modo` por
  query con default `validar`, `archivo` por `multipart`, `DefaultBodyLimit::max(16 * 1024 * 1024)`
  en la ruta, 200 en validar, 201 en aplicar, 422 `importacion_invalida` con `detalles` para filas,
  límites, encabezado, archivo vacío o ilegible, y 403 para `OPERACION`.
- [X] T011 [US16] Registrar `crate::handlers::importaciones::importar_productos` y los schemas
  `ResumenImportacion`, `FilaErrorImportacion`, `DetalleImportacion` en `backend/src/docs.rs`;
  ampliar `backend/tests/unit/test_openapi.rs` para exigir el path y los schemas.
- [X] T012 [P] [US16] Crear `frontend/src/lib/api/importaciones.ts` (depende de T004):
  `importarProductos(archivo: File, modo: 'validar' | 'aplicar')` con `apiFormData`, parseo Zod del
  resumen y tipos exportados; sin `fetch` fuera de `$lib/api/`.
- [X] T013 [US16] Crear `frontend/src/routes/admin/importacion/+page.svelte` (depende de T012):
  input de archivo, botones "Validar"/"Aplicar", estados de carga, vacío y error, resumen
  (`filas_leidas`, `validas`, `con_error`, `creados`), tabla de errores con fila/columna/mensaje,
  "Aplicar" deshabilitado si hay errores, etiquetas accesibles y Tailwind.
- [X] T014 [US16] Agregar el enlace "Importar" en `frontend/src/routes/+layout.svelte` visible solo
  para `GESTION`, apuntando a `/admin/importacion` (depende de T013).

**Checkpoint**: US16 funcional y testeable de punta a punta.

---

## Phase 4: Polish & Cross-Cutting Concerns

**Purpose**: documentación, performance y gates finales.

- [X] T015 [P] Documentar el endpoint y los códigos en `docs/api.md` (sección "Importación masiva
  (spec 016)") y actualizar el estado de 016 en `specs/README.md`.
- [X] T016 [P] Agregar el test de SC-024 en `backend/tests/integration/test_importacion_http.rs`:
  generar 5.000 filas válidas, aplicar y verificar `creados = 5000` en < 60 s, exactamente 5.000
  productos y sus movimientos; fallar si se supera el umbral.
- [X] T017 Ejecutar los gates: `cargo test`, `cargo sqlx prepare --check`, `npm test`,
  `npm run check`, `npm run build`; corregir cualquier diferencia y confirmar que no hay
  `unwrap/panic!` en rutas de producción.
- [ ] T018 Ejecutar los escenarios de [quickstart.md](./quickstart.md) (dry-run, aplicar, duplicados,
  límites, 403, UI y 5.000 filas) y registrar el resultado en el PR.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: sin dependencias.
- **Foundational (Phase 2)**: depende de Setup; **bloquea** US16.
- **US16 (Phase 3)**: depende de Foundational; los tests T005–T007 van antes de T008–T012.
- **Polish (Phase 4)**: depende de US16 completa.

### Task Dependencies

- T001 → T005, T006 (los targets deben existir para correr los tests).
- T002 → T007, T012.
- T003 → T010 (variante de error).
- T004 → T012.
- T005 → T008 → T009 → T010 → T011.
- T006 se valida contra T009–T011.
- T007 → T012 → T013 → T014.
- T015–T018 al final.

### Parallel Opportunities

- T002, T003 y T004 pueden ir en paralelo (archivos distintos).
- T005, T006 y T007 pueden escribirse en paralelo (RED).
- T012 y T013 comparten la API; T013 depende de T012.
- T015 y T016 en paralelo al final.

---

## Parallel Example: User Story 16

```text
# RED en paralelo:
T005 Unit tests de parser/validación (backend/tests/unit/test_importacion_validacion.rs)
T006 Integration tests HTTP (backend/tests/integration/test_importacion_http.rs)
T007 Tests del cliente API (frontend/src/lib/api/importaciones.test.ts)

# Frontend en paralelo con el backend ya estable:
T012 $lib/api/importaciones.ts
T013 /admin/importacion/+page.svelte
```

---

## Implementation Strategy

### MVP First (US16 backend)

1. Phase 1 (T001–T002) y Phase 2 (T003–T004).
2. Tests RED T005–T006.
3. T008–T011: parser, transacción atómica, handler y OpenAPI.
4. **STOP and VALIDATE**: `quickstart.md` pasos 3–6 y `cargo test`.
5. Luego T007, T012–T014 (UI) y T015–T018.

### Incremental Delivery

1. Base de error + transporte listos.
2. API de dry-run y aplicar con tests → valor backend.
3. Pantalla de importación → valor para el Gerente.
4. Documentación, performance y gates → cierre.

---

## Notes

- [P] = archivos distintos, sin dependencias pendientes.
- Ningún test se marca completo sin haberlo visto fallar antes (P-IV).
- No se modifica `services/importaciones.rs` sin tests RED de T005/T006.
- Cualquier cambio de esquema detectado al implementar obliga a re-verificar contra la BD y actualizar
  `research.md`/`data-model.md` antes de seguir.
