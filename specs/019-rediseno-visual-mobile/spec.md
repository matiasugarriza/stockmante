# Especificación de Funcionalidad: Rediseño visual móvil en modo oscuro cyberpunk

**Spec**: `019-rediseno-visual-mobile` | **Caso de uso**: US19 | **Prioridad**: P1

**Creado**: 2026-11-07 | **Estado**: Implementación parcial (58/87 tareas); MVP móvil y guard de
contraste en verde | **Módulo**: 5 — Experiencia móvil

**Plan**: [plan.md](./plan.md) · **Tareas**: [tasks.md](./tasks.md) · **Diseño**:
[research.md](./research.md), [data-model.md](./data-model.md),
[contrato de tokens](./contracts/tokens-movil.md), [contrato de interfaz](./contracts/ui-movil.md) y
[quickstart.md](./quickstart.md).

**Origen**: pedido del usuario de adaptar y optimizar la experiencia visual de Stockmante en
dispositivos móviles con una interfaz de alta densidad bajo estética cyberpunk dark, para que
operarios y administradores consulten kardex, registren entradas/salidas y supervisen alertas
críticas desde pantallas táctiles pequeñas sin perder claridad operativa ni rendimiento visual.

**Referencia visual**: no existen muestras móviles del producto. La referencia es la descripción
cromática y de efectos de esta spec, aplicada sobre la identidad ya implementada en 018
(`captures/samples/dark-2.png` como base de superficies y jerarquía). Mientras no se acuerde un
mock móvil, las reglas de esta spec son la referencia aceptada.

**Pantallas alcanzadas**: las 21 pantallas actuales — 18 rutas privadas (`/`, `/caja`,
`/movimientos`, `/transferencias`, `/conteos`, `/asistente`, `/admin/usuarios`,
`/admin/categorias`, `/admin/productos`, `/admin/depositos`, `/admin/stock`, `/admin/proveedores`,
`/admin/compras`, `/admin/ventas`, `/admin/clientes`, `/admin/reportes`, `/admin/importacion`,
`/admin/auditoria`) y 3 de acceso (`/login`, `/registro`, `/verificar-correo`) — más el modo de
cambio obligatorio de contraseña que vive en `/login`.

**Depende de**: 018 (tokens visuales, marco, primitivas de interfaz y tema claro/oscuro), 001
(acceso y sesión), 003–017 (páginas y endpoints existentes).

## Clarifications

### Session 2026-11-07

Decisiones confirmadas por el usuario en esta sesión (opción recomendada en los tres casos).

- Q: ¿El modo oscuro cyberpunk es la única apariencia móvil o convive con Claro/Oscuro/Sistema de
  018? → A: Es obligatorio en el layout móvil. Teléfonos y tablets (hasta 1023 px) usan siempre la
  identidad oscura cyberpunk y el selector de tema se oculta en el marco móvil; el escritorio
  (1024 px o más) conserva Claro/Oscuro/Sistema de 018 sin cambios. Se descarta una tercera paleta
  y se mantiene la preferencia guardada para cuando el usuario vuelva al escritorio.
- Q: ¿Barra inferior con "Más" o solo menú lateral retráctil? → A: Navegación híbrida. La barra
  inferior muestra 4–5 destinos principales según el rol y el botón "Más" abre el panel lateral con
  la navegación completa agrupada; ningún destino queda a más de dos toques.
- Q: ¿Las 21 pantallas o solo las operativas? → A: Cobertura completa. Las 18 rutas privadas, las 3
  de acceso y el modo de cambio obligatorio de contraseña reciben la adaptación móvil, con sus
  estados y su evidencia visual.

### Sesión 2026-11-07 (2) — cierre del riesgo de accesibilidad del oscuro obligatorio

Al planificar se detectó que, al ocultar el selector de tema y forzar el oscuro, el usuario pierde
la vía de escape a un tema claro. En lugar de reintroducir el tema claro en móvil, el riesgo se
cierra por diseño con cuatro medidas:

- Q: ¿Cómo se garantiza la legibilidad si no hay tema claro de escape? → A: La paleta móvil se
  verifica contra **WCAG AAA (7:1)** para texto de cuerpo, datos y badges, 4,5:1 para texto grande y
  3:1 para bordes de control y foco (FR-103). Los pares aprobados viven en un contrato y un test
  automático falla si un par no alcanza su mínimo o si se usa un tono crudo como texto.
- Q: ¿Y un usuario que necesita más contraste o su propio esquema? → A: El layout móvil respeta
  `prefers-contrast: more` (sube contraste sin salir del oscuro) y `forced-colors: active` (cede el
  color al sistema y quita resplandores), FR-133. No se agrega un control de tema ni de contraste
  propio: se usan las preferencias del sistema, que es el mecanismo estándar y no se puede olvidar.
