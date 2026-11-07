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
- **[Story]**: `US1`…`US5`, equivalentes a US18.1…US18.5 de `spec.md`
- Cada tarea incluye la ruta exacta del archivo o carpeta

## Path Conventions

Aplicación web existente: `frontend/src/` para SvelteKit y `scripts/` para herramientas. La evidencia
vive en `specs/018-rediseno-visual-desktop/evidencia/`.

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: preparar tokens, iconografía y la herramienta de evidencia antes de tocar pantallas.

- [X] T001 Auditar las 17 capturas de `captures/` contra `captures/samples/light-2.png` y `dark-2.png` y confirmar el inventario de rutas/roles de `specs/018-rediseno-visual-desktop/data-model.md`
- [X] T002 [P] Crear `frontend/src/lib/diseno/tokens.css` con los tokens claro/oscuro, tipografía, radios, sombras y breakpoints de `specs/018-rediseno-visual-desktop/contracts/tokens.md`
- [X] T003 Importar `frontend/src/lib/diseno/tokens.css` en `frontend/src/routes/layout.css` y retirar las variables crema/naranja del `:root`
- [X] T004 [P] Crear `frontend/src/lib/components/ui/Icono.svelte` con los 20 iconos SVG definidos en `specs/018-rediseno-visual-desktop/contracts/ui.md`
- [X] T005 [P] Crear `scripts/capturar-rediseno.mjs` con Playwright: login con `CAPTURE_EMAIL`/`CAPTURE_PASSWORD` del entorno, recorrido de las 18 pantallas (17 rutas + modo de cambio de contraseña), temas claro/oscuro a 1440×900 y chequeo a 320 px, salida en `specs/018-rediseno-visual-desktop/evidencia/`
- [X] T006 Ejecutar la suite base en `frontend/` (`npm run test`, `npm run check`) y registrar el resultado de partida en `specs/018-rediseno-visual-desktop/evidencia/validacion.md`

**Checkpoint**: existen tokens, iconos y la herramienta de capturas; la suite base está registrada.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: lógica pura testeada, primitivas visuales y marco. Sin esto no se puede rediseñar
ninguna página.

**⚠️ CRITICAL**: ninguna historia de usuario puede comenzar hasta completar esta fase.

### Lógica pura

- [X] T007 [P] Escribir el test que falla de `gruposVisibles` (RBAC, grupo vacío, orden, ruta activa, contador) en `frontend/src/lib/navegacion/navegacion.test.ts`
- [X] T008 Implementar `frontend/src/lib/navegacion/navegacion.ts` hasta GREEN con el contrato de `specs/018-rediseno-visual-desktop/contracts/ui.md` §2
- [X] T009 [P] Escribir el test que falla de formato de moneda ARS, enteros, porcentaje y fecha corta en `frontend/src/lib/utils/formato.test.ts`
- [X] T010 Implementar `frontend/src/lib/utils/formato.ts` hasta GREEN
- [X] T011 [P] Escribir el test que falla de escalas de barras, arcos de dona, porcentajes y casos cero/negativo en `frontend/src/lib/utils/graficos.test.ts`
- [X] T012 Implementar `frontend/src/lib/utils/graficos.ts` hasta GREEN

### Primitivas visuales

