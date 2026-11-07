# Especificación de Funcionalidad: Rediseño visual del escritorio

**Spec**: `018-rediseno-visual-desktop` | **Caso de uso**: US18 | **Prioridad**: P1

**Creado**: 2026-11-07 | **Estado**: Especificado; pendiente de plan y tasks | **Módulo**: 5 —
Experiencia de escritorio

**Origen**: pedido del usuario de rehacer la interfaz de todas las páginas de la aplicación tomando
como modelo visual las muestras `captures/samples/light-2.png` y `captures/samples/dark-2.png`
(panel de administración StarAdmin, variantes clara y oscura), y de registrar el trabajo en esta
spec 018.

**Referencia visual**: `captures/samples/light-2.png` y `captures/samples/dark-2.png`.

**Inventario a rediseñar**: `captures/01-login.png` … `captures/17-admin-auditoria.png`
(17 capturas; el cambio obligatorio de contraseña es un modo de `/login` y no tiene captura
propia).

**Depende de**: 001 (autenticación, sesión y cambio de contraseña), 003–016 (páginas y endpoints ya
existentes) y 017 (pantalla del asistente).

## Clarifications

### Session 2026-11-07

Decisiones confirmadas por el usuario en esta sesión (opción recomendada en los cuatro casos).

- Q: ¿El rediseño puede agregar o quitar funcionalidad, o debe ser estrictamente visual? → A:
  Rediseño estructural de la interfaz **sin** cambios de comportamiento, contratos de API, esquema
  ni reglas de negocio. El tablero de Inicio es la única superficie nueva y solo lee endpoints
  existentes.
- Q: ¿Qué identidad visual reemplaza a la referencia crema/naranja que quedó a medio implementar?
  → A: La de las muestras `light-2`/`dark-2`: marco de administración con barra lateral, tarjetas
  blancas sobre fondo gris azulado, acento azul, acentos teal y ámbar, y variante oscura azul
  noche. La paleta crema/naranja se retira.
- Q: ¿Cómo se resuelven los gráficos del tablero sin sumar dependencias? → A: Con SVG y CSS propios
  (barras, dona y línea), sin librería de gráficos ni de iconos; los iconos son SVG inline.
- Q: ¿Se rediseñan todas las páginas en una sola entrega o de a una? → A: Todas las capturadas, en
  incrementos por grupo (marco, acceso, operación, administración) y con evidencia visual por
  página en ambos temas antes de considerarla terminada.

## Escenarios de Usuario y Pruebas *(obligatorio)*

### User Story 18.1 - Marco de escritorio coherente y tema claro/oscuro (Prioridad: P1)

Como **usuario autenticado**, quiero que cualquier pantalla de la aplicación se abra dentro de un
mismo marco de escritorio —barra lateral con la navegación agrupada, barra superior con el contexto
de la página, mi usuario y el selector de tema— para orientarme rápido y trabajar siempre en el
mismo lugar.

**Por qué esta prioridad**: es la base de todas las demás historias. Sin marco, tokens y tema no
hay rediseño posible y las páginas seguirían pareciendo pantallas sueltas.

**Prueba independiente**: iniciar sesión, recorrer todas las rutas desde la barra lateral y
verificar que el marco se mantiene, que la ruta activa se resalta, que el tema cambia y persiste
tras recargar, y que no hay desborde horizontal entre 320 px y 1440 px.

**Escenarios de aceptación**:

1. **Dado** un usuario autenticado, **Cuando** entra a cualquier ruta privada, **Entonces** ve la
   barra lateral con los grupos "Operación" y "Administración", la barra superior con el título de
   la página, su nombre y rol, el selector de tema y el menú de sesión, y el contenido dentro del
   área principal.
2. **Dado** un usuario `OPERACION`, **Cuando** mira la barra lateral, **Entonces** no aparecen
   Productos, Categorías, Depósitos, Stock, Proveedores, Compras, Usuarios ni Auditoría, y ninguna
   ruta de administración queda accesible por navegación (el backend sigue siendo la autoridad).
3. **Dado** un usuario `GESTION`, **Cuando** hay productos bajo el mínimo, **Entonces** el ítem
   Stock muestra un contador visible y accesible con el total de alertas.
4. **Dado** un tema guardado `dark`, **Cuando** se recarga cualquier página, **Entonces** el primer
   pintado ya es oscuro, sin destello claro y sin saltos de layout.
