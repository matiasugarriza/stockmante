# Tasks: Rediseño visual móvil en modo oscuro cyberpunk

**Input**: Design documents from `specs/019-rediseno-visual-mobile/`

**Prerequisites**: [plan.md](./plan.md), [spec.md](./spec.md), [research.md](./research.md),
[data-model.md](./data-model.md), [contracts/](./contracts/), [quickstart.md](./quickstart.md)

**Tests**: obligatorios para la lógica pura (Constitución IV: RED antes de GREEN). Cada tarea de
test se escribe y se corre para verla fallar antes de su implementación. Las plantillas se validan
con `svelte-check` y la aceptación visual con las capturas de Playwright; no se escriben tests de
estilo.

**Organization**: tareas agrupadas por historia de usuario para que cada incremento sea
independientemente verificable. El orden de fases sigue el plan: fundaciones → marco → tablero →
listas → registro rápido → acceso y administración → cierre.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: puede ejecutarse en paralelo (archivos distintos, sin dependencias pendientes)
- **[Story]**: `US1`…`US5`, equivalentes a US19.1…US19.5 de `spec.md`
- Cada tarea incluye la ruta exacta del archivo o carpeta

## Path Conventions

Aplicación web existente: `frontend/src/` para SvelteKit, `scripts/` para herramientas y
`specs/019-rediseno-visual-mobile/` para evidencia y documentos. No se toca `backend/`.

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: preparar evidencia, herramienta de capturas y línea base antes de tocar componentes.

- [ ] T001 Auditar colores literales (`rgba(`, hex sueltos) y media queries en `frontend/src/lib/components/**/*.svelte` y `frontend/src/routes/**/*.svelte`; registrar en `specs/019-rediseno-visual-mobile/research.md` los que deban pasar a tokens
- [X] T002 [P] Crear la estructura `specs/019-rediseno-visual-mobile/evidencia/{vertical,horizontal,compacta}/` y sembrar `specs/019-rediseno-visual-mobile/evidencia/validacion.md` con la tabla de V1–V13
- [X] T003 [P] Crear `scripts/capturar-movil.mjs` con Playwright: login con `CAPTURE_EMAIL`/`CAPTURE_PASSWORD`, emulación 390×844, 844×390 y 320×568, recorrido de las 21 rutas y del cambio de contraseña, salida por carpeta
- [X] T004 Ejecutar la línea base en `frontend/` (`npm run test`, `npm run check`, `npm run build`) y registrar el resultado de partida en `specs/019-rediseno-visual-mobile/evidencia/validacion.md`
- [X] T005 [P] Agregar el icono `puntos` (menú desbordable) a `frontend/src/lib/components/ui/iconos.ts` y `frontend/src/lib/components/ui/Icono.svelte`
- [X] T006 [P] Corregir el idioma del documento (`lang="en"` → `lang="es"`) en `frontend/src/app.html` para que el lector de pantalla pronuncie en español

**Checkpoint**: hay evidencia base, herramienta de capturas e iconografía; la suite está registrada.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: tokens, dispositivo, tema forzado, avisos, navegación, contraste y primitivas táctiles.
Sin esto ninguna historia se puede implementar.

**⚠️ CRITICAL**: ninguna historia puede comenzar hasta completar esta fase.

### Lógica pura (RED → GREEN)