- Q: ¿El estado del stock puede comunicarse solo por color? → A: No. Todo badge lleva etiqueta
  textual (Normal, Bajo, Crítico) y todo aviso lleva texto e icono (FR-118, FR-125). El color
  refuerza, no informa por sí solo.

## Escenarios de Usuario y Pruebas *(obligatorio)*

### User Story 19.1 - Marco móvil cyberpunk con navegación de una mano (Prioridad: P1)

Como **operario o administrador con teléfono**, quiero que la aplicación se abra en un marco
compacto de estética cyberpunk oscura, con una cabecera que me ubique y una navegación al alcance
del pulgar, para recorrer el sistema con una sola mano entre pasillos y sin zoom.

**Por qué esta prioridad**: es la base de todas las demás historias. Sin marco, navegación e
identidad móvil no se puede adaptar ninguna pantalla y el teléfono seguiría mostrando un escritorio
encogido.

**Prueba independiente**: abrir la aplicación en un teléfono de 390×844, recorrer todos los
destinos permitidos para cada rol desde la barra inferior y el panel lateral, y verificar que el
marco se mantiene, la ruta activa se resalta, el fondo no se desplaza horizontalmente y la
identidad oscura cyberpunk se aplica en todas las pantallas.

**Escenarios de aceptación**:

1. **Dado** un teléfono en vertical (menos de 768 px de ancho), **Cuando** un usuario autenticado
   abre cualquier ruta privada, **Entonces** ve una cabecera compacta con el título de la página,
   la búsqueda rápida y el menú de usuario, y una barra de navegación inferior fija con los destinos
   principales de su rol y el botón "Más" (FR-108/FR-109).
2. **Dado** que el destino activo no está en la barra inferior (por ejemplo, Auditoría),
   **Cuando** se abre esa ruta, **Entonces** el botón "Más" aparece marcado como activo y el panel
   lateral muestra el ítem con `aria-current`.
3. **Dado** un usuario `OPERACION`, **Cuando** mira la barra inferior y el panel lateral,
   **Entonces** no aparecen Conteos ni destinos de Administración, y ninguna ruta restringida queda
   accesible por navegación (el backend sigue siendo la autoridad).
4. **Dado** el panel lateral abierto, **Cuando** el usuario toca el fondo, presiona Escape o elige
   un destino, **Entonces** el panel se cierra, el foco vuelve al botón que lo abrió y el fondo no
   quedó desplazado.
5. **Dado** el panel lateral abierto, **Cuando** un usuario navega con teclado, **Entonces** el foco
   queda atrapado dentro del panel y no alcanza el contenido de atrás.
6. **Dado** un teléfono con muesca o barra de gestos, **Cuando** se abre cualquier pantalla,
   **Entonces** la cabecera y la barra inferior respetan el área segura y ningún control queda
   tapado por el sistema.
7. **Dado** un usuario que cambia de vertical a horizontal, **Cuando** gira el teléfono,
   **Entonces** la navegación se reacomoda sin perder el destino activo ni el estado de la página.

**Checkpoint**: el marco móvil, la navegación y la identidad cyberpunk funcionan en todas las
rutas; el contenido interno de cada página puede seguir pendiente.

---

### User Story 19.2 - Tablero móvil con las métricas que importan (Prioridad: P2)

Como **operario o gerente**, quiero que Inicio muestre en el teléfono las métricas clave —stock
total, productos activos y alertas críticas— en tarjetas deslizables de una columna, para saber
cómo está el inventario antes de moverme.

**Por qué esta prioridad**: es la pantalla de entrada y donde primero se nota si la adaptación móvil
funciona con datos reales; además demuestra el lenguaje visual cyberpunk con información de negocio.

**Prueba independiente**: con datos sembrados, comparar cada número del carrusel contra la API; con
la base vacía, verificar ceros y estados vacíos; con un usuario `OPERACION`, confirmar que no se
muestran métricas ni secciones restringidas.

**Escenarios de aceptación**:

1. **Dado** un usuario `GESTION`, **Cuando** abre Inicio en el teléfono, **Entonces** ve las
   métricas clave en un carrusel horizontal con ajuste de una tarjeta a la vez (o una cuadrícula de
   una columna que se puede colapsar), cada una con etiqueta, valor y contexto temporal.
2. **Dado** que hay alertas de stock crítico, **Cuando** se abre Inicio, **Entonces** la tarjeta de
   alertas aparece en la primera posición y su acento es naranja/rojo neón, sin depender solo del
   color para comunicar el problema.
3. **Dado** que no hay alertas, **Cuando** se abre Inicio, **Entonces** la tarjeta lo dice con un
   mensaje positivo y una acción para revisar el catálogo, no con un espacio vacío.
4. **Dado** un usuario `OPERACION`, **Cuando** abre Inicio, **Entonces** solo ve las métricas
   permitidas por su rol y el sistema no solicita datos de gestión.