5. **Dado** la preferencia `system`, **Cuando** el sistema operativo cambia entre claro y oscuro,
   **Entonces** la interfaz acompaña el cambio sin recargar.
6. **Dado** una ventana de 1024 px o menos, **Cuando** se abre la aplicación, **Entonces** la barra
   lateral se oculta tras un botón de menú y se abre como panel superpuesto, sin empujar el
   contenido ni generar scroll horizontal.
7. **Dado** la barra lateral colapsada, **Cuando** se pasa el foco o el puntero por un ítem,
   **Entonces** se ve su etiqueta y el ítem conserva nombre accesible.

**Checkpoint**: el marco, los tokens y el tema funcionan en todas las rutas; las páginas todavía
pueden conservar su contenido interno anterior.

---

### User Story 18.2 - Inicio como tablero operativo (Prioridad: P2)

Como **gerente u operario**, quiero que la pantalla de Inicio resuma el estado del inventario —stock
bajo mínimo, productos activos, movimientos recientes y evolución— para empezar el día con contexto
y detectar problemas sin navegar página por página.

**Por qué esta prioridad**: es la pantalla que la muestra usa como referencia y el punto de entrada
diario; además demuestra que el lenguaje visual funciona con datos reales, no solo con formularios.

**Prueba independiente**: con datos sembrados, comparar cada número y serie del tablero contra la
respuesta de la API; con la base vacía, verificar ceros y estados vacíos; con un usuario
`OPERACION`, verificar que los widgets reservados a gestión no se muestran ni generan errores.

**Escenarios de aceptación**:

1. **Dado** un usuario `GESTION`, **Cuando** abre Inicio, **Entonces** ve tarjetas de indicadores con
   stock bajo mínimo, productos activos, movimientos del día y valor del stock, cada una con su
   etiqueta, el número y una referencia temporal o de contexto.
2. **Dado** un usuario `OPERACION`, **Cuando** abre Inicio, **Entonces** ve solo los indicadores y
   widgets permitidos por su rol y no ve ni solicita datos de gestión.
3. **Dado** movimientos de distinto tipo, **Cuando** se abre Inicio, **Entonces** el gráfico de
   barras muestra el total exacto por tipo (entrada, salida, merma y ajuste), coherente con la API y
   con una leyenda legible en ambos temas.
4. **Dado** el catálogo con y sin alerta, **Cuando** se abre Inicio, **Entonces** la dona muestra la
   proporción de productos bajo mínimo frente al total y una lista con los casos más urgentes.
5. **Dado** un endpoint que falla, **Cuando** se abre Inicio, **Entonces** solo ese widget muestra
   un mensaje de error con acción de reintento; el resto del tablero sigue visible y usable.
6. **Dado** un usuario que quiere operar, **Cuando** usa los accesos rápidos, **Entonces** llega a
   Caja, Movimientos, Transferencias o Asistente según su rol.
7. **Dado** una base sin movimientos, **Cuando** se abre Inicio, **Entonces** los gráficos muestran
   un estado vacío explicativo, no un área en blanco ni un error.

**Checkpoint**: Inicio es un tablero real y autónomo; las páginas de operación y administración aún
no están rediseñadas.

---

### User Story 18.3 - Acceso rediseñado y coherente (Prioridad: P3)

Como **visitante o usuario con contraseña temporal**, quiero que las pantallas de login, registro,
verificación de correo y cambio obligatorio de contraseña compartan el lenguaje visual del resto de
la aplicación, para percibir un producto cuidado y confiar en él desde el primer contacto.

**Por qué esta prioridad**: es la puerta de entrada y hoy conserva una identidad anterior
(crema/naranja) que ya no representa al producto; su comportamiento, en cambio, no debe cambiar.

**Prueba independiente**: ejecutar los cuatro recorridos de acceso —login con y sin SSO, registro,
verificación con y sin token, y cambio obligatorio de contraseña— y comprobar que los campos,
validaciones, mensajes, redirecciones y controles de sesión son los mismos, con la nueva estética y
en ambos temas.

**Escenarios de aceptación**:

1. **Dado** un visitante, **Cuando** abre `/login`, **Entonces** ve una tarjeta centrada con marca,
   título, campos de email y contraseña, acción principal, enlaces a registro y el selector de tema,
   en el lenguaje visual de las muestras.
2. **Dado** un login con credenciales inválidas, **Cuando** se envía, **Entonces** se muestra el
   mensaje real de la API sin revelar datos y sin perder lo escrito.