- [X] T013 [P] Crear `frontend/src/lib/components/ui/Boton.svelte` con variantes, tamaño, cargando y foco visible
- [X] T014 [P] Crear `frontend/src/lib/components/ui/Campo.svelte` con label, ayuda, error y `aria-describedby`
- [X] T015 [P] Crear `frontend/src/lib/components/ui/EncabezadoPagina.svelte` con título, descripción y snippet de acción
- [X] T016 [P] Crear `frontend/src/lib/components/ui/Tarjeta.svelte` con encabezado, descripción, acción y pie opcionales
- [X] T017 [P] Crear `frontend/src/lib/components/ui/EtiquetaEstado.svelte` con variantes éxito/aviso/peligro/info/neutro
- [X] T018 [P] Crear `frontend/src/lib/components/ui/Aviso.svelte` con variantes y mensaje accionable
- [X] T019 [P] Crear `frontend/src/lib/components/ui/EstadoVacio.svelte` con icono, texto y acción
- [X] T020 [P] Crear `frontend/src/lib/components/ui/Esqueleto.svelte` con variantes línea/bloque/tarjeta
- [X] T021 [P] Crear `frontend/src/lib/components/ui/TablaDatos.svelte` con scroll interno, carga, error, vacío y filas por snippet
- [X] T022 [P] Crear `frontend/src/lib/components/ui/Paginacion.svelte` con total, página y estados deshabilitados
- [X] T023 [P] Crear `frontend/src/lib/components/ui/Modal.svelte` con foco atrapado, Escape, backdrop y scroll lock
- [X] T024 [P] Crear `frontend/src/lib/components/ui/BotonTema.svelte` conectado a `frontend/src/lib/tema/tema.ts`

### Marco

- [X] T025 Crear `frontend/src/lib/components/marco/ItemNavegacion.svelte` con icono, etiqueta, `aria-current` y contador opcional
- [X] T026 [P] Crear `frontend/src/lib/components/marco/MenuUsuario.svelte` con nombre, rol, tema y cierre de sesión
- [X] T027 [P] Crear `frontend/src/lib/components/marco/BarraSuperior.svelte` con botón de menú, contexto de página y menú de usuario
- [X] T028 [P] Crear `frontend/src/lib/components/marco/BarraLateral.svelte` que renderice `gruposVisibles` y el selector de tema
- [X] T029 Crear `frontend/src/lib/components/marco/MarcoApp.svelte` con sidebar, topbar, `<main id="contenido">` y estado colapsado/superpuesto
- [X] T030 Crear `frontend/src/lib/diseno/acceso.css` con el lienzo y la tarjeta de acceso usando los tokens nuevos, e importarlo desde `frontend/src/routes/layout.css`
- [X] T031 Crear `frontend/src/lib/components/marco/LienzoAcceso.svelte` con marca, panel, selector de tema y contenido
- [X] T032 Reescribir `frontend/src/routes/+layout.svelte` para decidir entre `MarcoApp` y `LienzoAcceso`, conservando `refrescarSesion`, alertas de stock, redirección y `admin/+layout.svelte`
- [X] T033 Verificar `frontend/src/app.html` y `frontend/src/lib/tema/tema.ts`: la clave `stockmante:tema`, `data-tema` y el anti-destello siguen vigentes; ajustar solo si el rediseño lo exige

**Checkpoint**: tokens, primitivas, navegación y marco compilan; `npm run test` y `npm run check` en
verde. Las historias pueden comenzar.

---

## Phase 3: User Story 18.1 - Marco de escritorio y tema (Priority: P1) 🎯 MVP

**Goal**: que cualquier ruta privada se abra dentro del marco, con navegación por rol, tema
persistido y comportamiento responsive.

**Independent Test**: iniciar sesión como GESTION y OPERACION, recorrer la navegación, cambiar de
tema, recargar y reducir la ventana a 320 px sin desborde.

### Implementation for User Story 18.1

- [X] T034 [US1] Conectar `BarraLateral.svelte` con `gruposVisibles`, `$sesion` y `alertasBajoMinimo` en `frontend/src/routes/+layout.svelte`
- [X] T035 [US1] Implementar el estado activo por ruta y el contador de Stock en `frontend/src/lib/components/marco/BarraLateral.svelte`
- [X] T036 [US1] Integrar `BotonTema.svelte` en la barra superior y en el lienzo de acceso, verificando `light`/`dark`/`system` y la persistencia
- [X] T037 [US1] Implementar colapso (≥1024), panel superpuesto (<1024), backdrop y cierre con Escape en `frontend/src/lib/components/marco/MarcoApp.svelte` y `BarraLateral.svelte`
- [X] T038 [US1] Agregar landmarks, enlace "Saltar al contenido", foco visible y `aria-current` en `frontend/src/lib/components/marco/` y `frontend/src/routes/+layout.svelte`
- [X] T039 [US1] Verificar el guard de gestión en `frontend/src/routes/admin/+layout.svelte` y la redirección de OPERACION a `/caja`
- [X] T040 [US1] Correr `npm run test` y `npm run check` en `frontend/` y corregir regresiones del marco
- [X] T041 [US1] Generar evidencia del marco en `specs/018-rediseno-visual-desktop/evidencia/{claro,oscuro}/` para `/`, `/caja`, `/movimientos`, `/admin/productos` y `/login`, más chequeo a 320 px en `evidencia/angosto/`
- [ ] T042 [US1] Revisar manualmente los escenarios 1–7 de US18.1 en `specs/018-rediseno-visual-desktop/spec.md` y anotar el resultado en `specs/018-rediseno-visual-desktop/evidencia/validacion.md`

