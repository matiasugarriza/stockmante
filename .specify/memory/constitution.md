<!-- Sync Impact Report
- Version change: 1.3.3 → 1.3.4 (PATCH)
  - PATCH: se registran los waivers reservados W-003 (012) y W-004 (008). No se agrega, elimina
    ni redefine ningún principio; el núcleo normativo I–V se conserva íntegro.
- Modified principles: ninguno
- Added sections: ninguno
- Removed sections: ninguno
- Other changes:
  - W-003: RED no observado en T006/T020/T023 de 012 (tests escritos junto con el código).
  - W-004: tests de 008 que pasaron en su primera corrida (T006, T031, T032 y los casos de falla
    de T007) porque verifican comportamiento ya implementado.
  - Se conservan W-001, W-002, W-005 y W-006 sin cambios.
- Templates: `plan-template.md`, `spec-template.md`, `tasks-template.md` sin cambios necesarios.
- Follow-up TODOs:
  - Condición de cierre de W-003 y W-004: suite de backend en verde y `cargo sqlx prepare --check`
    sin diferencias en los gates de 012 (T036) y 008 (T036).
-->

# Stockmante Constitution

## Core Principles

### I. Simplicidad y Anti-Sobreingeniería (NO NEGOCIABLE)

- Cada funcionalidad implementada DEBE resolver un requisito actual de una spec; NO SE agregan abstracciones, capas, dependencias ni configuraciones "por si acaso" (YAGNI).
- La solución más simple que cumpla la spec GANA. NO SE crean factory/strategy/plugin/eventos/colas sin una variación concreta y demostrada en el código existente.
- Cada módulo, servicio o componente DEBE tener un propósito único y acotado; si un archivo crece más allá de lo que su nombre promete, se divide o se simplifica.
- NO SE introducen dependencias nuevas sin justificación registrada en la spec o el PR: resolver primero con stdlib/stack ya presente.

Justificación: el mantenimiento y el testing de un sistema de inventario se degradan exponencialmente con cada abstracción innecesaria; la simplicidad es un requisito de calidad, no un lujo.

### II. Mantenibilidad (NO NEGOCIABLE)

- Frontend: componentes Svelte pequeños, reactivos y modulares; un componente con una sola responsabilidad visual o de interacción.
- Backend: handlers y servicios acotados; la lógica de negocio vive en funciones nombrables y extraíbles, no incrustada en routers.
- Todo acceso HTTP desde el frontend DEBE pasar por funciones centralizadas y tipadas (por ejemplo `$lib/api/*.ts`); NO SE escriben `fetch` dispersos en componentes.
- El código DEBE ser autoexplicativo: nombres descriptivos en español de negocio; los comentarios explican el "por qué", nunca el "qué".
- `panic!`, `unwrap()` y `expect()` PROHIBIDOS en código de producción del backend: toda falla se propaga como `Result<T, AppError>`.

Justificación: legibilidad y límites claros reducen el costo de cada cambio futuro y permiten que cualquier agente o humano retome el código sin contexto extra.

### III. Claridad en Mensajes (NO NEGOCIABLE)

- Todo mensaje mostrado al usuario —backend (API), chatbot o frontend— DEBE responder: qué pasó, por qué falló (si falló) y qué hacer a continuación.
- Los mensajes de error del backend, del chatbot y de la interfaz DEBEN ser específicos ("Stock insuficiente en Depósito Central: 3 disponibles, 5 solicitados"), nunca genéricos ("Error interno"), salvo la excepción de seguridad del bullet siguiente.
- **Excepción de seguridad — anti-enumeración en autenticación y registro**: `POST /api/auth/login` PUEDE devolver un mensaje y un código HTTP únicos e idénticos para email inexistente, contraseña incorrecta y cuenta inactiva, para no revelar la existencia de cuentas. `POST /api/auth/registro` PUEDE devolver el mismo mensaje y código HTTP para un correo ya registrado que para un registro exitoso, para no revelar la existencia de cuentas. La excepción se limita a esos endpoints, DEBE estar documentada en la spec del módulo y NO habilita mensajes genéricos en ningún otro endpoint ni oculta el motivo de un fallo de dominio.
- Cualquier confirmación de acción sobre inventario DEBE mostrar el resultado concreto (stock resultante, depósito, producto) después de ejecutarse.
- Las salidas destinadas a lectura programática (respuestas JSON de la API) DEBEN ser estables y sin adornos (sin emojis ni texto decorativo en el flujo de datos).