3. **Dado** un usuario con contraseña temporal, **Cuando** inicia sesión, **Entonces** ve el modo de
   cambio obligatorio con la misma estética, la política de contraseña y los mensajes de error
   actuales.
4. **Dado** un entorno sin SSO configurado, **Cuando** se abre `/login`, **Entonces** no se muestran
   botones de proveedores y se informa que no hay SSO disponible, como hoy.
5. **Dado** un registro exitoso o un enlace de verificación sin token, **Cuando** se completa la
   acción, **Entonces** se ve el mismo mensaje y la misma navegación que antes, con la nueva
   presentación.
6. **Dado** cualquier campo del acceso, **Cuando** se navega solo con teclado, **Entonces** el foco
   es visible, el orden es lógico y los errores se anuncian con `role="alert"`.
7. **Dado** un tema oscuro guardado, **Cuando** se abre `/login`, **Entonces** la pantalla se pinta
   oscura desde el primer frame y el selector refleja la preferencia.

**Checkpoint**: los cuatro recorridos de acceso están rediseñados y sin regresiones; el marco
privado ya está listo.

---

### User Story 18.4 - Operación diaria en tarjetas (Prioridad: P4)

Como **operario o gerente**, quiero que Caja, Movimientos, Transferencias, Conteos y Asistente
presenten sus formularios, resultados e historiales en tarjetas claras y consistentes, para operar
más rápido y leer el resultado de cada acción sin ambigüedad.

**Por qué esta prioridad**: son las pantallas de uso intensivo; su valor depende del marco y de las
primarias visuales, pero no bloquean el acceso ni el tablero.

**Prueba independiente**: en cada página, ejecutar su flujo principal (buscar en Caja, registrar un
movimiento, transferir, abrir y cerrar/consultar un conteo, enviar una consulta al asistente) y
comprobar que el resultado concreto —producto, depósito, cantidad y stock resultante— se muestra en
la tarjeta de resultado con la nueva presentación.

**Escenarios de aceptación**:

1. **Dado** el usuario en Caja, **Cuando** busca por SKU o elige categoría, **Entonces** ve los
   filtros en una tarjeta, el resultado en tabla o tarjetas y el aviso de truncados con la nueva
   jerarquía visual.
2. **Dado** el usuario en Movimientos, **Cuando** registra un movimiento, **Entonces** la tarjeta de
   resultado muestra tipo, producto, depósito, cantidad y stock resultante, y el historial se
   actualiza sin recargar la página.
3. **Dado** el usuario en Transferencias, **Cuando** confirma una transferencia, **Entonces** ve el
   stock resultante en origen y destino y el historial mantiene su paginación.
4. **Dado** el usuario en Conteos, **Cuando** abre un conteo, **Entonces** la tabla de líneas, los
   estados y las acciones de cerrar/cancelar mantienen su comportamiento y se leen con badges.
5. **Dado** el usuario en Asistente, **Cuando** envía una consulta, **Entonces** se ve el estado de
   carga, la respuesta y, si hay propuesta, la tarjeta de confirmación con la misma jerarquía visual
   que el resto.
6. **Dado** un error de dominio, **Cuando** ocurre en cualquiera de estas páginas, **Entonces** se
   muestra en la tarjeta correspondiente con el mensaje real de la API y una acción para reintentar
   o corregir.

**Checkpoint**: operación diaria rediseñada; queda la administración.

---

### User Story 18.5 - Administración en tablas de datos (Prioridad: P5)

Como **gerente**, quiero que las ocho pantallas de administración usen el mismo patrón de tabla de
datos —encabezado, filtros, badges de estado, acciones por fila y paginación— para revisar y
gestionar el catálogo, los depósitos, el stock, las compras y la auditoría con menos esfuerzo
visual.

**Por qué esta prioridad**: son pantallas de menor frecuencia y dependen de todas las primitivas
anteriores; su ausencia no impide operar, pero completa el rediseño.

**Prueba independiente**: en cada una de las ocho páginas, listar, filtrar, paginar y ejecutar una
acción de fila (editar, desactivar, reactivar, confirmar, anular, ver) verificando que el resultado
y los mensajes son los de antes, con la nueva presentación.

**Escenarios de aceptación**:

1. **Dado** el usuario en Usuarios, Categorías, Productos, Depósitos, Proveedores, Compras o
   Auditoría, **Cuando** abre la página, **Entonces** ve un encabezado con título, descripción y
   acción principal, una barra de filtros y una tarjeta de tabla con badges de estado y acciones.