**Checkpoint**: US18.1 funciona sola; el resto de las páginas conserva su contenido interno anterior
pero ya vive en el marco.

---

## Phase 4: User Story 18.2 - Tablero de Inicio (Priority: P2)

**Goal**: reemplazar la pantalla mínima de Inicio por un tablero real con indicadores, gráficos,
actividad y accesos rápidos.

**Independent Test**: comparar cada número y gráfico con la API; verificar widgets ocultos para
OPERACION y errores independientes.

### Implementation for User Story 18.2

- [X] T043 [P] [US2] Crear `frontend/src/lib/schemas/reportes.ts` con el schema de stock valorizado (`items`, `total`) y su tipo
- [X] T044 [P] [US2] Escribir el test que falla del cliente `reportes` en `frontend/src/lib/api/reportes.test.ts` (URL, método, parseo y error)
- [X] T045 [P] [US2] Implementar `frontend/src/lib/api/reportes.ts` con `obtenerStockValorizado()` usando `api()` de `frontend/src/lib/api/client.ts`, hasta GREEN
- [X] T046 [P] [US2] Crear `frontend/src/lib/components/ui/TarjetaKpi.svelte` con etiqueta, métrica tabular, contexto y estado de carga
- [X] T047 [P] [US2] Crear `frontend/src/lib/components/ui/GraficoBarras.svelte` usando `escalaBarras` de `frontend/src/lib/utils/graficos.ts`
- [X] T048 [P] [US2] Crear `frontend/src/lib/components/ui/GraficoDona.svelte` usando `arcoDona` de `frontend/src/lib/utils/graficos.ts`
- [X] T049 [P] [US2] Crear `frontend/src/lib/components/inicio/PanelActividad.svelte` con últimos movimientos y enlace a `/movimientos`
- [X] T050 [P] [US2] Crear `frontend/src/lib/components/inicio/AccesosRapidos.svelte` con accesos por rol
- [X] T051 [US2] Reescribir `frontend/src/routes/+page.svelte` con encabezado, KPIs, gráficos, actividad y accesos rápidos
- [X] T052 [US2] Implementar la carga independiente por widget, el ocultamiento por rol y por endpoint no disponible (403/404), y el reintento en `frontend/src/routes/+page.svelte`
- [X] T053 [US2] Implementar estados vacíos y esqueletos del tablero en `frontend/src/routes/+page.svelte` y `frontend/src/lib/components/inicio/`
- [ ] T054 [US2] Verificar los totales contra `GET /api/stock`, `GET /api/movimientos`, `GET /api/productos` y `GET /api/reportes/stock-valorizado`, y que un 403/404 de este último oculte el KPI sin romper el tablero
- [X] T055 [US2] Generar evidencia de `/` en `specs/018-rediseno-visual-desktop/evidencia/{claro,oscuro}/04-inicio.png` y el chequeo a 320 px

**Checkpoint**: Inicio es un tablero autónomo y verificado; las páginas de operación y administración
siguen pendientes.

---

## Phase 5: User Story 18.3 - Acceso rediseñado (Priority: P3)

**Goal**: re-pielar login, cambio obligatorio, registro y verificación con la nueva identidad sin
cambiar comportamiento.

**Independent Test**: ejecutar los cuatro recorridos de acceso y comparar mensajes, validaciones y
redirecciones con los de 001.