- [X] T007 [P] Escribir el test que falla de `luminanciaRelativa` y `ratioContraste` (valores WCAG de referencia) en `frontend/src/lib/utils/contraste.test.ts`
- [X] T008 Implementar `frontend/src/lib/utils/contraste.ts` hasta GREEN
- [X] T009 [P] Escribir el test que falla de `clasificarDispositivo` (767 → `telefono`, 768 y 1023 → `tablet`, 1024 → `escritorio`), `esLayoutMovil` y `orientacion` en `frontend/src/lib/dispositivo/dispositivo.test.ts`
- [X] T010 Implementar `frontend/src/lib/dispositivo/dispositivo.ts` con el store y `iniciarDispositivo` (resize y orientationchange) hasta GREEN
- [X] T011 [P] Extender el test que falla de `resolverTema` con `esLayoutMovil` (móvil siempre `dark`) en `frontend/src/lib/tema/tema.test.ts` y actualizar los casos existentes
- [X] T012 Implementar el forzado móvil en `frontend/src/lib/tema/tema.ts` y `frontend/src/app.html` (evaluar `matchMedia('(max-width: 1023px)')` antes del primer pintado y re-aplicar al cruzar el corte) hasta GREEN
- [X] T013 [P] Escribir el test que falla de `clasificarStock` (stock `<= 0` → `critico`, `bajo_minimo` → `bajo`, resto → `normal`) en `frontend/src/lib/utils/inventario.test.ts`
- [X] T014 Implementar `clasificarStock` y el texto del badge en `frontend/src/lib/utils/inventario.ts` hasta GREEN
- [X] T015 [P] Escribir el test que falla de la cola de avisos con temporizador inyectado (alta, cierre manual, expiración) y de `mensajeMovimiento` (tipo, producto, depósito, cantidad y stock resultante) en `frontend/src/lib/stores/avisos.test.ts`
- [X] T016 Implementar `frontend/src/lib/stores/avisos.ts` (cola, temporizador y `mensajeMovimiento`) hasta GREEN
- [X] T017 [P] Extender el test que falla de `destinosBarraInferior` (4 destinos + "Más", cuarto según rol, "Más" activo fuera de la barra, RBAC) en `frontend/src/lib/navegacion/navegacion.test.ts`
- [X] T018 Implementar `destinosBarraInferior` en `frontend/src/lib/navegacion/navegacion.ts` hasta GREEN

### Tokens y accesibilidad

- [X] T019 Agregar los tokens cyberpunk, la geometría móvil, el área segura y los modos `prefers-contrast: more`, `forced-colors: active` y `prefers-reduced-motion` en `frontend/src/lib/diseno/tokens.css` y `frontend/src/lib/diseno/movil.css`; importar `movil.css` en `frontend/src/routes/layout.css`
- [X] T020 Extender `frontend/src/lib/utils/contraste.test.ts` para leer `tokens.css` y `movil.css` con `fs`, validar la matriz de `contracts/tokens-movil.md` y fallar si un par baja de su mínimo o se usa `--peligro`/`--acento-secundario` como `color:` de texto (depende de T008 y T019)

### Primitivas compartidas

- [X] T021 [P] Crear `frontend/src/lib/components/ui/AvisoFlotante.svelte` (variantes éxito/error, `role="status"`, cierre manual, destello de una pulsación)
- [X] T022 Extender `frontend/src/lib/components/ui/Modal.svelte` con `variante="hoja"`: en móvil ocupa `100dvh`, cabecera fija, cuerpo desplazable, pie de acciones sticky y restauración de la posición de scroll; conserva foco atrapado y Escape
- [X] T023 [P] Crear `frontend/src/lib/components/movil/TarjetaRegistro.svelte` con `titulo`, `subtitulo`, `mono`, `badge`, `campos`, `acciones` y `onabrir` según `contracts/ui-movil.md` §2
- [X] T024 Crear `frontend/src/lib/components/movil/TarjetaInventario.svelte` con SKU monoespaciado, nombre a dos líneas con elipsis, badge con texto y acciones de 44×44 (depende de T014 y T023)
- [X] T025 Modificar `frontend/src/routes/+layout.svelte` para iniciar `iniciarDispositivo()`, montar `AvisoFlotante` y conservar `iniciarTema()`
- [X] T026 [P] Ajustar el área táctil y la tipografía de `frontend/src/lib/components/ui/Boton.svelte`, `frontend/src/lib/components/ui/Campo.svelte` y `frontend/src/lib/components/ui/EtiquetaEstado.svelte` (mínimo 44×44, campos a 16 px, badge con texto)

**Checkpoint**: fundaciones listas; las historias pueden comenzar y el guard de contraste existe.

---

## Phase 3: User Story 1 - Marco móvil cyberpunk con navegación de una mano (Priority: P1) 🎯 MVP

**Goal**: toda pantalla autenticada se abre en un marco compacto oscuro con cabecera, búsqueda rápida,
barra inferior de 4 destinos y panel "Más" con la navegación completa.