2. **Dado** el usuario en Productos, **Cuando** busca, filtra por categoría o cambia entre
   Activos/Inactivos/Todos, **Entonces** la tabla y el contador se actualizan igual que hoy.
3. **Dado** el usuario en Stock, **Cuando** filtra por depósito o bajo mínimo, **Entonces** ve la
   tabla con énfasis visual en las filas bajo mínimo y la paginación conserva su comportamiento.
4. **Dado** el usuario en Depósitos, **Cuando** crea un depósito, elige uno y gestiona su árbol de
   ubicaciones, **Entonces** las dos columnas y el panel de ubicaciones mantienen su lógica y se
   presentan como tarjetas.
5. **Dado** el usuario en Compras, **Cuando** confirma o anula una compra, **Entonces** el estado se
   refleja con badge y el mensaje concreto de la API se muestra en la página.
6. **Dado** una tabla con muchas filas, **Cuando** se pagina o se cambia el tamaño de ventana,
   **Entonces** el encabezado de la tabla se mantiene legible y no hay desborde del cuerpo de la
   página.
7. **Dado** una tabla vacía, **Cuando** no hay resultados, **Entonces** se muestra un estado vacío
   con la acción siguiente sugerida (crear, limpiar filtros o ampliar el filtro).

**Checkpoint**: las 17 capturas están rediseñadas; resta la evidencia y la verificación final.

---

### Edge Cases

- **Nombres largos**: SKU, email, nombre de producto o de depósito exceden la columna; se truncan
  con elipsis y `title`, sin romper la fila ni empujar acciones fuera de vista.
- **Volumen**: 10 000 productos o páginas de 100 filas; la tabla pagina y el layout no se desplaza.
- **Datos vacíos**: ceros, listas vacías y gráficos sin serie muestran estado vacío explicativo, no
  un error.
- **API caída o lenta**: cada tarjeta o tabla muestra su propio error con reintento; el marco y el
  resto de la página siguen utilizables.
- **Permisos**: un `OPERACION` no ve widgets ni ítems de gestión; si una llamada devuelve 403, se
  oculta el widget y no se muestra un error crudo.
- **Almacenamiento bloqueado**: si `localStorage` falla, el tema funciona en memoria durante la
  sesión y no rompe ninguna pantalla.
- **Sin preferencia guardada**: se sigue al sistema; si el sistema no expone preferencia, se usa
  claro.
- **Cambio de tema con modal abierto**: el modal, su foco y su contraste se mantienen.
- **Barra lateral colapsada**: etiquetas accesibles y tooltips; el ítem activo sigue visible.
- **320 px**: la tabla se desplaza dentro de su tarjeta, nunca el `body`; los botones conservan
  tamaño táctil.
- **Movimiento reducido**: sin animaciones no esenciales cuando `prefers-reduced-motion` está
  activo.
- **Impresión**: fuera de alcance; no se promete una hoja de impresión.

## Requisitos Funcionales

### Identidad, tema y accesibilidad

- **FR-079 Capa única de tokens.** Toda la interfaz DEBE consumir una única capa de tokens visuales
  (color, superficie, borde, texto, estado, tipografía, espaciado, radio y sombra) con variantes
  clara y oscura; las páginas NO definen colores sueltos fuera de esa capa.
- **FR-080 Tema Light/Dark/System.** La preferencia DEBE poder elegirse entre claro, oscuro y
  seguir al sistema, persistir en el navegador y aplicarse antes del primer pintado para evitar el
  destello del tema anterior; sin preferencia guardada se sigue al sistema y, ante un fallo de
  almacenamiento, se usa claro sin romper la pantalla.
- **FR-081 Login rediseñado.** `/login` DEBE conservar exactamente los campos, validaciones, SSO
  disponible, mensajes de error, redirección y control de sesión de 001, con la nueva identidad
  visual y el selector de tema.
- **FR-082 Estados visibles y contraste.** Cada pantalla DEBE representar carga, vacío, error y
  guardado en curso, y todo texto y control DEBE alcanzar contraste WCAG AA en ambos temas.
- **FR-083 Cambio obligatorio de contraseña.** El modo de cambio obligatorio DEBE compartir la
  estética del acceso, mantener la política y los mensajes de 001 y no permitir operar hasta
  completarlo.
