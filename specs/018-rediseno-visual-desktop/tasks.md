# Tasks: Rediseño visual del escritorio

**Input**: Design documents from `specs/018-rediseno-visual-desktop/`

**Prerequisites**: [plan.md](./plan.md), [spec.md](./spec.md), [research.md](./research.md),
[data-model.md](./data-model.md), [contracts/](./contracts/), [quickstart.md](./quickstart.md)

**Tests**: la constitución exige RED antes de GREEN para la lógica pura. Se incluyen tests de
`navegacion`, `formato`, `graficos` y del cliente `reportes`; las plantillas se validan con
`svelte-check` y la aceptación visual con capturas Playwright.

**Organization**: tareas agrupadas por historia de usuario para que cada incremento sea
independientemente verificable.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: puede ejecutarse en paralelo (archivos distintos, sin dependencias pendientes)
- **[Story]**: US18.1 … US18.5 según `spec.md`
- Cada tarea incluye la ruta exacta del archivo o carpeta

## Path Conventions

Aplicación web existente: `frontend/src/` para SvelteKit y `scripts/` para herramientas. La evidencia
vive en `specs/018-rediseno-visual-desktop/evidencia/`.

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: preparar tokens, iconografía y la herramienta de evidencia antes de tocar pantallas.

- [ ] T001 Auditar las 17 capturas de `captures/` contra `captures/samples/light-2.png` y `dark-2.png` y confirmar el inventario de rutas/roles de `specs/018-rediseno-visual-desktop/data-model.md`
- [ ] T002 [P] Crear `frontend/src/lib/diseno/tokens.css` con los tokens claro/oscuro, tipografía, radios, sombras y breakpoints de `specs/018-rediseno-visual-desktop/contracts/tokens.md`
- [ ] T003 Importar `frontend/src/lib/diseno/tokens.css` en `frontend/src/routes/layout.css` y retirar las variables crema/naranja del `:root`
- [ ] T004 [P] Crear `frontend/src/lib/components/ui/Icono.svelte` con los 20 iconos SVG definidos en `specs/018-rediseno-visual-desktop/contracts/ui.md`
- [ ] T005 [P] Crear `scripts/capturar-rediseno.mjs` con Playwright: login, recorrido de las 18 rutas, temas claro/oscuro a 1440×900 y chequeo a 320 px, salida en `specs/018-rediseno-visual-desktop/evidencia/`
- [ ] T006 Ejecutar la suite base en `frontend/` (`npm run test`, `npm run check`) y registrar el resultado de partida en `specs/018-rediseno-visual-desktop/evidencia/antes/estado.md`

**Checkpoint**: existen tokens, iconos y la herramienta de capturas; la suite base está registrada.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: lógica pura testeada, primitivas visuales y marco. Sin esto no se puede rediseñar
ninguna página.

**⚠️ CRITICAL**: ninguna historia de usuario puede comenzar hasta completar esta fase.

### Lógica pura

- [ ] T007 [P] Escribir el test que falla de `gruposVisibles` (RBAC, grupo, orden, ruta activa, contador) en `frontend/src/lib/navegacion/navegacion.test.ts`
- [ ] T008 Implementar `frontend/src/lib/navegacion/navegacion.ts` hasta GREEN con el contrato de `specs/018-rediseno-visual-desktop/contracts/ui.md` §2
- [ ] T009 [P] Escribir el test que falla de formato de moneda ARS, enteros, porcentaje y fecha corta en `frontend/src/lib/utils/formato.test.ts`
- [ ] T010 Implementar `frontend/src/lib/utils/formato.ts` hasta GREEN
- [ ] T011 [P] Escribir el test que falla de escalas de barras, arcos de dona, porcentajes y casos cero/negativo en `frontend/src/lib/utils/graficos.test.ts`
- [ ] T012 Implementar `frontend/src/lib/utils/graficos.ts` hasta GREEN

### Primitivas visuales