**Independent Test**: a 390×844, recorrer todos los destinos de ambos roles desde la barra y el
panel, buscar un producto y verificar que el marco se mantiene, que la ruta activa se resalta, que
no hay desborde horizontal y que el tema es oscuro desde el primer pintado.

### Tests de la historia

- [X] T027 [P] [US1] Escribir el test que falla de `buscarInventario` (búsqueda por nombre/SKU, fallback por código de barras, sin resultados y error) en `frontend/src/lib/api/inventario.test.ts`

### Implementación

- [X] T028 [P] [US1] Crear `frontend/src/lib/components/movil/CabeceraMovil.svelte` (título con elipsis, botón de búsqueda con nombre accesible, menú de usuario, sin selector de tema)
- [X] T029 [P] [US1] Crear `frontend/src/lib/components/movil/BarraInferior.svelte` usando `destinosBarraInferior`, con `aria-current`, contador de alertas y área segura
- [X] T030 [US1] Crear `frontend/src/lib/components/movil/MarcoMovil.svelte` (cabecera, `main` con scroll-padding, barra inferior, panel y aviso; depende de T028 y T029)
- [X] T031 [US1] Modificar `frontend/src/lib/components/marco/MarcoApp.svelte` para renderizar `MarcoMovil` cuando `$layout.esMovil` y conservar el marco de 018 en escritorio (depende de T030)
- [X] T032 [US1] Modificar `frontend/src/lib/components/marco/BarraLateral.svelte`: panel móvil con foco atrapado, Escape, bloqueo de scroll, retorno de foco, área segura y estilo cyberpunk
- [X] T033 [US1] Modificar `frontend/src/lib/components/marco/BarraSuperior.svelte` y `frontend/src/lib/components/ui/BotonTema.svelte` para no renderizar el selector en móvil y mantener el área de toque
- [X] T034 [US1] Implementar `frontend/src/lib/api/inventario.ts` hasta GREEN (compone `listarStock` y `buscarProductos`; sin endpoints nuevos)
- [X] T035 [US1] Crear `frontend/src/lib/components/movil/BuscadorMovil.svelte` (overlay, debounce 250 ms, máximo 20 resultados, descarte de respuestas viejas, estados y `TarjetaInventario`)
- [X] T036 [US1] Integrar el buscador en `CabeceraMovil` y resolver la navegación al resultado o a la hoja de movimiento según la pantalla
- [X] T037 [US1] Agregar al marco el enlace "Saltar al contenido", los landmarks (`header`, `nav`, `main`), `aria-current` y el orden de foco en `frontend/src/lib/components/movil/MarcoMovil.svelte` y `BarraInferior.svelte`
- [ ] T038 [US1] Verificar la historia con `quickstart.md` V1, V3 y V4 (oscuro sin destello, navegación, RBAC y búsqueda) y registrar en `specs/019-rediseno-visual-mobile/evidencia/validacion.md`

**Checkpoint**: el marco móvil, la navegación y la búsqueda funcionan en todas las rutas; las páginas
aún muestran sus tablas.

---

## Phase 4: User Story 2 - Tablero móvil con las métricas que importan (Priority: P2)

**Goal**: Inicio muestra las métricas clave en un carrusel de una tarjeta por vista, con las alertas
primero y cada widget resolviendo su propia carga, vacío y error.

**Independent Test**: con datos sembrados, comparar cada número del carrusel contra la API; con la
base vacía, verificar ceros y estados vacíos; con un `OPERACION`, confirmar que no se piden datos de
gestión y que un endpoint caído solo rompe su tarjeta.

### Tests de la historia

- [X] T039 [P] [US2] Escribir el test que falla de `indiceSiguiente` e `indiceAnterior` (sin salir de rango, con y sin vuelta) en `frontend/src/lib/utils/carrusel.test.ts`

### Implementación