- **FR-084 Adaptabilidad 320–1440 px.** Ninguna pantalla DEBE generar desborde horizontal del
  `body` entre 320 px y 1440 px; las tablas anchas se desplazan dentro de su tarjeta y la barra
  lateral se vuelve panel superpuesto en anchos reducidos.
- **FR-085 Accesibilidad.** La interfaz DEBE tener etiquetas asociadas a campos, foco visible,
  navegación por teclado, landmarks (`nav`, `main`, `header`), `aria-current` en la ruta activa,
  nombres accesibles en botones icónicos y respeto por `prefers-reduced-motion`.
- **FR-086 Identidad de referencia.** La identidad visual DEBE seguir las muestras `light-2` y
  `dark-2`: marco de administración con barra lateral blanca (clara) o azul noche (oscura), barra
  superior, fondo gris azulado o azul noche, tarjetas de superficie con borde y radio moderado,
  acento primario azul, acentos secundarios teal y ámbar, tipografía sans compacta y jerarquía
  título/subtítulo/etiqueta. La paleta crema/naranja de la iteración anterior queda retirada.
- **FR-087 Entrega incremental por página.** El rediseño DEBE completarse página por página, y cada
  página se considera terminada solo cuando tiene su evidencia visual en ambos temas, sus estados
  cubiertos y cero regresiones funcionales.

### Marco y navegación

- **FR-088 Marco de escritorio.** Todas las rutas autenticadas DEBEN renderizarse dentro de un marco
  con barra lateral fija, barra superior y área de contenido con scroll propio; las rutas de acceso
  (`/login`, `/registro`, `/verificar-correo`) DEBEN usar un lienzo propio sin barra lateral.
- **FR-089 Navegación agrupada y RBAC.** La barra lateral DEBE agrupar "Operación" (Inicio, Caja,
  Movimientos, Transferencias, Conteos, Asistente) y "Administración" (Productos, Categorías,
  Depósitos, Stock, Proveedores, Compras, Usuarios, Auditoría), con icono, etiqueta y estado activo;
  los ítems de administración SOLO se muestran a `GESTION` y el ítem Stock muestra el contador de
  bajo mínimo cuando es mayor que cero.
- **FR-090 Tablero de Inicio.** Inicio DEBE mostrar indicadores, gráficos, actividad reciente y
  accesos rápidos construidos únicamente con endpoints existentes, sin nuevas rutas de backend ni
  cambios de esquema; cada widget DEBE resolver su propia carga, vacío y error y respetar el rol.
- **FR-091 Patrón de página operativa.** Caja, Movimientos, Transferencias, Conteos y Asistente
  DEBEN usar el patrón encabezado (título, descripción, acción principal) + tarjeta de formulario +
  tarjeta de resultado/historial, conservando el resultado concreto que exige el Principio III.
- **FR-092 Patrón de página de administración.** Usuarios, Categorías, Productos, Depósitos, Stock,
  Proveedores, Compras y Auditoría DEBEN usar el patrón encabezado + barra de filtros + tarjeta de
  tabla con badges de estado, acciones por fila, contador y paginación, sin alterar validaciones ni
  permisos.
- **FR-093 Catálogo de componentes reutilizables.** La interfaz DEBE apoyarse en un catálogo acotado
  de componentes visuales (botón, campo, tarjeta, badge, tabla, paginación, modal, aviso, estado
  vacío, esqueleto, KPI, gráficos e iconos) y NO DEBE duplicar primitivas equivalentes por página.
- **FR-094 Acceso secundario.** `/registro` y `/verificar-correo` DEBEN adoptar el mismo lienzo,
  tarjeta y tipografía que `/login`, conservando sus mensajes y navegación actuales.
- **FR-095 Controles de sesión y tema.** La barra superior DEBE exponer el selector Light/Dark/System
  y el menú de usuario con nombre, rol y cierre de sesión, reutilizando el store de sesión y el
  comportamiento de 001.
- **FR-096 Evidencia visual.** Cada página rediseñada DEBE entregarse con capturas a 1440×900 en
  tema claro y oscuro, más una comprobación a 320 px sin desborde, almacenadas como evidencia de la
  spec.
- **FR-097 Sin cambios de comportamiento.** El rediseño NO DEBE modificar endpoints, contratos,
  esquema, reglas de negocio, validaciones ni permisos; la suite existente DEBE seguir en verde y
  solo se agregan pruebas unitarias para la lógica pura nueva (navegación, formato y escalas de
  gráficos).

## Entidades Clave *(modelo de diseño, sin datos nuevos)*