- [ ] T013 [P] Crear `frontend/src/lib/components/ui/Boton.svelte` con variantes, tamaño, cargando y foco visible
- [ ] T014 [P] Crear `frontend/src/lib/components/ui/Campo.svelte` con label, ayuda, error y `aria-describedby`
- [ ] T015 [P] Crear `frontend/src/lib/components/ui/EncabezadoPagina.svelte` con título, descripción y snippet de acción
- [ ] T016 [P] Crear `frontend/src/lib/components/ui/Tarjeta.svelte` con encabezado, descripción, acción y pie opcionales
- [ ] T017 [P] Crear `frontend/src/lib/components/ui/EtiquetaEstado.svelte` con variantes éxito/aviso/peligro/info/neutro
- [ ] T018 [P] Crear `frontend/src/lib/components/ui/Aviso.svelte` con variantes y mensaje accionable
- [ ] T019 [P] Crear `frontend/src/lib/components/ui/EstadoVacio.svelte` con icono, texto y acción
- [ ] T020 [P] Crear `frontend/src/lib/components/ui/Esqueleto.svelte` con variantes línea/bloque/tarjeta
- [ ] T021 [P] Crear `frontend/src/lib/components/ui/TablaDatos.svelte` con scroll interno, carga, error, vacío y filas por snippet
- [ ] T022 [P] Crear `frontend/src/lib/components/ui/Paginacion.svelte` con total, página y estados deshabilitados
- [ ] T023 [P] Crear `frontend/src/lib/components/ui/Modal.svelte` con foco atrapado, Escape, backdrop y scroll lock
- [ ] T024 [P] Crear `frontend/src/lib/components/ui/BotonTema.svelte` conectado a `frontend/src/lib/tema/tema.ts`

### Marco

- [ ] T025 Crear `frontend/src/lib/components/marco/ItemNavegacion.svelte` con icono, etiqueta, `aria-current` y contador opcional
- [ ] T026 [P] Crear `frontend/src/lib/components/marco/MenuUsuario.svelte` con nombre, rol, tema y cierre de sesión
- [ ] T027 [P] Crear `frontend/src/lib/components/marco/BarraSuperior.svelte` con botón de menú, contexto de página y menú de usuario
- [ ] T028 [P] Crear `frontend/src/lib/components/marco/BarraLateral.svelte` que renderice `gruposVisibles` y el selector de tema
- [ ] T029 Crear `frontend/src/lib/components/marco/MarcoApp.svelte` con sidebar, topbar, `<main id="contenido">` y estado colapsado/superpuesto
- [ ] T030 Crear `frontend/src/lib/diseno/acceso.css` con el lienzo y la tarjeta de acceso usando los tokens nuevos
- [ ] T031 Crear `frontend/src/lib/components/marco/LienzoAcceso.svelte` con marca, panel, selector de tema y contenido
- [ ] T032 Reescribir `frontend/src/routes/+layout.svelte` para decidir entre `MarcoApp` y `LienzoAcceso`, conservando `refrescarSesion`, alertas de stock, redirección y `admin/+layout.svelte`
- [ ] T033 Verificar `frontend/src/app.html` y `frontend/src/lib/tema/tema.ts`: la clave `stockmante:tema`, `data-tema` y el anti-destello siguen vigentes; ajustar solo si el rediseño lo exige

**Checkpoint**: tokens, primitivas, navegación y marco compilan; `npm run test` y `npm run check` en
verde. Las historias pueden comenzar.

---

## Phase 3: User Story 18.1 - Marco de escritorio y tema (Priority: P1) 🎯 MVP

**Goal**: que cualquier ruta privada se abra dentro del marco, con navegación por rol, tema
persistido y comportamiento responsive.

**Independent Test**: iniciar sesión como GESTION y OPERACION, recorrer la navegación, cambiar de
tema, recargar y reducir la ventana a 320 px sin desborde.

### Implementation for User Story 18.1