### Implementation for User Story 18.3

- [X] T056 [US3] Reescribir la estructura de `frontend/src/routes/login/+page.svelte` con `LienzoAcceso`, `Campo`, `Boton`, `Aviso` y `BotonTema`, conservando `LoginSchema`, SSO y redirección
- [X] T057 [US3] Re-pielar el modo de cambio obligatorio de contraseña en `frontend/src/routes/login/+page.svelte` conservando `CambioPasswordSchema` y sus mensajes
- [X] T058 [P] [US3] Re-pielar `frontend/src/routes/registro/+page.svelte` con el mismo lienzo y `RegistroSchema`
- [X] T059 [P] [US3] Re-pielar `frontend/src/routes/verificar-correo/+page.svelte` con `Aviso`, estados de éxito/error y enlaces actuales
- [X] T060 [US3] Eliminar las reglas `.login-*` obsoletas de `frontend/src/routes/layout.css` y consolidar el estilo en `frontend/src/lib/diseno/acceso.css`
- [ ] T061 [US3] (parcial: `/login`, `/registro` y `/verificar-correo` capturados; falta el modo de cambio de contraseña) Generar evidencia de `/login`, cambio de contraseña, `/registro` y `/verificar-correo` en `specs/018-rediseno-visual-desktop/evidencia/{claro,oscuro}/` y el chequeo a 320 px
- [ ] T062 [US3] Revisar los escenarios 1–7 de US18.3 y registrar el resultado en `specs/018-rediseno-visual-desktop/evidencia/validacion.md`

**Checkpoint**: los cuatro recorridos de acceso están rediseñados y sin regresiones.

---

## Phase 6: User Story 18.4 - Operación diaria (Priority: P4)

**Goal**: llevar Caja, Movimientos, Transferencias, Conteos y Asistente al patrón de encabezado +
tarjetas + resultado/historial.

**Independent Test**: ejecutar la acción principal de cada página y verificar el resultado concreto y
los estados de error/carga.

### Implementation for User Story 18.4

- [X] T063 [P] [US4] Rediseñar `frontend/src/routes/caja/+page.svelte` con `EncabezadoPagina`, `Tarjeta`, `TablaDatos` y `EstadoVacio`, conservando debounce y aviso de truncados
- [X] T064 [P] [US4] Rediseñar `frontend/src/routes/movimientos/+page.svelte` con tarjeta de formulario, tarjeta de resultado e historial, conservando la creación y el stock resultante
- [X] T065 [P] [US4] Rediseñar `frontend/src/routes/transferencias/+page.svelte` con el mismo patrón, conservando origen/destino, validaciones y paginación
- [X] T066 [P] [US4] Rediseñar `frontend/src/routes/conteos/+page.svelte` con tarjetas, tabla de líneas, badges de estado y modales re-pielados
- [X] T067 [P] [US4] Rediseñar `frontend/src/routes/asistente/+page.svelte` y `frontend/src/lib/components/asistente/TarjetaPropuesta.svelte` con carga, historial y confirmación
- [ ] T068 [US4] Verificar que las cinco páginas muestran el resultado concreto (producto, depósito, cantidad y stock resultante) y los errores reales de la API
- [X] T069 [US4] Generar evidencia de las cinco páginas en `specs/018-rediseno-visual-desktop/evidencia/{claro,oscuro}/` y el chequeo a 320 px
- [ ] T070 [US4] Revisar los escenarios 1–6 de US18.4 y registrar el resultado en `specs/018-rediseno-visual-desktop/evidencia/validacion.md`

**Checkpoint**: la operación diaria está rediseñada y verificada.

---

## Phase 7: User Story 18.5 - Administración en tablas (Priority: P5)

**Goal**: llevar las ocho páginas de administración al patrón de encabezado + filtros + tabla, con
badges, acciones y paginación.

**Independent Test**: en cada página, listar, filtrar, paginar y ejecutar una acción de fila sin
cambios de comportamiento.

### Implementation for User Story 18.5