- [X] T040 [US2] Implementar `frontend/src/lib/utils/carrusel.ts` hasta GREEN
- [X] T041 [US2] Crear `frontend/src/lib/components/movil/CarruselMetricas.svelte` con scroll-snap de una tarjeta, botones anterior/siguiente, puntos, teclado y `aria-roledescription="carrusel"` (depende de T040)
- [X] T042 [US2] Adaptar `frontend/src/routes/+page.svelte` para móvil: carrusel de métricas, alertas primero, orden métricas → casos urgentes → actividad → accesos rápidos
- [X] T043 [US2] Revisar los estados de carga, vacío y error por widget en `frontend/src/routes/+page.svelte` y `frontend/src/lib/components/inicio/*.svelte`, sin bloquear el resto del tablero
- [ ] T044 [US2] Verificar la historia con `quickstart.md` V5 y registrar la evidencia

**Checkpoint**: Inicio es un tablero móvil autónomo; las listas siguen en tabla.

---

## Phase 5: User Story 3 - Inventario en tarjetas táctiles con badges de stock (Priority: P3)

**Goal**: las tablas densas se convierten en tarjetas con SKU, nombre, badge de stock y acciones
rápidas; los filtros viven en un panel colapsable y en horizontal la lista pasa a dos columnas.

**Independent Test**: en Stock, Productos, Caja, Movimientos, Compras y Auditoría, filtrar y paginar
en el teléfono, verificar cada fila como tarjeta con badge textual y acciones de 44×44, y confirmar
que las acciones producen el mismo resultado que en escritorio.

### Implementación

- [X] T045 [P] [US3] Crear `frontend/src/lib/components/movil/ListaTarjetas.svelte` (estados, contador, esqueletos, vacío, error y pie) según `contracts/ui-movil.md` §4
- [X] T046 [P] [US3] Escribir el test que falla de `contarFiltrosActivos` en `frontend/src/lib/utils/filtros.test.ts`, implementarlo en `frontend/src/lib/utils/filtros.ts` y crear `frontend/src/lib/components/movil/FiltrosMovil.svelte` (panel colapsable, chips de filtros activos, limpiar de una vez)
- [X] T047 [US3] Convertir la tabla de `frontend/src/routes/admin/stock/+page.svelte` a `ListaTarjetas` con `TarjetaInventario`, badge y filtros móviles, conservando el desglose y la paginación
- [X] T048 [US3] Convertir los resultados de `frontend/src/routes/caja/+page.svelte` a tarjetas con acción "Agregar al carrito" y conservar el aviso de truncados
- [X] T049 [US3] Convertir el historial de `frontend/src/routes/movimientos/+page.svelte` a tarjetas con tipo, motivo, depósito, fecha y líneas
- [ ] T050 [US3] Convertir el historial de `frontend/src/routes/transferencias/+page.svelte` a tarjetas con origen, destino y cantidades
- [ ] T051 [US3] Convertir las dos tablas de `frontend/src/routes/conteos/+page.svelte` a tarjetas (conteos y líneas) conservando estados y acciones
- [X] T052 [P] [US3] Convertir `frontend/src/lib/components/productos/ProductoTabla.svelte` a tarjetas con nombre, SKU, categoría, ubicaciones, precio y estado
- [X] T053 [P] [US3] Convertir `frontend/src/lib/components/categorias/CategoriaTabla.svelte` a tarjetas con nombre, descripción y acciones
- [X] T054 [P] [US3] Convertir `frontend/src/lib/components/proveedores/ProveedorTabla.svelte` a tarjetas con datos de contacto y acciones
- [X] T055 [P] [US3] Convertir `frontend/src/lib/components/compras/CompraTabla.svelte` a tarjetas con proveedor, fecha, total, estado y acciones
- [X] T056 [P] [US3] Convertir `frontend/src/lib/components/usuarios/UsuarioTabla.svelte` a tarjetas con nombre, correo, rol, estado y acciones
- [ ] T057 [US3] Convertir la tabla de `frontend/src/routes/admin/ventas/+page.svelte` a tarjetas con fecha, cliente, total y estado
- [ ] T058 [US3] Convertir la tabla de `frontend/src/routes/admin/clientes/+page.svelte` a tarjetas con nombre, documento y contacto
- [ ] T059 [US3] Convertir la tabla de `frontend/src/routes/admin/auditoria/+page.svelte` a tarjetas con acción, entidad, usuario y fecha
- [ ] T060 [US3] Convertir el árbol de ubicaciones de `frontend/src/routes/admin/depositos/+page.svelte` a una lista jerárquica táctil con las mismas acciones
- [ ] T061 [US3] Aplicar "fila como tarjeta" a las tablas de `frontend/src/lib/components/reportes/Kardex.svelte`, `Rotacion.svelte`, `StockValorizado.svelte` y `Ventas.svelte` con `data-etiqueta` por celda y las reglas CSS en `frontend/src/lib/diseno/movil.css`
- [ ] T062 [US3] Aplicar la misma variante a la tabla de errores y a la zona de carga de `frontend/src/routes/admin/importacion/+page.svelte`, con controles de 44×44
- [ ] T063 [US3] Agregar en `frontend/src/lib/diseno/movil.css` la disposición de dos columnas para horizontal (teléfono y tablet) y verificar que no se pierdan acciones, badges ni paginación
- [ ] T064 [US3] Verificar la historia con `quickstart.md` V6 y registrar la evidencia