5. **Dado** un endpoint que falla, **Cuando** se abre Inicio, **Entonces** solo esa tarjeta muestra
   el error con acción de reintento; el resto del tablero sigue visible y usable.
6. **Dado** el carrusel, **Cuando** se navega con teclado o lector de pantalla, **Entonces** las
   tarjetas se anuncian en orden, los botones de anterior/siguiente tienen nombre accesible y el
   desplazamiento no atrapa el foco.
7. **Dado** una base sin movimientos, **Cuando** se abre Inicio, **Entonces** el resumen de
   actividad y los gráficos muestran un estado vacío explicativo, no un área en blanco ni un error.

**Checkpoint**: Inicio es un tablero móvil real y autónomo; las listas de inventario aún muestran
tablas.

---

### User Story 19.3 - Inventario en tarjetas táctiles con badges de stock (Prioridad: P3)

Como **operario en movimiento**, quiero que las tablas densas se conviertan en tarjetas con SKU,
nombre, stock actual y acciones rápidas, para identificar un producto y su nivel de stock sin zoom ni
desplazamiento lateral.

**Por qué esta prioridad**: es el flujo de consulta que el operario hace decenas de veces por turno
y el que más se degrada en una pantalla pequeña; depende del marco y de la identidad, pero no de la
administración completa.

**Prueba independiente**: en Stock, Productos, Caja, Movimientos y Kardex, buscar y filtrar en el
teléfono, verificar que cada fila aparece como tarjeta con SKU, nombre, badge de stock y acciones, y
que las acciones siguen produciendo los mismos resultados que en escritorio.

**Escenarios de aceptación**:

1. **Dado** un teléfono, **Cuando** se abre `/admin/stock` o el resultado de `/caja`,
   **Entonces** las filas se ven como tarjetas verticales con SKU en monoespaciada, nombre
   truncado con puntos suspensivos, stock actual con badge de color y acciones rápidas.
2. **Dado** un producto con stock sobre el mínimo, **Cuando** se muestra su tarjeta, **Entonces** el
   badge es verde y dice "Normal"; con stock bajo es naranja y dice "Bajo"; con stock en cero o bajo
   el mínimo crítico es rojo y dice "Crítico".
3. **Dado** un nombre de producto muy largo, **Cuando** se muestra la tarjeta, **Entonces** el nombre
   se trunca a dos líneas con elipsis y el nombre completo sigue disponible como texto accesible o
   al abrir el detalle, sin empujar las acciones fuera de vista.
4. **Dado** una pantalla de menos de 360 px, **Cuando** se muestran tarjetas, **Entonces** los
   títulos largos se truncan con elipsis y no aparece desplazamiento horizontal del cuerpo.
5. **Dado** el teléfono en horizontal o una tablet, **Cuando** se abre una lista, **Entonces** el
   layout se reorganiza en una tabla compacta de dos columnas o cuadrícula de dos columnas, sin
   perder acciones ni badges.
6. **Dado** una lista con 50 o más tarjetas, **Cuando** se desplaza, **Entonces** la barra inferior
   no tapa la última tarjeta, la paginación o "cargar más" conserva el total y no se pierde la
   posición de lectura al refrescar.
7. **Dado** una lista vacía, **Cuando** no hay resultados, **Entonces** se muestra un estado vacío
   con la siguiente acción sugerida (limpiar filtros, crear producto o ampliar la búsqueda).

**Checkpoint**: consulta y lectura de inventario resueltas en tarjetas; queda registrar movimientos.

---

### User Story 19.4 - Registro rápido de movimiento con modal y destello neón (Prioridad: P4)

Como **operario o cajero**, quiero tocar una tarjeta de producto, abrir una hoja a pantalla completa
con teclado numérico, confirmar y recibir un aviso neón con el resultado, para registrar una entrada
o salida en segundos sin salir de la pantalla.

**Por qué esta prioridad**: convierte la consulta en acción; requiere las tarjetas y el marco, y su
valor operativo es alto, pero puede entregarse después de la lectura.

**Prueba independiente**: desde una tarjeta de Stock o el botón flotante de Movimientos, registrar
una entrada y una salida válida e inválida, comprobar el bloqueo del fondo, el teclado numérico, la
validación de máximos y el aviso con el stock resultante.

**Escenarios de aceptación**:

1. **Dado** un usuario con permiso de operar, **Cuando** toca la acción rápida de una tarjeta o el
   botón flotante, **Entonces** se despliega una hoja a pantalla completa con los campos táctiles
   grandes y el campo de cantidad abre teclado numérico.
2. **Dado** la hoja abierta, **Cuando** el usuario intenta desplazar el fondo, **Entonces** el fondo
   no se mueve; al cerrar la hoja el scroll vuelve a su posición anterior.