- [X] T071 [P] [US5] Rediseñar `frontend/src/routes/admin/usuarios/+page.svelte` y `frontend/src/lib/components/usuarios/{UsuarioTabla,UsuarioForm,PasswordTemporalModal,CredencialesModal}.svelte`
- [X] T072 [P] [US5] Rediseñar `frontend/src/routes/admin/categorias/+page.svelte` y `frontend/src/lib/components/categorias/{CategoriaTabla,CategoriaForm}.svelte`
- [X] T073 [P] [US5] Rediseñar `frontend/src/routes/admin/productos/+page.svelte` y `frontend/src/lib/components/productos/{FiltrosCatalogo,ProductoTabla,ProductoForm}.svelte`
- [X] T074 [US5] Rediseñar `frontend/src/routes/admin/depositos/+page.svelte` con las dos columnas de depósitos/ubicaciones y el panel de ubicaciones por producto
- [X] T075 [P] [US5] Rediseñar `frontend/src/routes/admin/stock/+page.svelte` con filtros, énfasis de bajo mínimo y paginación
- [X] T076 [P] [US5] Rediseñar `frontend/src/routes/admin/proveedores/+page.svelte` y `frontend/src/lib/components/proveedores/{ProveedorTabla,ProveedorForm}.svelte`
- [X] T077 [P] [US5] Rediseñar `frontend/src/routes/admin/compras/+page.svelte` y `frontend/src/lib/components/compras/{CompraTabla,CompraForm}.svelte`
- [X] T078 [P] [US5] Rediseñar `frontend/src/routes/admin/auditoria/+page.svelte` con filtros y tabla
- [ ] T079 [US5] Verificar en las ocho páginas los estados vacío/carga/error, los badges y el contador de paginación
- [X] T080 [US5] Generar evidencia de las ocho páginas en `specs/018-rediseno-visual-desktop/evidencia/{claro,oscuro}/` y el chequeo a 320 px
- [ ] T081 [US5] Revisar los escenarios 1–7 de US18.5 y registrar el resultado en `specs/018-rediseno-visual-desktop/evidencia/validacion.md`

**Checkpoint**: las 17 capturas están rediseñadas; resta el cierre transversal.

---

## Phase 8: Polish & Cross-Cutting Concerns

**Purpose**: limpieza, accesibilidad, contraste, evidencia completa y documentación.

- [X] T082 [P] Buscar y eliminar clases muertas y colores hexadecimales fuera de `frontend/src/lib/diseno/tokens.css` en `frontend/src/`
- [X] T083 [P] Auditar contraste AA en ambos temas sobre `specs/018-rediseno-visual-desktop/contracts/tokens.md` §5 y ajustar tokens si hace falta
- [ ] T084 [P] Auditar teclado, foco, landmarks, `aria-current`, modales y `prefers-reduced-motion` en las 17 rutas y el modo de cambio de contraseña
- [X] T085 Ejecutar `npm run test`, `npm run check` y `npm run build` en `frontend/` y corregir cualquier fallo
- [ ] T086 (parcial 2026-11-07: 17 claras + 17 oscuras + 12 angostas generadas; falta el modo de cambio de contraseña, que requiere `CAPTURE_EMAIL_TEMP`/`CAPTURE_PASSWORD_TEMP`) Ejecutar `node scripts/capturar-rediseno.mjs` y verificar 18 capturas claras + 18 oscuras + 8 angostas en `specs/018-rediseno-visual-desktop/evidencia/`
- [X] T087 [P] Actualizar `specs/README.md` (fila 018, FR-079…FR-097, SC-032…SC-037 y estado real) y `docs/arquitectura.md` con la nota del sistema de diseño
- [ ] T088 Revisar que no haya cambios en `backend/`, `migrations/`, `frontend/src/lib/api/*` existentes ni `frontend/src/lib/schemas/*` existentes (solo se agrega `reportes.ts`)
- [ ] T089 Recorrer el checklist de `specs/018-rediseno-visual-desktop/quickstart.md` §6 y dejar el resultado en `specs/018-rediseno-visual-desktop/evidencia/validacion.md`

---