- [ ] T034 [US1] Conectar `BarraLateral.svelte` con `gruposVisibles`, `$sesion` y `alertasBajoMinimo` en `frontend/src/routes/+layout.svelte`
- [ ] T035 [US1] Implementar el estado activo por ruta y el contador de Stock en `frontend/src/lib/components/marco/BarraLateral.svelte`
- [ ] T036 [US1] Integrar `BotonTema.svelte` en la barra superior y en el lienzo de acceso, verificando `light`/`dark`/`system` y la persistencia
- [ ] T037 [US1] Implementar colapso (≥1024), panel superpuesto (<1024), backdrop y cierre con Escape en `frontend/src/lib/components/marco/MarcoApp.svelte` y `BarraLateral.svelte`
- [ ] T038 [US1] Agregar landmarks, enlace "Saltar al contenido", foco visible y `aria-current` en `frontend/src/lib/components/marco/` y `frontend/src/routes/+layout.svelte`
- [ ] T039 [US1] Verificar el guard de gestión en `frontend/src/routes/admin/+layout.svelte` y la redirección de OPERACION a `/caja`
- [ ] T040 [US1] Correr `npm run test` y `npm run check` en `frontend/` y corregir regresiones del marco
- [ ] T041 [US1] Generar evidencia del marco en `specs/018-rediseno-visual-desktop/evidencia/{claro,oscuro}/` para `/`, `/caja`, `/movimientos`, `/admin/productos` y `/login`, más chequeo a 320 px en `evidencia/angosto/`
- [ ] T042 [US1] Revisar manualmente los escenarios 1–7 de US18.1 en `specs/018-rediseno-visual-desktop/spec.md` y anotar el resultado en `specs/018-rediseno-visual-desktop/evidencia/antes/estado.md`

**Checkpoint**: US18.1 funciona sola; el resto de las páginas conserva su contenido interno anterior
pero ya vive en el marco.

---

## Phase 4: User Story 18.2 - Tablero de Inicio (Priority: P2)

**Goal**: reemplazar la pantalla mínima de Inicio por un tablero real con indicadores, gráficos,
actividad y accesos rápidos.

**Independent Test**: comparar cada número y gráfico con la API; verificar widgets ocultos para
OPERACION y errores independientes.

### Implementation for User Story 18.2

- [ ] T043 [P] [US2] Crear `frontend/src/lib/schemas/reportes.ts` con el schema de stock valorizado (`items`, `total`) y su tipo
- [ ] T044 [P] [US2] Crear `frontend/src/lib/api/reportes.ts` con `obtenerStockValorizado()` usando `api()` de `frontend/src/lib/api/client.ts`
- [ ] T045 [P] [US2] Escribir el test del cliente `reportes` en `frontend/src/lib/api/reportes.test.ts` (URL, método, parseo y error)
- [ ] T046 [P] [US2] Crear `frontend/src/lib/components/ui/TarjetaKpi.svelte` con etiqueta, métrica tabular, contexto y estado de carga
- [ ] T047 [P] [US2] Crear `frontend/src/lib/components/ui/GraficoBarras.svelte` usando `escalaBarras` de `frontend/src/lib/utils/graficos.ts`
- [ ] T048 [P] [US2] Crear `frontend/src/lib/components/ui/GraficoDona.svelte` usando `arcoDona` de `frontend/src/lib/utils/graficos.ts`
- [ ] T049 [P] [US2] Crear `frontend/src/lib/components/inicio/PanelActividad.svelte` con últimos movimientos y enlace a `/movimientos`
- [ ] T050 [P] [US2] Crear `frontend/src/lib/components/inicio/AccesosRapidos.svelte` con accesos por rol
- [ ] T051 [US2] Reescribir `frontend/src/routes/+page.svelte` con encabezado, KPIs, gráficos, actividad y accesos rápidos
- [ ] T052 [US2] Implementar la carga independiente por widget, el ocultamiento por rol y el reintento en `frontend/src/routes/+page.svelte`
- [ ] T053 [US2] Implementar estados vacíos y esqueletos del tablero en `frontend/src/routes/+page.svelte` y `frontend/src/lib/components/inicio/`
- [ ] T054 [US2] Verificar los totales contra `GET /api/stock`, `GET /api/movimientos`, `GET /api/productos`, `GET /api/transferencias` y `GET /api/reportes/stock-valorizado`
- [ ] T055 [US2] Generar evidencia de `/` en `specs/018-rediseno-visual-desktop/evidencia/{claro,oscuro}/04-inicio.png` y el chequeo a 320 px

**Checkpoint**: Inicio es un tablero autónomo y verificado; las páginas de operación y administración
siguen pendientes.

---

## Phase 5: User Story 18.3 - Acceso rediseñado (Priority: P3)

