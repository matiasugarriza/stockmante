<!-- Sync Impact Report
- Version change: 1.0.0 → 1.1.0 (MINOR, luego corregido en 1.1.1) → 1.1.1 (PATCH)
  - Razonamiento MAJOR vs MINOR del Principio III en 1.1.0: se consideró el
    mismo principio con alcance ampliado, no una redefinición incompatible. El
    núcleo normativo original ("mensajes claros para el operador/desarrollador,
    errores específicos, no genéricos") se conservó íntegro; lo eliminado
    (stdout/stderr de CLI, salidas machine-readable binarias) era una asunción
    de herramienta que violaba el Principio I (YAGNI) porque el stack no expone
    CLI. Por eso MINOR y no MAJOR. De todos modos, el cambio de título
    (Terminal → Mensajes) y la eliminación de la definición CLI son materiales
    y quedan documentados aquí como ampliación de alcance.
  - 1.1.1 (PATCH): correcciones posteriores a la revisión de 1.1.0 — mapeo
    HTTP de errores de dominio vs persistencia en el Principio V; referencia
    al quality gate en el Principio IV; eliminación de la última mención a
    una herramienta (Engram) en el flujo SDD; este párrafo de razonamiento.
- Modified principles:
  - I. Simplicidad y Anti-Sobreingeniería — bullet roto unificado; etiqueta de
    título estandarizada a NO NEGOCIABLE.
  - II. Mantenibilidad — solo estandarización de etiqueta y "Justificación:".
  - III. Claridad en Terminal (UX) → III. Claridad en Mensajes — eliminada la
    asunción de CLI (stdout/stderr/salidas machine-readable); conserva y amplía
    mensajes del backend, chatbot y frontend.
  - IV. Testabilidad Primero — agregado criterio de tests de frontend
    ($lib/api/*.ts y stores); verificación de esquema referenciada, no
    duplicada; 1.1.1 agrega referencia al quality gate cargo sqlx prepare --check.
  - V. Integridad de Datos y Manejo de Errores — eliminadas referencias a
    herramientas/habilidades (MCP, SKILLS.md); 1.1.1 separa mapeo HTTP:
    errores de dominio → AppError específica con 409/422; infraestructura o
    persistencia → AppError genérica con 500 sin exponer detalles internos.
- Added sections: none
- Removed sections: none (Verificación de esquema queda como única fuente
  canónica dentro de Restricciones Técnicas; Principio IV, V y el paso 2 del
  SDD la referencian)
- Other changes:
  - Flujo SDD: ruta de specs alineada con Spec Kit (specs/<NNN-feature>/spec.md
    vía /speckit.specify, plan/tasks cuando corresponda); sin numeración de
    herramientas en los pasos; 1.1.1 elimina la última mención a Engram
    (quedó solo "la memoria persistente del proyecto"; Engram vive en AGENTS.md).
  - Quality gates: cargo sqlx prepare → cargo sqlx prepare --check; suite
    frontend vinculada al Principio III/IV.
  - Idioma: "Rationale:" → "Justificación:"; "(NON-NEGOTIABLE)" →
    "(NO NEGOCIABLE)" en los cinco títulos; eliminada la frase introductoria
    "Reglas no negociables:" de cada lista (convención única: etiqueta en el
    título).
- Follow-up TODOs: none
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
- Los mensajes de error del backend, del chatbot y de la interfaz DEBEN ser específicos ("Stock insuficiente en Depósito Central: 3 disponibles, 5 solicitados"), nunca genéricos ("Error interno").
- Cualquier confirmación de acción sobre inventario DEBE mostrar el resultado concreto (stock resultante, depósito, producto) después de ejecutarse.
- Las salidas destinadas a lectura programática (respuestas JSON de la API) DEBEN ser estables y sin adornos (sin emojis ni texto decorativo en el flujo de datos).

Justificación: el operador y el desarrollador trabajan desde la interfaz web y el chatbot; un mensaje ambiguo provoca errores operativos de inventario y debugging lento.

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
- **Cumplimiento**: cada PR y cada revisión DEBE verificar el cumplimiento de estos principios; cualquier complejidad añadida DEBE justificarse; las violaciones se corrigen en el mismo PR o se elevan como enmienda formal (nunca se ignoran por costumbre).

**Version**: 1.1.1 | **Ratified**: 2026-10-11 | **Last Amended**: 2026-10-12