3. **Dado** una cantidad mayor al stock disponible o un valor vacío, cero, negativo o no numérico,
   **Cuando** se confirma, **Entonces** el envío se bloquea, el mensaje aparece junto al campo y el
   foco vuelve a ese campo.
4. **Dado** una operación válida, **Cuando** se confirma, **Entonces** el botón se deshabilita
   mientras está en curso, se muestra un aviso flotante con destello neón una sola vez y el mensaje
   informa tipo de movimiento, producto, depósito, cantidad y stock resultante.
5. **Dado** el aviso de confirmación, **Cuando** el usuario lo ignora, **Entonces** se anuncia como
   estado del sistema, se puede cerrar y no tapa la acción siguiente ni la barra inferior.
6. **Dado** un error del servidor (por ejemplo, stock insuficiente en el depósito), **Cuando** se
   confirma, **Entonces** la hoja permanece abierta, se muestra el mensaje concreto de la API y los
   datos ya cargados no se pierden.
7. **Dado** una operación en curso, **Cuando** el usuario toca repetidamente el botón de confirmar,
   **Entonces** se registra un solo movimiento.

**Checkpoint**: registro rápido completo; resta la coherencia de acceso y administración.

---

### User Story 19.5 - Acceso y administración coherentes en móvil (Prioridad: P5)

Como **visitante o gerente**, quiero que el login, el registro, la verificación y las pantallas de
administración compartan la identidad móvil cyberpunk, para no sentir que salgo del producto al
entrar o al gestionar el catálogo.

**Por qué esta prioridad**: completa la cobertura y demuestra que el lenguaje escala; son pantallas
de menor frecuencia y dependen de todas las primitivas anteriores.

**Prueba independiente**: ejecutar los recorridos de acceso en un teléfono y, en cada pantalla de
administración, listar, filtrar, paginar y ejecutar una acción de tarjeta comprobando que los
resultados y los mensajes son los de antes.

**Escenarios de aceptación**:

1. **Dado** un teléfono de 320 px de ancho, **Cuando** un visitante abre `/login`, **Entonces** ve
   una tarjeta a pantalla completa con la marca, los campos, la acción principal y los enlaces, sin
   desborde horizontal y con el teclado sin tapar el campo activo.
2. **Dado** un usuario con contraseña temporal, **Cuando** inicia sesión en el teléfono,
   **Entonces** ve el modo de cambio obligatorio con la misma estética y la política de contraseña
   vigente.
3. **Dado** un login con credenciales inválidas, **Cuando** se envía, **Entonces** se muestra el
   mensaje real de la API sin revelar datos y sin perder lo escrito.
4. **Dado** un usuario `GESTION` en cualquier pantalla de administración, **Cuando** abre la página,
   **Entonces** ve el encabezado con título y acción principal, los filtros en un panel colapsable y
   la lista de tarjetas con badges y acciones.
5. **Dado** una acción de tarjeta destructiva (baja, anulación), **Cuando** se toca, **Entonces**
   pide confirmación y, al completarse, muestra el mensaje concreto de la API en la misma pantalla.
6. **Dado** un usuario `OPERACION`, **Cuando** intenta abrir una ruta de administración,
   **Entonces** es redirigido o ve el rechazo, sin que la interfaz muestre acciones que no puede
   completar.
7. **Dado** cualquier pantalla de administración con muchas tarjetas, **Cuando** se filtra y pagina,
   **Entonces** el contador y los filtros activos son visibles y se pueden limpiar de una vez.

**Checkpoint**: las 21 pantallas y el cambio obligatorio de contraseña están cubiertos; resta la
evidencia y la verificación final.

---

### Edge Cases

- **Pantallas muy pequeñas (menos de 360 px)**: títulos y nombres largos se truncan con elipsis
  (`text-overflow: ellipsis`); no se rompe el layout ni aparece desplazamiento horizontal.
- **Rotación de pantalla**: en horizontal de teléfono o tablet la lista de tarjetas pasa a una tabla
  compacta de dos columnas o cuadrícula de dos columnas, conservando acciones y badges.
- **Contraste en modo oscuro cyberpunk**: prohibido el texto gris oscuro sobre fondo negro; toda
  tipografía secundaria usa tonos claros equivalentes a `slate-400` o superiores.
- **Teclado en pantalla abierto**: el campo activo y el botón de confirmar quedan visibles; el
  layout no queda tapado ni obliga a desplazarse a ciegas.
- **Área segura y muescas**: cabecera, barra inferior y hojas respetan los márgenes del sistema.
- **Ocho o más destinos**: la navegación no desborda; los excedentes viven en el panel lateral.
- **Datos vacíos**: ceros y listas vacías muestran estado explicativo con acción siguiente, no un
  error ni un espacio en blanco.