- **Página**: ruta existente con su captura de origen, título, grupo de navegación, rol mínimo,
  patrón (marco, acceso, operativa, administración o tablero) y evidencia visual.
- **Ítem de navegación**: etiqueta, ruta, icono, grupo, visibilidad por familia de rol y contador
  opcional.
- **Token visual**: nombre semántico, valor en tema claro, valor en tema oscuro y uso permitido.
- **Componente de interfaz**: nombre, variantes, estados, accesibilidad y páginas que lo consumen.
- **Widget de tablero**: título, endpoint de origen, transformación, rol mínimo, estado vacío y
  estado de error.

El detalle de estas entidades vive en `data-model.md` del plan.

## Criterios de Éxito Medibles

- **SC-032** El tema elegido se conserva entre recargas y se aplica antes del primer pintado: con
  `dark` guardado, 0 destellos claros observables en 10 recargas de 5 rutas distintas.
- **SC-033** Cero regresiones funcionales en los cuatro recorridos de acceso (login con y sin SSO,
  registro, verificación con y sin token y cambio obligatorio de contraseña): los escenarios de 001
  se cumplen y la suite de frontend queda en verde.
- **SC-034** Ninguna de las 18 rutas presenta desborde horizontal entre 320 px y 1440 px, y todo
  texto y control alcanza contraste AA en ambos temas.
- **SC-035** Comparación visual contra `light-2`/`dark-2` a 1440×900: el marco (barra lateral, barra
  superior, área de contenido), las tarjetas, la paleta y la tipografía se reconocen como la misma
  familia visual en el 100 % de las páginas.
- **SC-036** Cada página rediseñada se entrega con sus dos capturas (claro y oscuro), su chequeo a
  320 px y cero regresiones en sus flujos existentes.
- **SC-037** Las 17 capturas de `captures/` tienen su página rediseñada y alcanzable desde el marco;
  ningún ítem de navegación queda sin página y ninguna página queda fuera de la navegación.

## Supuestos

- Las muestras `captures/samples/light-2.png` y `dark-2.png` son la referencia definitiva y
  reemplazan la identidad crema/naranja de la iteración anterior de 018.
- El objetivo es escritorio: 1440×900 como tamaño de referencia, con adaptación funcional hasta
  320 px; no se diseña una experiencia móvil específica más allá del panel lateral superpuesto.
- No hay cambios de backend, API ni base de datos; el tablero lee endpoints ya implementados y
  tolera que alguno no esté disponible según el rol o el entorno.
- No se incorporan librerías de gráficos, iconos, componentes ni fuentes externas; se usan SVG/CSS
  propios y la pila tipográfica del sistema.
- Los textos siguen en español (es-AR), sin internacionalización nueva.
- La implementación parcial existente de tema (`tema.ts`, script de `app.html`, login con paleta
  crema/naranja) se conserva en su lógica y se re-piela con la nueva identidad; no se descarta el
  trabajo de accesibilidad ni el store.
- Las capturas de evidencia se guardan dentro de esta spec y no reemplazan a `captures/`, que queda
  como evidencia del estado anterior.
- Las páginas de 013 (ventas), 014 (reportes) y 016 (importación) no tienen captura ni pantalla
  propia en esta iteración; si existen rutas futuras, adoptan el marco por FR-088 sin rediseño
  específico en esta spec.

## Fuera de Alcance

- Nuevas funcionalidades de negocio, endpoints, migraciones o cambios de permisos.
- Diseño móvil nativo, aplicación de escritorio empaquetada o PWA.
- Impresión, exportación a PDF o captura de pantalla integrada.
- Gráficos en tiempo real, analítica avanzada o predicción.
- Internacionalización, RTL o temas por usuario persistidos en el servidor.
- Reemplazo del stack SvelteKit/Tailwind o adopción de un framework de componentes externo.
- Rediseño de páginas que no estén en `captures/`.

## Dependencias y orden de implementación

1. Tokens, primitivas y marco (FR-079, FR-080, FR-084…FR-089, FR-093, FR-095).
2. Re-piel del acceso y sus modos (FR-081, FR-083, FR-094).
3. Tablero de Inicio (FR-090).
4. Páginas de operación (FR-091).
5. Páginas de administración (FR-092).
6. Evidencia visual, accesibilidad y regresión (FR-082, FR-085, FR-096, FR-097; SC-032…SC-037).

La spec no introduce migraciones ni dependencias de backend; se apoya en los contratos existentes de
003–017.