Justificación: el operador y el desarrollador trabajan desde la interfaz web y el chatbot; un mensaje ambiguo provoca errores operativos de inventario y debugging lento. La excepción de login es una decisión de seguridad acotada, no una licencia para mensajes pobres.

### IV. Testabilidad Primero (NO NEGOCIABLE)

- La lógica de negocio DEBE separarse de la I/O (HTTP, BD, reloj, red) para que pueda probarse con entradas y salidas deterministas.
- Toda funcionalidad con resultado esperado claro DEBE tener un test; ante un bug, primero se escribe el test que reproduce el fallo (RED), luego la corrección (GREEN), luego se refactoriza manteniendo la suite en verde.
- En frontend, la lógica de `$lib/api/*.ts` y de los stores DEBE tener tests; los componentes visuales se prueban si contienen lógica.
- NO SE aceptan dependencias ocultas de reloj, aleatoriedad o red sin inyección o mock explícito.
- Las consultas SQLx DEBEN ser verificables contra el esquema real (ver: Verificación de esquema y el quality gate `cargo sqlx prepare --check`); los tipos nullables se confirman con el esquema, no se suponen.

Justificación: el inventario es integridad de datos; los tests deterministas son la red de seguridad que permite cambiar sin miedo.

### V. Integridad de Datos y Manejo de Errores (NO NEGOCIABLE)

- Todo movimiento de stock (entrada, salida, merma, ajuste, transferencia) DEBE ejecutarse dentro de una transacción SQLx explícita (`pool.begin()` … `commit()`); sin excepciones, sin atajos fuera de transacción.
- Fuente de verdad del stock: `stock_por_deposito(producto_id, deposito_id, cantidad)`. `productos.stock_actual` es únicamente un cache denormalizado del total global y DEBE actualizarse en la MISMA transacción que la tabla puente.
- El stock NUNCA puede quedar negativo: guarda `CHECK (cantidad >= 0)` en la BD más guarda condicional en la consulta.
- ANTES de escribir cualquier SQL, migración o struct `FromRow`, el esquema real DEBE verificarse (ver: Verificación de esquema).
- Todo error de dominio DEBE mapearse a una variante específica de `AppError` (por ejemplo `StockInsuficiente`) con código HTTP 409 o 422. Los errores de infraestructura o persistencia DEBEN mapearse a una variante genérica de `AppError` con código 500, sin exponer detalles internos al cliente.

Justificación: un movimiento de stock parcial o corrupto es pérdida financiera directa e irreversible para el negocio.

## Restricciones Técnicas y Seguridad

Stack fijo (agente y humanos deben respetarlo):

- Frontend: SvelteKit en modo SPA + Tailwind CSS.
- Backend: Rust + Axum + SQLx (compile-time checked queries).
- Base de datos: PostgreSQL; migraciones SQL versionadas en `/migrations`.
- Asistente/Chatbot: LangChain + API de LLM con Tools.

Seguridad del chatbot (obligatoria):

- Lecturas (consultar stock, buscar productos): permitidas con credenciales de solo lectura.
- Escrituras (entrada/salida/merma/ajuste): NUNCA desde el chatbot contra la BD; SIEMPRE vía `POST /api/movimientos` de la API Rust, que valida autenticación, permisos (OPERARIO/ADMIN) y ejecuta la transacción atómica.
- El token de sesión lo inyecta el host autenticado; NUNCA se le pide al usuario que pegue tokens en el chat.