- **API caída o lenta**: cada tarjeta o widget muestra su propio error con reintento; el marco y el
  resto de la pantalla siguen utilizables.
- **Permisos**: un `OPERACION` no ve acciones ni destinos de gestión; si una llamada devuelve 403 se
  oculta la acción y no se muestra un error crudo.
- **Red lenta al registrar**: el botón se deshabilita en curso y nunca se duplica el movimiento.
- **Hoja abierta y botón atrás del teléfono**: cierra la hoja antes de abandonar la página; si hay
  datos sin guardar pide confirmación.
- **Movimiento reducido**: sin animaciones de resplandor ni transiciones no esenciales cuando
  `prefers-reduced-motion` está activo.
- **Modo claro del sistema**: el layout móvil ignora la preferencia clara y usa siempre la identidad
  oscura cyberpunk (FR-104); el escritorio conserva Claro/Oscuro/Sistema. En ningún caso debe quedar
  una pantalla ilegible.
- **Tablet en horizontal**: densidad intermedia, sin llegar a la barra lateral de escritorio.
- **Impresión**: fuera de alcance; no se promete hoja de impresión móvil.

## Requisitos Funcionales

### Identidad visual, tema y ergonomía

- **FR-102 Identidad cyberpunk móvil.** La apariencia cyberpunk del modo oscuro móvil DEBE usar
  fondos negros y gris carbón de alta pureza, superficies elevadas un paso más claras, bordes
  finos con resplandor sutil en componentes interactivos y acciones principales. Los acentos DEBEN
  ser funcionales y estables en toda la aplicación: cian/turquesa para estados activos y navegación
  seleccionada, violeta/fucsia para métricas secundarias, y naranja/rojo neón solo para alertas de
  stock crítico. El resplandor NO DEBE degradar el contraste del texto ni ser la única señal de un
  estado.
- **FR-103 Contraste y legibilidad en oscuro.** Al no existir un tema claro de escape en móvil, el
  texto de cuerpo, las etiquetas de datos y los badges DEBEN alcanzar **WCAG AAA (7:1)** sobre su
  superficie real; el texto grande DEBE alcanzar 4,5:1, y los bordes de controles interactivos, el
  foco y los indicadores no textuales DEBEN alcanzar 3:1. Se PROHÍBE usar los tonos de estado crudos
  (rojo y violeta de acento) como color de texto: el texto de estado usa sus variantes claras
  verificadas.
  Se PROHÍBE el texto gris oscuro sobre fondo negro y toda tipografía secundaria DEBE usar tonos
  claros equivalentes a `slate-400` o superiores. Los colores de estado DEBEN conservar su
  significado (verde normal, naranja bajo, rojo crítico), ser legibles bajo luz artificial de
  almacén y no ser nunca la única señal.
- **FR-104 Modo oscuro cyberpunk obligatorio en móvil.** El layout móvil (hasta 1023 px) DEBE usar
  siempre la identidad oscura cyberpunk, con independencia de la preferencia del sistema o de una
  preferencia clara guardada; el selector de tema de 018 NO DEBE mostrarse en el marco móvil y la
  preferencia guardada DEBE conservarse para cuando el usuario vuelva al escritorio (1024 px o
  más). La identidad oscura DEBE aplicarse antes del primer pintado y no DEBE producir destellos ni
  saltos de layout.
- **FR-105 Efectos y movimiento reducido.** El resplandor, las transiciones y el destello de
  confirmación DEBEN ser sutiles, de una sola pulsación y respetar `prefers-reduced-motion`; la
  interfaz NO DEBE mantener animaciones continuas en reposo.
- **FR-106 Ergonomía táctil y área segura.** Todo control interactivo DEBE ofrecer un área de toque
  mínima de 44×44 px; los campos numéricos DEBEN abrir teclado numérico y usar tipografía de al
  menos 16 px para evitar el zoom automático del navegador; el contenido DEBE respetar los márgenes
  de seguridad de pantallas con muesca o barra de gestos.

### Marco y navegación

- **FR-107 Marco móvil compacto.** En teléfonos, toda pantalla autenticada DEBE renderizarse en un
  marco con cabecera compacta (título, búsqueda rápida y menú de usuario), contenido a ancho
  completo y sin el pie de página de escritorio; la cabecera DEBE permanecer visible al desplazarse.
- **FR-108 Navegación móvil híbrida de una mano.** La navegación DEBE combinar una barra inferior
  con 4–5 destinos principales visibles y un botón "Más" que abra el panel lateral con el resto de
  los destinos permitidos por el rol; ningún destino DEBE quedar a más de dos toques y la
  navegación NO DEBE exigir volver al inicio de la página.