**Checkpoint**: consulta y lectura de inventario resueltas en tarjetas; queda registrar movimientos.

---

## Phase 6: User Story 4 - Registro rápido de movimiento con modal y destello neón (Priority: P4)

**Goal**: desde una tarjeta o el botón flotante, abrir una hoja a pantalla completa, validar la
cantidad contra el stock del depósito, confirmar y recibir un aviso con el stock resultante.

**Independent Test**: desde Stock, registrar una entrada y una salida válidas y una cantidad
inválida; comprobar bloqueo del fondo, teclado numérico, validación de máximos, aviso con el stock
resultante y un solo movimiento ante toques repetidos.

### Tests de la historia

- [X] T065 [P] [US4] Escribir el test que falla de `validarCantidadMovil` (entrada > 0; salida/merma ≤ disponible; ajuste ≠ 0 y solo `GESTION`; vacío, cero, negativo y decimales) en `frontend/src/lib/utils/inventario.test.ts`

### Implementación

- [X] T066 [US4] Implementar `validarCantidadMovil` en `frontend/src/lib/utils/inventario.ts` hasta GREEN
- [X] T067 [US4] Crear `frontend/src/lib/components/movimientos/HojaMovimiento.svelte` con depósito, desglose de stock (`obtenerStockProducto`), tipo/motivo, cantidad, validación, `crearMovimiento` y error dentro de la hoja (depende de T066)
- [X] T068 [P] [US4] Crear `frontend/src/lib/components/movil/BotonFlotante.svelte` (56×56, nombre accesible, por encima de la barra y del área segura)
- [X] T069 [US4] Integrar el FAB "Ajuste de stock" y la hoja en `frontend/src/routes/admin/stock/+page.svelte`
- [X] T070 [US4] Integrar el FAB "Nuevo movimiento" y la hoja en `frontend/src/routes/movimientos/+page.svelte`
- [ ] T071 [US4] Crear `frontend/src/lib/components/ventas/HojaVenta.svelte` (carrito, totales, descuento, confirmar) e integrar el FAB "Nueva venta" en `frontend/src/routes/caja/+page.svelte`
- [ ] T072 [US4] Agregar el FAB con foco a la acción principal en `frontend/src/routes/transferencias/+page.svelte`, `frontend/src/routes/conteos/+page.svelte` y `frontend/src/routes/asistente/+page.svelte`
- [X] T073 [US4] Integrar el aviso neón con el resultado concreto (`Entrada registrada: producto · depósito · cantidad → stock`) y el bloqueo de doble envío en las hojas de movimiento y venta
- [ ] T074 [US4] Verificar la historia con `quickstart.md` V7 y registrar la evidencia

**Checkpoint**: registro rápido completo; resta la coherencia de acceso y administración.

---

## Phase 7: User Story 5 - Acceso y administración coherentes en móvil (Priority: P5)

**Goal**: login, registro, verificación, cambio obligatorio de contraseña y formularios de
administración comparten la identidad móvil, con teclado y áreas táctiles correctas.