**Goal**: re-pielar login, cambio obligatorio, registro y verificación con la nueva identidad sin
cambiar comportamiento.

**Independent Test**: ejecutar los cuatro recorridos de acceso y comparar mensajes, validaciones y
redirecciones con los de 001.

### Implementation for User Story 18.3

- [ ] T056 [US3] Reescribir la estructura de `frontend/src/routes/login/+page.svelte` con `LienzoAcceso`, `Campo`, `Boton`, `Aviso` y `BotonTema`, conservando `LoginSchema`, SSO y redirección
- [ ] T057 [US3] Re-pielar el modo de cambio obligatorio de contraseña en `frontend/src/routes/login/+page.svelte` conservando `CambioPasswordSchema` y sus mensajes
- [ ] T058 [P] [US3] Re-pielar `frontend/src/routes/registro/+page.svelte` con el mismo lienzo y `RegistroSchema`
- [ ] T059 [P] [US3] Re-pielar `frontend/src/routes/verificar-correo/+page.svelte` con `Aviso`, estados de éxito/error y enlaces actuales
- [ ] T060 [US3] Eliminar las reglas `.login-*` obsoletas de `frontend/src/routes/layout.css` y consolidar el estilo en `frontend/src/lib/diseno/acceso.css`
- [ ] T061 [US3] Generar evidencia de `/login`, cambio de contraseña, `/registro` y `/verificar-correo` en `specs/018-rediseno-visual-desktop/evidencia/{claro,oscuro}/` y el chequeo a 320 px
- [ ] T062 [US3] Revisar los escenarios 1–7 de US18.3 y registrar el resultado en `specs/018-rediseno-visual-desktop/evidencia/antes/estado.md`

**Checkpoint**: los cuatro recorridos de acceso están rediseñados y sin regresiones.

---

## Phase 6: User Story 18.4 - Operación diaria (Priority: P4)

**Goal**: llevar Caja, Movimientos, Transferencias, Conteos y Asistente al patrón de encabezado +
tarjetas + resultado/historial.

**Independent Test**: ejecutar la acción principal de cada página y verificar el resultado concreto y
los estados de error/carga.

### Implementation for User Story 18.4

- [ ] T063 [P] [US4] Rediseñar `frontend/src/routes/caja/+page.svelte` con `EncabezadoPagina`, `Tarjeta`, `TablaDatos` y `EstadoVacio`, conservando debounce y aviso de truncados
- [ ] T064 [P] [US4] Rediseñar `frontend/src/routes/movimientos/+page.svelte` con tarjeta de formulario, tarjeta de resultado e historial, conservando la creación y el stock resultante
- [ ] T065 [P] [US4] Rediseñar `frontend/src/routes/transferencias/+page.svelte` con el mismo patrón, conservando origen/destino, validaciones y paginación
- [ ] T066 [P] [US4] Rediseñar `frontend/src/routes/conteos/+page.svelte` con tarjetas, tabla de líneas, badges de estado y modales re-pielados
- [ ] T067 [P] [US4] Rediseñar `frontend/src/routes/asistente/+page.svelte` y `frontend/src/lib/components/asistente/TarjetaPropuesta.svelte` con carga, historial y confirmación
- [ ] T068 [US4] Verificar que las cinco páginas muestran el resultado concreto (producto, depósito, cantidad y stock resultante) y los errores reales de la API
- [ ] T069 [US4] Generar evidencia de las cinco páginas en `specs/018-rediseno-visual-desktop/evidencia/{claro,oscuro}/` y el chequeo a 320 px
- [ ] T070 [US4] Revisar los escenarios 1–6 de US18.4 y registrar el resultado en `specs/018-rediseno-visual-desktop/evidencia/antes/estado.md`

**Checkpoint**: la operación diaria está rediseñada y verificada.

---

## Phase 7: User Story 18.5 - Administración en tablas (Priority: P5)

**Goal**: llevar las ocho páginas de administración al patrón de encabezado + filtros + tabla, con
badges, acciones y paginación.

**Independent Test**: en cada página, listar, filtrar, paginar y ejecutar una acción de fila sin
cambios de comportamiento.

### Implementation for User Story 18.5