- **FR-109 Barra de navegación inferior.** La barra DEBE permanecer fija sobre el contenido, mostrar
  icono y etiqueta por destino, marcar el ítem activo con `aria-current` y mostrar el contador de
  alertas de stock cuando exista. La selección de destinos principales DEBE priorizar Inicio, Caja y
  Movimientos y completarse con hasta dos destinos según el rol; el botón "Más" DEBE estar siempre
  presente y marcarse como activo cuando la ruta vigente no esté en la barra.
- **FR-110 Panel lateral retráctil.** El panel lateral DEBE contener la navegación completa agrupada
  (Operación y Administración), respetar el rol, cerrarse con el fondo, Escape o la elección de un
  destino, devolver el foco al botón de origen y bloquear el desplazamiento del fondo mientras está
  abierto.
- **FR-111 Búsqueda rápida en la cabecera.** La cabecera DEBE ofrecer una búsqueda que encuentre un
  producto por SKU o código de barras y por nombre sin abandonar la pantalla actual, mostrando los
  resultados como tarjetas verticales; DEBE reutilizar la búsqueda ya existente y no crear
  endpoints nuevos.
- **FR-112 Adaptación por orientación.** En vertical las listas DEBEN ser de una columna; en
  horizontal de teléfono o tablet DEBEN reorganizarse en tabla compacta de dos columnas o cuadrícula
  de dos columnas, sin perder acciones, badges ni paginación.
- **FR-113 Continuidad con escritorio y tablet.** Las pantallas de escritorio (1024 px o más)
  conservan el marco de 018 sin cambios visuales; las tablets (768–1023 px) adoptan la navegación
  móvil con densidad intermedia y mantienen el acceso a todas las funciones de su rol.

### Tablero (Command Center móvil)

- **FR-114 Métricas clave en formato deslizable.** Inicio DEBE mostrar Total de stock (o stock
  valorizado), Productos activos y Alertas críticas —más los indicadores ya existentes que el rol
  permita— en un carrusel horizontal con ajuste de una tarjeta a la vez o en una cuadrícula de una
  columna colapsable; cada tarjeta DEBE mostrar etiqueta, valor y contexto.
- **FR-115 Prioridad y orden del tablero.** Si hay alertas, su tarjeta DEBE ir primero y usar el
  acento de alerta; luego se muestran las métricas restantes, la lista de casos urgentes, la
  actividad reciente y los accesos rápidos, en ese orden de lectura.
- **FR-116 Estados por widget.** Cada widget del tablero DEBE resolver su propia carga, vacío y
  error con reintento, sin bloquear ni romper los demás; los gráficos DEBEN simplificarse o
  desplazarse dentro de su tarjeta conservando una leyenda legible.

### Listas, tablas y tarjetas

- **FR-117 Tabla a tarjetas en móvil.** En teléfonos, toda tabla de datos DEBE renderizarse como
  lista de tarjetas con la información esencial, sin exigir zoom ni desplazamiento horizontal para
  leerla. Las tablas de escritorio se conservan en 1024 px o más.
- **FR-118 Contenido de la tarjeta de inventario.** Cada tarjeta DEBE mostrar SKU en monoespaciada,
  nombre en hasta dos líneas con elipsis, stock actual con badge de color y etiqueta textual
  (Normal, Bajo, Crítico), el contexto relevante (depósito, ubicación o fecha) y las acciones; el
  estado NO DEBE comunicarse solo por color.
- **FR-119 Acciones rápidas por tarjeta.** Cada tarjeta DEBE exponer su acción principal como botón
  de al menos 44×44 px y las secundarias en un menú desbordable; las acciones destructivas DEBEN
  pedir confirmación y comunicar el resultado concreto.
- **FR-120 Filtros y paginación móviles.** Los filtros DEBEN vivir en un panel colapsable o en chips,
  mostrar los filtros activos y permitir limpiarlos de una vez; la paginación DEBE ser compacta o
  por "cargar más", conservar el total y no perder la posición de lectura.
- **FR-121 Estados de lista.** Toda lista DEBE representar carga con esqueletos del mismo alto (sin
  salto de layout), vacío con acción siguiente, error con reintento y refresco que conserva los
  datos visibles.

### Registro rápido de movimiento

- **FR-122 Acción flotante.** Las pantallas operativas DEBEN ofrecer un botón flotante de acción
  rápida (56×56 px) con la acción principal según pantalla y rol (Nuevo movimiento, Ajuste de stock,
  Nueva venta), nombre accesible y sin tapar la última tarjeta ni la barra inferior.
- **FR-123 Hoja de pantalla completa.** Al activar una acción rápida, DEBE abrirse una hoja a
  pantalla completa con campos táctiles grandes, teclado numérico automático y bloqueo del
  desplazamiento del fondo; se cierra con gesto, botón atrás o Escape y pide confirmación si hay
  datos sin guardar.
