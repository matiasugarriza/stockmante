# Tasks: 020-escaneo-codigos-barra

**Input**: Design documents from `/specs/020-escaneo-codigos-barra/`

**Prerequisites**: [plan.md](./plan.md) ✓, [spec.md](./spec.md) ✓ (clarificada 2026-11-07),
[research.md](./research.md) ✓, [data-model.md](./data-model.md) ✓,
[contracts/api.md](./contracts/api.md) ✓, [contracts/ui.md](./contracts/ui.md) ✓,
[quickstart.md](./quickstart.md) ✓

**Tests**: obligatorios (Constitución P-IV). Escribir y ver fallar (RED) antes de implementar y
registrarlo en `progress.md`. Gate final: `cargo test`, `cargo sqlx prepare --check`, `npm test`,
`npm run check` y `npm run build` en verde.

**Organización**: por historia de usuario. US1 (cámara) es el MVP porque el usuario no tiene
lector físico. US2 reutiliza el resolvedor y los handlers de US1. US3 (backend + ficha) y US4
(etiquetas) son independientes entre sí salvo el botón «Generar código» de US4.

**Esquema** (P-V): contrastado con `migrations/0002`, `0006` y `0008`; sin migración nueva. T000
lo verifica contra la BD real antes de cualquier tarea de backend.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: puede ejecutarse en paralelo (archivo distinto, sin dependencias pendientes)
- **[Story]**: US1–US4 de la spec
- Cada tarea cita el FR/SC que cubre y una ruta concreta

## Path Conventions

- Backend: `backend/src/`, `backend/tests/`, `backend/Cargo.toml`
- Frontend: `frontend/src/`, `frontend/vite.config.ts`, `frontend/package.json`
- Documentación: `docs/`, `specs/020-escaneo-codigos-barra/`, `specs/README.md`

---

## Phase 1: Setup

**Purpose**: verificar el esquema, preparar dependencias, targets de test y el acceso HTTPS desde
el celular.