**Independent Test**: a 320 y 390 px, ejecutar los recorridos de acceso y, en cada pantalla de
administración, abrir un formulario, guardar y ejecutar una acción destructiva comprobando que los
mensajes y resultados son los de antes.

### Implementación

- [ ] T075 [US5] Adaptar `frontend/src/lib/components/marco/LienzoAcceso.svelte` y las rutas `frontend/src/routes/login/+page.svelte`, `registro/+page.svelte` y `verificar-correo/+page.svelte` a móvil (tarjeta a pantalla completa, teclado sin tapar campos, sin desborde a 320 px)
- [ ] T076 [US5] Ajustar el modo de cambio obligatorio de contraseña de `frontend/src/routes/login/+page.svelte` a la estética móvil, conservando política y mensajes de 001
- [ ] T077 [P] [US5] Ajustar los formularios de administración a la hoja móvil y a una columna: `frontend/src/lib/components/productos/ProductoForm.svelte`, `categorias/CategoriaForm.svelte`, `proveedores/ProveedorForm.svelte`, `compras/CompraForm.svelte` y `usuarios/UsuarioForm.svelte`
- [ ] T078 [US5] Revisar las confirmaciones destructivas de tarjetas (baja, anulación, eliminación de ubicación) en `frontend/src/routes/admin/*/+page.svelte` y en `frontend/src/lib/components/**/*Tabla.svelte`: nombre del registro y resultado concreto
- [ ] T079 [US5] Verificar la historia con `quickstart.md` V8 y V9 y registrar la evidencia

**Checkpoint**: las 21 pantallas y el cambio de contraseña están cubiertos.

---

## Phase 8: Polish & Cross-Cutting Concerns

**Purpose**: accesibilidad, rendimiento, evidencia y no regresión.

- [ ] T080 Ejecutar el guard de contraste y corregir cualquier color literal restante en `frontend/src/lib/diseno/movil.css` y en los componentes móviles, hasta dejar la matriz de `contracts/tokens-movil.md` en verde
- [ ] T081 [P] Auditar áreas de toque (44×44), foco visible y orden de teclado en las 21 pantallas; corregir en `frontend/src/lib/components/movil/` y `frontend/src/lib/components/ui/`
- [ ] T082 [P] Verificar `prefers-contrast: more`, `forced-colors: active` y `prefers-reduced-motion` con `quickstart.md` V2, V10 y V11; registrar en `evidencia/validacion.md`
- [ ] T083 [P] Medir el rendimiento de la lista filtrada con 10 000 productos (`quickstart.md` V13) y registrar el p95
- [ ] T084 Ejecutar `scripts/capturar-movil.mjs`: 21 pantallas + cambio de contraseña en 390×844, 844×390 y 320×568 dentro de `specs/019-rediseno-visual-mobile/evidencia/`
- [ ] T085 Ejecutar la regresión de escritorio a 1440×900 (`quickstart.md` V12) y las suites `npm run test`, `npm run check` y `npm run build` en `frontend/`; registrar el resultado
- [ ] T086 Actualizar `specs/019-rediseno-visual-mobile/spec.md` (estado final), `specs/README.md` y `specs/019-rediseno-visual-mobile/evidencia/validacion.md` con V1–V13 y los criterios SC-039…SC-049
- [ ] T087 Revisar el diff completo: cero cambios en `backend/`, esquema, endpoints, permisos o dependencias; confirmar el alcance solo de presentación

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: sin dependencias; puede empezar ya.
- **Foundational (Phase 2)**: depende de Setup y **bloquea** todas las historias.
- **US1 (Phase 3)**: depende de Foundational; es el MVP.
- **US2 (Phase 4)**: depende de Foundational y de US1 (marco); puede empezar en paralelo con US3.
- **US3 (Phase 5)**: depende de Foundational y de US1; puede empezar en paralelo con US2.
- **US4 (Phase 6)**: depende de Foundational, US1 y US3 para la aceptación desde tarjeta; el flujo por FAB puede probarse solo con US1.
- **US5 (Phase 7)**: depende de Foundational, US1 y US3 (tarjetas de administración).
- **Polish (Phase 8)**: depende de todas las historias deseadas.