- [ ] T071 [P] [US5] Rediseñar `frontend/src/routes/admin/usuarios/+page.svelte` y `frontend/src/lib/components/usuarios/{UsuarioTabla,UsuarioForm,PasswordTemporalModal,CredencialesModal}.svelte`
- [ ] T072 [P] [US5] Rediseñar `frontend/src/routes/admin/categorias/+page.svelte` y `frontend/src/lib/components/categorias/{CategoriaTabla,CategoriaForm}.svelte`
- [ ] T073 [P] [US5] Rediseñar `frontend/src/routes/admin/productos/+page.svelte` y `frontend/src/lib/components/productos/{FiltrosCatalogo,ProductoTabla,ProductoForm}.svelte`
- [ ] T074 [US5] Rediseñar `frontend/src/routes/admin/depositos/+page.svelte` con las dos columnas de depósitos/ubicaciones y el panel de ubicaciones por producto
- [ ] T075 [P] [US5] Rediseñar `frontend/src/routes/admin/stock/+page.svelte` con filtros, énfasis de bajo mínimo y paginación
- [ ] T076 [P] [US5] Rediseñar `frontend/src/routes/admin/proveedores/+page.svelte` y `frontend/src/lib/components/proveedores/{ProveedorTabla,ProveedorForm}.svelte`
- [ ] T077 [P] [US5] Rediseñar `frontend/src/routes/admin/compras/+page.svelte` y `frontend/src/lib/components/compras/{CompraTabla,CompraForm}.svelte`
- [ ] T078 [P] [US5] Rediseñar `frontend/src/routes/admin/auditoria/+page.svelte` con filtros y tabla
- [ ] T079 [US5] Verificar en las ocho páginas los estados vacío/carga/error, los badges y el contador de paginación
- [ ] T080 [US5] Generar evidencia de las ocho páginas en `specs/018-rediseno-visual-desktop/evidencia/{claro,oscuro}/` y el chequeo a 320 px
- [ ] T081 [US5] Revisar los escenarios 1–7 de US18.5 y registrar el resultado en `specs/018-rediseno-visual-desktop/evidencia/antes/estado.md`

**Checkpoint**: las 17 capturas están rediseñadas; resta el cierre transversal.

---

## Phase 8: Polish & Cross-Cutting Concerns

**Purpose**: limpieza, accesibilidad, contraste, evidencia completa y documentación.

- [ ] T082 [P] Buscar y eliminar clases muertas y colores hexadecimales fuera de `frontend/src/lib/diseno/tokens.css` en `frontend/src/`
- [ ] T083 [P] Auditar contraste AA en ambos temas sobre `specs/018-rediseno-visual-desktop/contracts/tokens.md` §5 y ajustar tokens si hace falta
- [ ] T084 [P] Auditar teclado, foco, landmarks, `aria-current`, modales y `prefers-reduced-motion` en las 18 rutas
- [ ] T085 Ejecutar `npm run test`, `npm run check` y `npm run build` en `frontend/` y corregir cualquier fallo
- [ ] T086 Ejecutar `node scripts/capturar-rediseno.mjs` y verificar 18 capturas claras + 18 oscuras + 8 angostas en `specs/018-rediseno-visual-desktop/evidencia/`
- [ ] T087 [P] Actualizar `specs/README.md` (fila 018, FR-079…FR-097, SC-032…SC-037 y estado real) y `docs/arquitectura.md` con la nota del sistema de diseño
- [ ] T088 Revisar que no haya cambios en `backend/`, `migrations/`, `frontend/src/lib/api/*` existentes ni `frontend/src/lib/schemas/*` existentes (solo se agrega `reportes.ts`)
- [ ] T089 Recorrer el checklist de `specs/018-rediseno-visual-desktop/quickstart.md` §6 y dejar el resultado en `specs/018-rediseno-visual-desktop/evidencia/antes/estado.md`

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: sin dependencias; T003 depende de T002.
- **Foundational (Phase 2)**: depende de Setup; T008 depende de T007, T010 de T009, T012 de T011;
  T025–T029 dependen de T013–T024; T032 depende de T025–T031. **Bloquea todas las historias.**