Verificación de esquema (fuente canónica — Principio IV, Principio V y el flujo SDD la referencian): para cualquier tarea que toque la BD, ANTES de escribir SQL, migración o struct `FromRow`, el esquema real DEBE verificarse contra la base de datos en ejecución (tablas, columnas, tipos y nullables). Las herramientas concretas para hacerlo (servidor MCP de PostgreSQL, `sqlx-cli`, u otras) se definen en la capa operativa (`AGENTS.md` / `SKILLS.md`), no en esta constitución.

## Flujo de Desarrollo (SDD)

Secuencia obligatoria para cada tarea:

1. **Especificación**: NO se escribe código sin verificar o redactar la spec en `specs/<NNN-feature>/spec.md`, generada con `/speckit.specify` y, cuando corresponda, completada con `/speckit.plan` y `/speckit.tasks`.
2. **Verificación de esquema**: contrastar la spec contra el esquema real de la BD antes de generar código (ver: Verificación de esquema).
3. **Patrones operativos**: aplicar los patrones del catálogo (`SKILLS.md`); toda tarea que toque la BD comienza con la verificación de esquema.
4. **Implementación incremental**: cambios pequeños, verificando que no rompan código previo; tests y docs junto al cambio (unidad de trabajo = commit).
5. **Memoria**: consultar y actualizar la memoria persistente del proyecto antes y después de cada sesión de desarrollo (decisiones de arquitectura, gotchas, convenciones).

Quality gates antes de considerar terminada una tarea:

- `cargo test` (backend) en verde y `cargo sqlx prepare --check` sin diferencias; sin `unwrap`/`panic!` en rutas de producción.
- Suite frontend en verde; toda tarea cumple el Principio IV (y III, para mensajes al usuario) antes de cerrarse.
- Spec actualizada si el comportamiento cambió respecto de lo planificado.

## Governance

- Esta constitución prevalece sobre otras prácticas, guías o instrucciones en caso de conflicto; `AGENTS.md` y `SKILLS.md` son guía operativa y DEBEN alinearse con estos principios, no contradecirlos.
- **Enmiendas**: se realizan editando `.specify/memory/constitution.md`, con un informe de impacto (Sync Impact Report) justificando el cambio y un incremento de versión según semántica descrita abajo; el commit del cambio usa mensaje `docs: amend constitution to vX.Y.Z (...)`.
- **Versionado**: MAJOR = remoción o redefinición incompatible de un principio; MINOR = nuevo principio o sección, o expansión material de una guía; PATCH = clarificaciones, correcciones de redacción, cambios no semánticos.
- **Cumplimiento**: cada PR y cada revisión DEBE verificar el cumplimiento de estos principios; cualquier complejidad añadida DEBE justificarse; las violaciones se corrigen en el mismo PR, se elevan como enmienda formal o se registran como waiver acotado en la sección siguiente (nunca se ignoran por costumbre).

### Waivers registrados

Un waiver es una excepción histórica y acotada, aprobada y registrada. NO modifica el principio, NO se hereda a trabajo futuro y NO se invoca como precedente.

- **W-001 — P-IV: RED no observado en T017–T019 (aprobado 2026-10-25)**: los tests de US1 (`test_auth_login`, `test_sesion_unica`, `test_cambio_password`) se escribieron junto con el servicio y no registraron el fallo previo. Alcance: solo esas tres tareas de `specs/001-autenticacion-sesion/tasks.md` (T032). Condición de cierre: suite de backend en verde y `cargo sqlx prepare --check` sin diferencias, verificado el 2026-10-25. Desde `003-catalogo-productos` en adelante el RED es obligatorio y no existe waiver preventivo.