- **FR-124 Validación de cantidades.** DEBE impedirse el envío con cantidad vacía, cero, negativa,
  no numérica o mayor al stock disponible; el mensaje DEBE ser claro y concreto junto al campo, con
  el foco en el campo inválido, y los errores del servidor DEBEN mostrarse dentro de la hoja sin
  perder lo cargado.
- **FR-125 Aviso de confirmación neón.** La confirmación DEBE mostrarse como aviso flotante con
  destello neón de una sola pulsación que informa el resultado concreto (tipo, producto, depósito,
  cantidad y stock resultante), se anuncia como estado del sistema, se puede cerrar y no tapa la
  acción siguiente; el resultado DEBE quedar además visible en la tarjeta o el historial.
- **FR-126 Sin doble envío.** Mientras una operación está en curso, la confirmación DEBE
  deshabilitarse y no DEBE registrarse más de un movimiento por toque repetido.

### Alcance, calidad y no regresión

- **FR-127 Cobertura de pantallas móviles.** Las 21 pantallas actuales —18 rutas privadas y 3 de
  acceso— y el modo de cambio obligatorio de contraseña DEBEN recibir la adaptación móvil completa
  (marco, identidad, tarjetas y estados); toda pantalla DEBE conservar sus campos, validaciones,
  mensajes, permisos y navegación actuales.
- **FR-128 Sin cambios de comportamiento.** La adaptación móvil NO DEBE modificar endpoints,
  contratos, esquema, reglas de negocio, validaciones ni permisos; es una capa de presentación y de
  estado local de interfaz, y las suites existentes DEBEN seguir en verde.
- **FR-129 Accesibilidad móvil.** La interfaz DEBE tener etiquetas asociadas a campos, foco visible,
  navegación por teclado, nombres accesibles en botones icónicos, `aria-current` en la ruta activa,
  landmarks y respeto por `prefers-reduced-motion`.
- **FR-130 Rendimiento percibido.** La lista filtrada DEBE pintarse en menos de 300 ms, con
  desplazamiento fluido, sin saltos de layout y sin cargas que bloqueen la interacción.
- **FR-131 Evidencia móvil.** Cada una de las 21 pantallas y el modo de cambio obligatorio de
  contraseña DEBEN entregarse con capturas en 390×844 (vertical), 844×390 (horizontal) y 320×568
  (compacta), más una auditoría de áreas de toque y un reporte de contraste de los pares de color
  usados.
- **FR-132 Pruebas de lógica pura.** La clasificación de dispositivo y orientación, los umbrales de
  badge de stock, el estado de carrusel y filtros y el formato del aviso DEBEN tener pruebas
  automatizadas; la presentación se verifica con capturas y no con pruebas de estilo.
- **FR-133 Preferencias de accesibilidad del sistema.** Como el usuario no puede cambiar a un tema
  claro en móvil, el layout móvil DEBE respetar las preferencias de accesibilidad del sistema:
  `prefers-contrast: more` DEBE subir el contraste de textos, bordes y estados sin abandonar la
  identidad oscura, y `forced-colors: active` DEBE desactivar resplandores y sombras y ceder el
  color al sistema operativo. Ningún par de color aprobado puede quedar por debajo de su mínimo en
  estos modos, y ninguna señal de estado puede depender solo del color.

## Entidades Clave *(modelo de diseño, sin datos nuevos)*

- **Pantalla móvil**: ruta existente, categoría (acceso, operación, administración o tablero),
  patrón de presentación, rol mínimo, prioridad de entrega y evidencia visual.
- **Destino de navegación móvil**: etiqueta, ruta, icono, ubicación (barra inferior o panel lateral),
  visibilidad por rol y contador opcional.
- **Tarjeta de inventario**: SKU, nombre, stock actual, umbral, badge de estado, contexto y acciones.
- **Métrica del tablero**: etiqueta, valor, contexto, acento cromático, origen del dato y rol mínimo.
- **Badge de stock**: umbral, color, etiqueta textual y significado operativo.
- **Hoja de acción**: título, campos, validaciones, acciones y reglas de cierre.
- **Aviso flotante**: tipo, mensaje, resultado concreto, duración y descarte.
- **Token cyberpunk**: nombre semántico, valor, uso permitido y par de contraste verificado.

El detalle de estas entidades vive en `data-model.md` del plan.

## Criterios de Éxito Medibles

- **SC-039** El 100 % de los controles interactivos de las pantallas móviles mide al menos 44×44 px
  en la auditoría de áreas de toque, sin excepciones.
- **SC-040** Cero desplazamiento horizontal del cuerpo en las 21 pantallas entre 320 px y 430 px en
  vertical y entre 640 px y 1023 px en horizontal.
- **SC-041** Con 10 000 productos de referencia, el percentil 95 del tiempo entre enviar un filtro y
  ver la lista pintada es menor a 300 ms.