- **US18.1 (Phase 3)**: depende de Foundational.
- **US18.2 (Phase 4)**: depende de Foundational; no depende de US18.1 para sus componentes, pero la
  página se ve completa recién con el marco.
- **US18.3 (Phase 5)**: depende de Foundational; puede ir en paralelo con US18.2.
- **US18.4 (Phase 6)**: depende de Foundational; puede ir en paralelo con US18.2/US18.3.
- **US18.5 (Phase 7)**: depende de Foundational; puede ir en paralelo con US18.4.
- **Polish (Phase 8)**: depende de todas las historias deseadas.

### User Story Dependencies

- **US18.1 (P1)**: sin dependencias de otras historias; es el MVP.
- **US18.2 (P2)**: integra el marco pero se prueba con datos; no bloquea a las demás.
- **US18.3 (P3)**: independiente; solo comparte tokens y primitivas.
- **US18.4 (P4)**: independiente; comparte `EncabezadoPagina`, `Tarjeta`, `TablaDatos`.
- **US18.5 (P5)**: independiente; comparte el mismo catálogo.

### Within Each User Story

- Tests primero (RED) y luego implementación (GREEN) en la lógica pura.
- Primitivas antes que páginas; páginas antes que evidencia.
- La evidencia se genera después de que la página compila y funciona.
- Cada página se cierra con su revisión de escenarios antes de pasar a la siguiente.

### Parallel Opportunities

- T002, T004 y T005 en Setup.
- T007/T009/T011 (tests) y T013–T024 (primitivas) en Foundational.
- US18.2, US18.3, US18.4 y US18.5 pueden ejecutarse en paralelo una vez cerrada la fase 2.
- Dentro de US18.5, las ocho páginas son paralelizables salvo T074 (Depósitos) por su tamaño.
- Las capturas de evidencia de cada historia son paralelizables entre sí.

---

## Parallel Example: Foundational

```text
# Tests puros en paralelo
Task: "T007 test de gruposVisibles en frontend/src/lib/navegacion/navegacion.test.ts"
Task: "T009 test de formato en frontend/src/lib/utils/formato.test.ts"
Task: "T011 test de gráficos en frontend/src/lib/utils/graficos.test.ts"

# Primitivas en paralelo
Task: "T013 Boton.svelte"
Task: "T014 Campo.svelte"
Task: "T021 TablaDatos.svelte"
Task: "T023 Modal.svelte"
```

## Parallel Example: User Story 18.5

```text
Task: "T071 Usuarios"
Task: "T072 Categorías"
Task: "T073 Productos"
Task: "T075 Stock"
Task: "T076 Proveedores"
Task: "T077 Compras"
Task: "T078 Auditoría"
```

---

## Implementation Strategy

### MVP First (User Story 18.1)

1. Completar Phase 1 (Setup).
2. Completar Phase 2 (Foundational) — bloquea todo.
3. Completar Phase 3 (US18.1).
4. **STOP y VALIDAR**: navegación, tema, RBAC y responsive en ambos temas.
5. Continuar con US18.2 (tablero) como segunda entrega visible.

### Incremental Delivery

1. Setup + Foundational → base visual y marco.
2. US18.1 → marco usable en todas las rutas (MVP).
3. US18.3 → acceso coherente (rápido, poca superficie).
4. US18.2 → tablero real.
5. US18.4 → operación diaria.
6. US18.5 → administración.
7. Polish → contraste, evidencia completa y documentación.

### Parallel Team Strategy

Con varias personas:

1. Todo el equipo cierra Setup + Foundational.
2. Luego: una persona en US18.1, otra en US18.3, otra en US18.2.
3. Al cerrar US18.1, se reparten US18.4 y US18.5 por página.
4. Polish se hace en conjunto al final.

---

## Notes

- [P] = archivos distintos y sin dependencia pendiente.
- [Story] mapea cada tarea a US18.1–US18.5 para trazabilidad.
- No se modifican endpoints, contratos, esquema ni reglas de negocio.
- Cada tarea de página cierra con `npm run check` y, si toca lógica, con su test.
- La evidencia se guarda con el nombre de la captura original para comparar directo.
- Evitar tareas vagas: cada una nombra el archivo y el resultado esperado.