### User Story Dependencies

- **US1 (P1)**: sin dependencia de otras historias; entrega el marco navegable.
- **US2 (P2)**: independiente de US3; usa el marco de US1.
- **US3 (P3)**: independiente de US2; usa `TarjetaInventario` de Foundational.
- **US4 (P4)**: usa el marco de US1 y las tarjetas de US3; la hoja y el FAB se prueban con US1.
- **US5 (P5)**: usa el marco de US1 y las tarjetas de administración de US3.

### Within Each User Story

- Tests de lógica pura primero y fallando (RED) antes de la implementación.
- Componentes base antes de integrarlos en las páginas.
- La historia se considera terminada solo con su escenario de `quickstart.md` registrado.
- Commit por tarea o grupo lógico; la unidad de trabajo es el commit.

### Parallel Opportunities

- Setup: T002, T003, T005 y T006 en paralelo.
- Foundational: todos los tests T007, T009, T011, T013, T015 y T017 en paralelo; las primitivas T021, T023, T024 y T026 en paralelo.
- US1: T028, T029 y T030 en paralelo; T027 (test de búsqueda) en paralelo con ellos.
- US3: T045 y T046 en paralelo; las conversiones de tablas T052–T056 y T057–T059 en paralelo tras T045; T061 y T062 en paralelo.
- Polish: T081, T082 y T083 en paralelo.

---

## Parallel Example: User Story 1

```bash
# Fundaciones de la historia (archivos distintos):
Task: "Crear MarcoMovil.svelte en frontend/src/lib/components/movil/"
Task: "Crear CabeceraMovil.svelte en frontend/src/lib/components/movil/"
Task: "Crear BarraInferior.svelte en frontend/src/lib/components/movil/"
Task: "Escribir el test de buscarInventario en frontend/src/lib/api/inventario.test.ts"
```

## Parallel Example: User Story 3

```bash
# Conversiones independientes una vez creado ListaTarjetas:
Task: "Convertir ProductoTabla.svelte a tarjetas"
Task: "Convertir CategoriaTabla.svelte a tarjetas"
Task: "Convertir ProveedorTabla.svelte a tarjetas"
Task: "Convertir CompraTabla.svelte a tarjetas"
Task: "Convertir UsuarioTabla.svelte a tarjetas"
Task: "Convertir la tabla de /admin/ventas a tarjetas"
Task: "Convertir la tabla de /admin/clientes a tarjetas"
Task: "Convertir la tabla de /admin/auditoria a tarjetas"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Completar Setup y Foundational.
2. Completar US1: marco, navegación, búsqueda y tema oscuro forzado.
3. **PARAR Y VALIDAR**: `quickstart.md` V1, V3 y V4 en un teléfono real o emulado.
4. La app queda usable en móvil aunque las listas sigan en tabla.

### Incremental Delivery

1. Setup + Foundational → fundaciones y guard de contraste.
2. US1 → marco navegable (MVP).
3. US2 → tablero con carrusel.
4. US3 → tarjetas de inventario en todas las listas.
5. US4 → registro rápido con hoja y aviso.
6. US5 → acceso y administración.
7. Polish → accesibilidad, rendimiento, evidencia y no regresión.

### Parallel Team Strategy

1. El equipo completa Setup + Foundational.
2. Luego: una persona en US2 (tablero), otra en US3 (listas) y otra en preparar US4 (hoja) sobre US1.
3. US5 espera las tarjetas de administración de US3.
4. Polish se hace con las tres suites en verde.

---

## Notes

- [P] = archivos distintos y sin dependencias pendientes.
- Cada tarea de test debe verse fallar antes de implementar (Constitución IV).
- No se toca `backend/`: si una tarea parece requerir backend, se resuelve componiendo clientes existentes (D6 de `research.md`).
- Las capturas se guardan en `specs/019-rediseno-visual-mobile/evidencia/` y no reemplazan a las de 018.
- Evitar: tareas vagas, dos tareas sobre el mismo archivo en el mismo bloque y dependencias entre historias que rompan su prueba independiente.