- **SC-042** El 100 % de los pares texto/superficie, badge/superficie e indicador/superficie de la
  paleta móvil alcanza AAA (7:1 texto de cuerpo y datos; 4,5:1 texto grande; 3:1 bordes de control,
  foco e indicadores no textuales), verificado por un test automático sobre los tokens; no queda
  ningún texto gris oscuro sobre fondo negro ni tonos crudos de estado usados como texto.
- **SC-043** El 100 % de las tablas densas incluidas en el alcance se presenta como tarjetas en
  teléfono con SKU, nombre, badge de stock y acciones, y como tabla compacta de dos columnas en
  horizontal.
- **SC-044** Cero regresiones funcionales: los recorridos de acceso, operación y administración
  producen los mismos resultados y mensajes que antes, y la verificación automática del proyecto
  queda en verde.
- **SC-045** Cada una de las 21 pantallas y el modo de cambio obligatorio de contraseña se entrega
  con sus capturas en 390×844, 844×390 y 320×568, su auditoría de toque y su reporte de contraste.
- **SC-046** Un operario entrenado registra un movimiento desde la lista en 4 toques o menos y en
  menos de 30 segundos, con el stock resultante visible al terminar.
- **SC-047** Con una hoja de acción abierta, el fondo no se desplaza en 10 intentos de gesto, y en
  las pruebas de validación no se registra ningún envío por encima del stock disponible.
- **SC-048** Al abrir 5 pantallas distintas en un teléfono, 10 recargas no muestran destellos claros
  (la identidad oscura se aplica antes del primer pintado) y el área segura se respeta en un
  dispositivo con muesca.
- **SC-049** Con `prefers-contrast: more` activo, el 100 % de los pares aprobados mantiene o mejora
  su ratio y ningún texto baja de 7:1; con `forced-colors: active` no queda ningún resplandor ni
  sombra decorativa y los estados conservan texto e icono.

## Supuestos

- No existe un mock móvil del producto; la descripción de color, efectos y ergonomía de esta spec
  es la referencia aceptada hasta que el usuario aporte uno.
- La identidad cyberpunk oscura es obligatoria en el layout móvil (hasta 1023 px) y no reemplaza el
  rediseño de escritorio de 018, que se conserva sin cambios visuales ni pierde su selector
  Claro/Oscuro/Sistema.
- El selector de tema no aparece en el marco móvil; la preferencia guardada por el usuario sigue
  vigente y se aplica cuando vuelve al escritorio.
- Se reutilizan los tokens, componentes y páginas de 018, ampliados con variantes móviles; no se
  incorporan librerías de interfaz, iconos, gráficos ni gestos.
- La búsqueda rápida reutiliza la búsqueda de productos ya existente (003/004); no se agregan
  endpoints ni cambios de esquema.
- Puntos de corte propuestos por esta spec: teléfono por debajo de 768 px, tablet de 768 a 1023 px
  y escritorio desde 1024 px. El punto de 1024 px coincide con el marco de 018.
- Los volúmenes de prueba son páginas de hasta 100 filas y catálogos de hasta 10 000 productos, con
  paginación; no se promete lista infinita.
- La evidencia se obtiene con emulación de dispositivos y no reemplaza una prueba en hardware real,
  que se recomienda antes de cerrar la spec.
- Los textos siguen en español (es-AR), sin internacionalización nueva.
- La preferencia de tema, la sesión y las reglas de negocio siguen siendo las de 001 y 018; en el
  layout móvil la preferencia claro/oscuro no altera la identidad oscura (FR-104) y el modo móvil
  no crea almacenamiento nuevo más allá del estado local de interfaz.

## Fuera de Alcance

- Aplicación móvil nativa, PWA instalable, funcionamiento sin conexión o sincronización en segundo
  plano.
- Notificaciones push, vibración o háptica.
- Lectura de códigos de barras con la cámara: la búsqueda es por texto ingresado o selección.
- Rediseño de escritorio (018) o cambios en su marco en 1024 px o más.
- Nuevos endpoints, migraciones, permisos o reglas de negocio.
- Diseño específico para tablet más allá de la adaptación de dos columnas.
- Internacionalización, RTL o temas por usuario persistidos en el servidor.
- Impresión o exportación a PDF desde el teléfono.
- Pantallas que no existan al momento de implementar la spec.

## Dependencias y orden de implementación

1. Identidad, tokens móviles, marco y navegación (FR-102…FR-113).
2. Tablero móvil (FR-114…FR-116).
3. Listas y tarjetas de inventario (FR-117…FR-121).
4. Hoja de acción y aviso neón (FR-122…FR-126).
5. Cobertura de acceso y administración, accesibilidad, evidencia y no regresión (FR-127…FR-133;
   SC-039…SC-049).

La spec no introduce migraciones ni dependencias de backend; se apoya en los contratos existentes de
003–017 y en la capa visual de 018.