## Phase 9: Tipografía Outfit y segunda pasada de fidelidad (Session 2026-11-07 (2))

**Purpose**: aplicar FR-098 (Outfit) y FR-099 (rasgos del marco que faltaban frente a
`light-2`/`dark-2`), sin cambios de comportamiento.

- [X] T090 Instalar `@fontsource-variable/outfit` (versión exacta) en `frontend/package.json` e importarlo en `frontend/src/routes/layout.css`
- [X] T091 Cambiar `--fuente-sans` a `'Outfit Variable'` + pila del sistema en `frontend/src/lib/diseno/tokens.css` y `contracts/tokens.md` §2; reemplazar la monoespaciada decorativa (kicker y pie del acceso, versión de la barra lateral) por la sans en `frontend/src/lib/diseno/acceso.css`, `LienzoAcceso.svelte` y `BarraLateral.svelte`
- [X] T092 Ajustar el acento primario a `#1F3BB3` (claro) y `#3F5EFB` con texto blanco (oscuro) en `frontend/src/lib/diseno/tokens.css`, recalculando contraste AA en `contracts/tokens.md` §5
- [X] T093 Barra superior azul marino en tema claro con tokens de zona oscura (`.zona-oscura`) y saludo personalizado en Inicio en `frontend/src/lib/components/marco/BarraSuperior.svelte`
- [X] T094 Ítem activo en píldora apoyada en el borde y marca bicolor en `frontend/src/lib/components/marco/{ItemNavegacion,BarraLateral}.svelte`; corregir la superposición del pie (selector de tema + versión)
- [X] T095 Pie de página del marco en `frontend/src/lib/components/marco/MarcoApp.svelte`
- [X] T096 Indicadores de Inicio como franja de métricas (variante `plano` de `TarjetaKpi.svelte`) en `frontend/src/routes/+page.svelte`
- [X] T097 Ejecutar `npm run test`, `npm run check` y `npm run build` en `frontend/`
- [X] T098 Verificar SC-038 en el navegador (fuente calculada, `document.fonts.check`, sin peticiones a dominios externos) y regenerar la evidencia de `evidencia/{claro,oscuro}/`
- [X] T099 Registrar el resultado en `specs/018-rediseno-visual-desktop/evidencia/validacion.md`

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

## Trazabilidad requisitos → tareas

| Requisito | Tareas |
|---|---|
| FR-079 | T002, T003, T038, T082 |
| FR-080 | T024, T033, T036, T061 |
| FR-081 | T056 |
| FR-082 | T018, T019, T020, T079, T083 |
| FR-083 | T057 |
| FR-084 | T037, T041, T080 |
| FR-085 | T038, T084 |
| FR-086 | T001, T002, T013–T024 |
| FR-087 | T041, T055, T061, T069, T080 |
| FR-088 | T029, T032 |
| FR-089 | T007, T008, T034, T035, T039 |
| FR-090 | T043–T055 |
| FR-091 | T063–T067 |
| FR-092 | T071–T078 |
| FR-093 | T013–T024, T046–T048 |
| FR-094 | T058, T059 |
| FR-095 | T026, T033, T036 |
| FR-096 | T005, T041, T055, T061, T069, T080, T086 |
| FR-097 | T006, T040, T085, T088 |
| SC-032 | T033 |
| SC-033 | T056–T062 |
| SC-034 | T037, T083, T084 |
| SC-035 | T001, T041, T086 |
| SC-036 | T041, T055, T061, T069, T080 |
| SC-037 | T001, T034, T071–T078 |
| FR-098 | T090, T091 |
| FR-099 | T092–T096 |
| SC-038 | T098 |

---

## Notes

- [P] = archivos distintos y sin dependencia pendiente.
- [Story] mapea cada tarea a US18.1–US18.5 para trazabilidad.
- No se modifican endpoints, contratos, esquema ni reglas de negocio.
- Cada tarea de página cierra con `npm run check` y, si toca lógica, con su test.
- La evidencia se guarda con el nombre de la captura original para comparar directo.
- Evitar tareas vagas: cada una nombra el archivo y el resultado esperado.