- **W-002 — P-IV: RED no observado en T010/T015/T016 de 003 (aprobado 2026-11-06)**: los tests unitarios de normalización/validación (`test_categoria_normalizacion`, `test_sku`, `test_producto_validacion`) se escribieron después de T006 y quedaron verdes en su primera corrida; `progress.md` lo registra. El RED sí se observó en los tests HTTP de T011/T017/T018. Alcance: solo esas tres tareas de `specs/003-catalogo-productos/tasks.md`. No se reconstruye un RED artificial. Condición de cierre: suite de backend en verde y `cargo sqlx prepare --check` sin diferencias, verificado el 2026-11-06.

- **W-003 — P-IV: RED no observado en T006/T020/T023 de 012 (aprobado 2026-11-07)**: los tests de proveedores y compras de esas tareas se escribieron junto con el servicio y el handler, sin registrar el fallo previo. Alcance: solo T006, T020 y T023 de `specs/012-proveedores-compras/tasks.md`; desde la fase 6 (T024 en adelante) el RED es obligatorio. No se reconstruye un RED artificial; `specs/012-proveedores-compras/progress.md` DEBE registrarlo. Condición de cierre: suite de backend en verde y `cargo sqlx prepare --check` sin diferencias en T036 de 012.

- **W-004 — P-IV: tests de 008 verdes en su primera corrida (aprobado 2026-11-07)**: T006 (`test_transferencia_validacion`), los casos de falla de T007 y T031 (`fallas_no_escriben_nada`) y T032 (OpenAPI de transferencias) verifican reglas que ya estaban implementadas y pasaron en verde la primera vez. El RED sí se observó en el resto de `test_transferencias_http` (500 por columnas inexistentes y alias reservado, 404 sin siguiente paso, desborde sin control). Alcance: solo esas tareas de `specs/008-transferencias-depositos/tasks.md`. No se reconstruye un RED artificial; `specs/008-transferencias-depositos/progress.md` lo registra. Condición de cierre: suite de backend en verde y `cargo sqlx prepare --check` sin diferencias en T036 de 008.

- **W-005 — P-IV: RED no observado en T011/T012, T025 y T033 de 007 (aprobado 2026-11-07)**: los tests históricos de movimientos (`test_movimiento_validacion` en T011 y `test_movimientos_http` en T012) se escribieron junto con el servicio y no dejaron evidencia del fallo previo. T025 (concurrencia: dos altas simultáneas del mismo producto en ubicaciones distintas) y T033 (filtro `categoria_id` de `GET /api/stock`) verifican comportamiento ya implementado, así que pueden pasar en verde en su primera corrida. Alcance: solo esas cuatro tareas de `specs/007-movimientos-stock/tasks.md`; el resto de la ampliación (T021–T024, T031, T032) DEBE observar RED antes de T026. No se reconstruye un RED artificial; `specs/007-movimientos-stock/progress.md` DEBE registrar el estado de cada una. Condición de cierre: suite de backend en verde y `cargo sqlx prepare --check` sin diferencias en T030 (pendiente de verificación).

- **W-006 — P-V y P-IV: verificación de esquema retroactiva y RED no registrado en 004 (aprobado 2026-11-07)**: la verificación del esquema real de `productos` y `categorias` (T000) se registró el 2026-11-07, después de la implementación del 2026-10-26, y confirmó que el esquema coincide con `data-model.md`. Los tests de T003 (`test_busqueda`), T006 (`test_busqueda_http`) y T009 (schema y cliente de `buscarProductos`) existen y están verdes, pero `progress.md` no registró su fallo previo. Alcance: solo T000, T003, T006 y T009 de `specs/004-busqueda-caja/tasks.md`; la ampliación de 004 (T018 en adelante) DEBE observar RED y la verificación de esquema de T021 DEBE preceder a T022. No se reconstruye evidencia artificial; `specs/004-busqueda-caja/progress.md` DEBE registrar el waiver. Condición de cierre: suite de backend en verde y `cargo sqlx prepare --check` sin diferencias en T026 (pendiente de verificación).

**Version**: 1.3.4 | **Ratified**: 2026-10-11 | **Last Amended**: 2026-11-07