- [ ] T000 Verificar con el MCP de PostgreSQL (DBHub, readonly) o `sqlx-cli`: tipo, nulabilidad y
  constraint de `productos.codigo_barras` (`productos_codigo_barras_key`), `productos.sku`,
  largo de `auditoria.accion` (24) y `auditoria.tabla_ref` (32); contar los `codigo_barras` de
  13 dígitos con verificador EAN-13 inválido. Registrar resultado en
  `specs/020-escaneo-codigos-barra/progress.md` (crearlo) y en
  [research.md](./research.md#verificación-de-esquema-p-v) si difiere.
- [ ] T001 Registrar los targets `test_ean13` (`tests/unit/test_ean13.rs`) y
  `test_codigo_barras_http` (`tests/integration/test_codigo_barras_http.rs`) en
  `backend/Cargo.toml`, siguiendo el patrón de los `[[test]]` existentes.
- [ ] T002 [P] Agregar a `frontend/package.json` con versión exacta (publicada hace ≥ 2 semanas):
  `barcode-detector` 3.x, `jsbarcode` 3.12.x y `qrcode-generator` 2.0.x como dependencias y
  `@vitejs/plugin-basic-ssl` 2.3.x como devDependency; `npm install` y registrar la justificación
  en `progress.md` (plan § Complexity Tracking). *Parcial 2026-11-07: instalados
  `barcode-detector@3.2.3`, `zxing-wasm@3.1.6` (directa, para importar el `.wasm` local) y
  `@vitejs/plugin-basic-ssl@2.3.0`; `jsbarcode` y `qrcode-generator` se instalan con US4 (P-I).*
- [X] T003 [P] Modo `movil` en `frontend/vite.config.ts`: con `mode === 'movil'` agregar
  `basicSsl()` y `server.proxy` `{ '/api': 'http://localhost:3000' }`; crear
  `define` de `import.meta.env.PUBLIC_API_URL = ''` (el `.gitignore` excluye `.env.*`, así que no
  se usa `.env.movil`) y el script `"dev:movil": "vite dev --host --mode movil"`. `npm run dev` no
  cambia (FR-161, R7). *Hecho 2026-11-07: verificado con `curl -k` en `https://localhost:5173` y
  `https://192.168.1.48:5173`; `/api` llega al backend.*

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: normalización de lecturas y resolución exacta a producto, que usan US1 y US2.

**⚠️ CRITICAL**: US1 y US2 no empiezan hasta cerrar esta fase.

- [X] T004 [P] RED: `frontend/src/lib/codigos/escaner.test.ts` con `normalizarLectura` (trim,
  espacios internos, `\x1d` y otros controles; vacío → `null`), `clasificarErrorCamara`
  (`!isSecureContext` → `inseguro`, sin `mediaDevices` → `sin_soporte`, `NotAllowedError` →
  `permiso_denegado`, `NotFoundError`/`OverconstrainedError` → `sin_camara`, `NotReadableError` →
  `ocupada`, otro → `desconocido`, cada uno con el mensaje de [contracts/ui.md](./contracts/ui.md)),
  `elegirLectura` (una sola lectura ante detecciones repetidas) y `esProbableCodigo` (sin
  espacios, 8–64 alfanuméricos) (FR-137, FR-151, edge «dos veces en el campo»).
- [X] T005 [P] RED: casos de `resolverLectura` en `frontend/src/lib/api/productos.test.ts`: acierto
  por `campo=codigo` (no consulta SKU), fallback a `campo=sku`, producto inactivo → `inactivo`, sin
  resultados → `no_encontrado`, valor > 160 caracteres → `no_encontrado` sin fetch, error de red
  propagado con el valor intacto; verifica `incluir_inactivos=true` y que nunca se use
  `campo=todos` (FR-138, FR-140, FR-141, FR-155, SC-051).
- [X] T006 Implementar `frontend/src/lib/codigos/escaner.ts` (incluye `FORMATOS_LECTURA` =
  `ean_13, ean_8, upc_a, code_128, code_39, qr_code`) hasta poner T004 en verde (FR-157).
- [X] T007 Implementar `resolverLectura` y el tipo `ResultadoLectura` en
  `frontend/src/lib/api/productos.ts` (+ esquema en `frontend/src/lib/schemas/productos.ts`) hasta
  poner T005 en verde.

**Checkpoint**: `npm test` en verde; la resolución exacta está lista para las pantallas.

---

## Phase 3: User Story 1 - Escanear con la cámara del celular (Priority: P1) 🎯 MVP

**Goal**: leer EAN/UPC/Code 128/Code 39/QR con la cámara en las 5 pantallas con buscador de
producto, sin escribir nada.

**Independent Test**: desde el celular por HTTPS, escanear `7791234567898` en Caja muestra el
producto con stock en < 2 s; un código desconocido muestra «no encontrado»; un inactivo, «dado de
baja» ([quickstart.md §2–4](./quickstart.md)).

### Implementation for User Story 1

- [X] T008 [US1] Crear `frontend/src/lib/components/codigos/EscanerCamara.svelte`: `import()`
  diferido de `barcode-detector/ponyfill`, wasm local vía `zxing-wasm/reader/zxing_reader.wasm?url`
  + `setZXingModuleOverrides`, `getUserMedia` con `facingMode: { ideal: 'environment' }`, bucle de
  detección ~150 ms, una lectura por activación, detener pistas al leer/cancelar/error/`onDestroy`,
  `role="dialog"`, foco atrapado, `Escape`, avisos `aria-live` y botón «Reintentar»
  (FR-134, FR-151–FR-153, FR-156, FR-157).
- [X] T009 [US1] Crear `frontend/src/lib/components/codigos/BotonEscanear.svelte` (props de
  [contracts/ui.md](./contracts/ui.md), icono de cámara, área ≥ 44×44 px) que abre
  `EscanerCamara` y emite `onLectura` (FR-134, FR-156).
- [X] T010 [US1] Integrar en `frontend/src/routes/caja/+page.svelte`: botón junto al buscador;
  `encontrado` → único resultado con stock del depósito elegido; `inactivo` → aviso y no se
  agrega; `no_encontrado` → aviso con el valor en el buscador; `error_red` → «Reintentar» con el
  mismo valor (FR-139–FR-141, FR-155, FR-160).
- [X] T011 [P] [US1] Integrar en `frontend/src/routes/movimientos/+page.svelte`: la lectura
  selecciona el producto en la línea activa (FR-150, FR-160).
- [X] T012 [P] [US1] Integrar en `frontend/src/routes/transferencias/+page.svelte`: la lectura
  selecciona el producto a transferir (FR-160).
- [X] T013 [P] [US1] Integrar en `frontend/src/routes/conteos/+page.svelte`: la lectura resalta la
  línea del producto (`producto_id`) y enfoca su cantidad contada; si el producto no está en el
  conteo, aviso específico (FR-160).
- [X] T014 [P] [US1] Integrar en `frontend/src/routes/admin/productos/+page.svelte`: la lectura
  abre la ficha; `no_encontrado` ofrece a gestión «Crear producto con este código» (precarga
  `codigo_barras` en `frontend/src/lib/components/productos/ProductoForm.svelte`) (FR-140,
  FR-148, FR-160, edge «código inexistente»).
- [ ] T015 [US1] `npm run check` y validación manual con el celular según
  [quickstart.md §2–4](./quickstart.md); capturas en `specs/020-escaneo-codigos-barra/evidencia/` y
  tiempos de SC-050/SC-053 en `progress.md`; confirmar en la BD que no hubo movimientos (SC-054,
  SC-055, SC-061).

**Checkpoint**: MVP usable con la cámara del celular.

---

## Phase 4: User Story 2 - Lector físico o entrada manual (Priority: P1)

**Goal**: que una pistola (emula teclado) o el ingreso manual + Enter den el mismo resultado que la
cámara.

**Independent Test**: con el foco en el buscador, escribir ` 7791234567898 ` + Enter resuelve el
producto igual que la cámara; el SKU en minúsculas también ([quickstart.md §5](./quickstart.md)).

### Implementation for User Story 2

- [ ] T016 [US2] En las 5 pantallas de T010–T014, al pulsar Enter en el buscador: aplicar
  `normalizarLectura`; si `esProbableCodigo`, llamar al mismo manejador de lectura con
  `formato: 'teclado'` y, si resulta `no_encontrado`, seguir con la búsqueda normal de la pantalla
  (manual por nombre/SKU) sin aproximar el código (FR-135–FR-138, FR-140).
- [ ] T017 [US2] Verificar el texto de ayuda del buscador («SKU, código de barras o nombre») y que
  un valor inválido muestre «sin resultados» sin romper la pantalla; validación manual según
  [quickstart.md §5](./quickstart.md) y registro en `progress.md`.

**Checkpoint**: lectura por cámara, pistola y teclado con el mismo comportamiento.

---

## Phase 5: User Story 3 - Generar y asignar un EAN-13 (Priority: P2)

**Goal**: gestión genera (automático o desde 12 dígitos) y asigna un EAN-13 único, con
confirmación de reemplazo y auditoría.

**Independent Test**: asignar un EAN-13 automático a un producto sin código y encontrarlo
escaneando; duplicado → 409 sin cambios; operación → 403 ([quickstart.md §6](./quickstart.md)).

### Tests for User Story 3 (RED primero)

- [ ] T018 [P] [US3] RED: `backend/tests/unit/test_ean13.rs`: `digito_verificador_ean13`
  (`779123456789` → `8`), `es_ean13_valido` (`7791234567898` sí, `7791234567890` no),
  `ean13_automatico(42)` = `2000000000428` y con `i32::MAX`, `validar_codigo_barras` rechaza 13
  dígitos con verificador malo (mensaje con el dígito esperado) y sigue aceptando 13
  alfanuméricos no numéricos y 12 dígitos (FR-142, FR-143, SC-052).
- [ ] T019 [P] [US3] Actualizar el ejemplo inválido `7791234567890` → `7791234567898` en
  `backend/tests/unit/test_producto_validacion.rs`, `frontend/src/lib/api/inventario.test.ts` y
  `frontend/src/lib/schemas/productos.test.ts`; buscar otros EAN-13 inválidos en
  `backend/tests/` y en el seed (hallazgo de [research.md](./research.md#verificación-de-esquema-p-v)).
  Incluye `captures/samples/importacion/productos-validos.csv`: sus 5 códigos `7790…` tienen
  verificador inválido (ver progress.md, revisión del 2026-11-07).
- [ ] T020 [P] [US3] RED: `backend/tests/integration/test_codigo_barras_http.rs`: automático → 200
  + auditoría `ASIGNAR_CODIGO_BARRAS`; manual 12 dígitos → 200; manual 13 con verificador malo →
  422 sin cambios; código de otro producto → 409 `codigo_barras_duplicado` sin cambios; producto con
  código sin `reemplazar` → 409 `codigo_barras_existente` sin cambios; con `reemplazar: true` → 200
  + `REEMPLAZAR_CODIGO_BARRAS`; mismo código → 200 sin auditoría; operación → 403; id inexistente →
  404; ninguna fila nueva en `movimientos`. Agregar un caso de `PUT /api/productos/{id}` y uno de
  importación CSV con EAN-13 inválido → 422 (FR-144, FR-145, FR-148, FR-149, SC-057–SC-059).

### Implementation for User Story 3

- [ ] T021 [US3] Crear `backend/src/models/codigo_barras.rs` (funciones puras de R4, sin
  `unwrap`) y exportarlo en `backend/src/models/mod.rs`.
- [ ] T022 [US3] Agregar la regla EAN-13 a `validar_codigo_barras` en
  `backend/src/models/producto.rs` con el mensaje de [data-model.md](./data-model.md) hasta poner
  T018 en verde (FR-143).
- [ ] T023 [US3] En `backend/src/error.rs`: variante `CodigoBarrasExistente(String)` → 409
  `codigo_barras_existente`; ampliar el mensaje de `CodigoBarrasDuplicado` con el siguiente paso
  (P-III) y ajustar los tests que comparen el texto anterior.
- [ ] T024 [US3] Agregar `ACCION_ASIGNAR_CODIGO_BARRAS` y `ACCION_REEMPLAZAR_CODIGO_BARRAS` en
  `backend/src/services/auditoria.rs` (FR-149).
- [ ] T025 [US3] Implementar `asignar_codigo_barras(pool, id, solicitud, actor_id)` en
  `backend/src/services/productos.rs`: `pool.begin()`, `SELECT … FOR UPDATE`, reglas de
  [contracts/api.md](./contracts/api.md), `UPDATE`, mapeo de `23505` con `mapear_unicidad`,
  auditoría en la misma transacción y `commit` (FR-142–FR-145, FR-149).
- [ ] T026 [US3] Ruta `POST /{id}/codigo-barras` con `RequiereGestion` en
  `backend/src/handlers/productos.rs` y path OpenAPI en `backend/src/docs.rs`; `cargo test` hasta
  poner T020 en verde y `cargo sqlx prepare --check` (FR-148).
- [ ] T027 [P] [US3] RED: `frontend/src/lib/codigos/ean13.test.ts` con los mismos vectores de T018
  y `completarEan13` (12 → 13; 13 → valida).
- [ ] T028 [P] [US3] RED: casos de `asignarCodigoBarras` en `frontend/src/lib/api/productos.test.ts`
  (body, 409 existente/duplicado y 403 parseados al sobre de error).
- [ ] T029 [US3] Implementar `frontend/src/lib/codigos/ean13.ts` y `asignarCodigoBarras` en
  `frontend/src/lib/api/productos.ts` (+ esquema en `frontend/src/lib/schemas/productos.ts`) hasta
  poner T027 y T028 en verde.
- [ ] T030 [US3] Crear `frontend/src/lib/components/codigos/AsignarCodigo.svelte` según
  [contracts/ui.md](./contracts/ui.md): vista previa automática, entrada de 12 dígitos con
  validación en vivo, casilla de reemplazo obligatoria y mensaje de resultado (FR-142, FR-145).
- [ ] T031 [US3] En `frontend/src/routes/admin/productos/+page.svelte` agregar «Asignar código» solo
  para gestión; en `frontend/src/lib/components/productos/ProductoForm.svelte` pedir confirmación
  explícita al cambiar un `codigo_barras` no vacío (FR-145, FR-148).
- [ ] T032 [US3] Validación manual según [quickstart.md §6](./quickstart.md) (incluida la pantalla
  de Auditoría) y registro en `progress.md`.

**Checkpoint**: códigos EAN-13 generados, únicos y auditados.

---

## Phase 6: User Story 4 - Etiqueta en el formato elegido (Priority: P2)

**Goal**: vista previa, impresión y descarga de etiquetas 50 × 30 mm en EAN-13, Code 128 o QR,
escaneables con el celular.

**Independent Test**: generar la etiqueta de un producto en los 3 formatos, imprimirla o
descargarla y escanearla con la cámara: resuelve al mismo producto ([quickstart.md §7](./quickstart.md)).

### Implementation for User Story 4

- [ ] T033 [P] [US4] RED: `frontend/src/lib/codigos/etiqueta.test.ts`: `formatosDisponibles`
  (tabla de [data-model.md](./data-model.md#etiqueta): EAN-13 válido / interno / sin código),
  `valorCodificado` (código o SKU) y `nombreArchivo` (`etiqueta-<SKU>-<formato>.svg`)
  (FR-158, FR-159).
- [ ] T034 [US4] Implementar `frontend/src/lib/codigos/etiqueta.ts` hasta poner T033 en verde.
- [ ] T035 [US4] Crear `frontend/src/lib/components/codigos/EtiquetaProducto.svelte`: `import()`
  diferido de `jsbarcode` y `qrcode-generator`, SVG a 50 × 30 mm con símbolo, nombre (2 líneas),
  SKU y valor legible; formatos deshabilitados con motivo; aviso «Imprimí al 100 %»; «Imprimir» con
  `@page { size: 50mm 30mm; margin: 0 }` sobre un contenedor de impresión y «Descargar SVG»; sin
  código + EAN-13 → «Generar código» que abre `AsignarCodigo` (FR-146, FR-147, FR-154, FR-158).
- [ ] T036 [US4] Agregar «Etiqueta» solo para gestión en
  `frontend/src/routes/admin/productos/+page.svelte` (FR-148).
- [ ] T037 [US4] Imprimir y descargar las 3 variantes y escanearlas con la cámara del celular
  (y con lector físico si hay uno disponible) según [quickstart.md §7](./quickstart.md); evidencia y
  resultado de SC-056/SC-060 en `progress.md`.

**Checkpoint**: ciclo completo generar → etiquetar → escanear.

---

## Phase 7: Polish & Cross-Cutting Concerns

- [ ] T038 [P] Documentar `POST /api/productos/{id}/codigo-barras` y la regla EAN-13 en
  `docs/api.md`; actualizar el caso de uso de escaneo en `docs/casos-de-uso.md` y el modo
  `dev:movil` en `README.md`.
- [ ] T039 [P] Actualizar `specs/README.md`: estado de 020, rango FR-134 … FR-161 y
  SC-050 … SC-061 en la tabla de specs y en el registro de IDs.
- [ ] T040 Verificar en `npm run build` que el wasm y las librerías de etiqueta quedan en chunks
  diferidos y que no hay pedidos a CDN al abrir el escáner (pestaña de red).
- [ ] T041 Gates: `cargo test`, `cargo sqlx prepare --check`, `npm test`, `npm run check`,
  `npm run build`; buscar `unwrap(`/`expect(`/`panic!` en los archivos tocados del backend.
- [ ] T042 Cerrar `progress.md` (RED observado por tarea, decisiones, pendientes) y pasar el
  **Status** de `spec.md` a «Implemented» si el comportamiento coincide; si cambió, actualizar la
  spec.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: T000 bloquea a todo el backend (T018–T026). T002 bloquea T008 y T035. T003
  bloquea la validación con celular (T015, T037).
- **Foundational (Phase 2)**: bloquea US1 y US2.
- **US1 (Phase 3)**: tras Phase 2. Es el MVP.
- **US2 (Phase 4)**: tras US1 (reutiliza los manejadores de lectura de cada pantalla).
- **US3 (Phase 5)**: backend independiente de US1/US2 (puede empezar tras T000/T001); T031 toca
  `admin/productos/+page.svelte` y conviene hacerlo después de T014.
- **US4 (Phase 6)**: tras T002; el botón «Generar código» necesita T030. T036 toca el mismo archivo
  que T014 y T031.
- **Polish (Phase 7)**: al final.

### Within Each User Story

- Tests RED antes de la implementación (T004/T005 → T006/T007; T018/T020 → T021–T026;
  T027/T028 → T029; T033 → T034).
- Funciones puras → servicio → handler → cliente → componente → pantalla.

### Parallel Opportunities

- Setup: T002 y T003 en paralelo (T000/T001 son rápidos).
- Foundational: T004 y T005 en paralelo.
- US1: T011–T014 en paralelo (archivos distintos) una vez listos T008 y T009.
- US3: T018, T019 y T020 en paralelo; T027 y T028 en paralelo; el backend de US3 puede avanzar en
  paralelo con US1.
- US4: T033 en paralelo con cualquier tarea de US3.

## Parallel Example: User Story 1

```text
Tras T008 + T009:
  T011 movimientos/+page.svelte
  T012 transferencias/+page.svelte
  T013 conteos/+page.svelte
  T014 admin/productos/+page.svelte
```

## Parallel Example: User Story 3

```text
T018 tests/unit/test_ean13.rs
T019 ejemplos EAN en tests existentes
T020 tests/integration/test_codigo_barras_http.rs
T027 lib/codigos/ean13.test.ts
```

## Implementation Strategy

### MVP First (User Story 1)

1. Phase 1 (incluye T003 para abrir la app en el celular por HTTPS).
2. Phase 2.
3. Phase 3 → validar con el celular en Caja y luego en las otras 4 pantallas.
4. **STOP y validar**: con esto ya se identifica cualquier producto con código cargado.

### Incremental Delivery

1. MVP (US1) → 2. US2 (pistola/teclado, casi sin costo) → 3. US3 (generar/asignar) →
4. US4 (etiquetas, cierra el ciclo para productos sin código de proveedor) → 5. Polish.

## Notes

- Ninguna lectura escribe en la API: lo único que escribe es `POST …/codigo-barras` (gestión).
- Commit por tarea o grupo lógico; no saltear el RED ni reconstruirlo artificialmente.
- La validación con celular requiere backend corriendo y `npm run dev:movil` en el mismo PC.
