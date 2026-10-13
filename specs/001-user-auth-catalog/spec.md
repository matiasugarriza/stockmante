# Especificación de Funcionalidad: Módulo 1 — Gestión de Usuarios, Autenticación y Catálogo Base

**Rama de funcionalidad**: `001-user-auth-catalog`

**Creado**: 2026-10-12

**Estado**: Borrador

**Entrada**: Descripción del usuario: "Crear la especificación funcional para el Módulo 1: Gestión de Usuarios, Autenticación y Catálogo Base (Productos, Depósitos y Ubicaciones) de Stockmante. Alcance: CRUD de usuarios con roles Administrador/Gerente y Operario/Cajero; autenticación login/logout/sesión con RBAC; unicidad de correo; baja lógica de usuarios; CRUD de productos con SKU único y código de barras opcional/único; búsqueda rápida por SKU/código de barras/nombre/categoría para caja y almacén; baja lógica de productos; CRUD de depósitos y ubicaciones jerárquicas; asignación inicial de ubicaciones por producto; casos de uso para Administrador y Operario; edge cases (SKU duplicado, desactivar usuario en sesión, ubicaciones huérfanas)."

## Escenarios de Usuario y Pruebas *(obligatorio)*

### User Story 1 - Autenticación y sesión segura (Prioridad: P1)

Como **Operario/Cajero**, quiero iniciar sesión con mi correo y contraseña para acceder solo a las funciones que mi rol me permite, y cerrar sesión cuando termino mi turno para que nadie más use mi cuenta desde el puesto.

**Por qué esta prioridad**: Sin autenticación no hay control de acceso; es la puerta de entrada al resto del sistema y protege la integridad del inventario.

**Prueba independiente**: Se puede probar completamente iniciando sesión con credenciales válidas, verificando que el operario ve solo su alcance de pantallas/acciones, y cerrando sesión; entrega valor (acceso controlado) sin necesidad del catálogo.

**Escenarios de aceptación**:

1. **Dado** un usuario activo con rol Operario, **Cuando** envía correo y contraseña correctos, **Entonces** se crea una sesión y se redirige al panel operativo sin opciones de administración de usuarios.
2. **Dado** un usuario inexistente o contraseña incorrecta, **Cuando** intenta iniciar sesión, **Entonces** se rechaza con mensaje genérico de credenciales inválidas (sin revelar si el correo existe).
3. **Dado** un usuario inactivo, **Cuando** intenta iniciar sesión, **Entonces** se rechaza con mensaje claro de cuenta desactivada.
4. **Dado** un usuario con sesión activa, **Cuando** elige "Cerrar sesión", **Entonces** la sesión queda invalidada y cualquier request posterior exige autenticación.

---

### User Story 2 - Gestión de usuarios por el Administrador (Prioridad: P1)

Como **Administrador**, quiero crear, editar, desactivar y reactivar usuarios con rol y estado, para mantener el personal registrado sin bajas físicas que borren historial.

**Por qué esta prioridad**: El alta de personal es prerrequisito operativo; la baja lógica preserva trazabilidad de movimientos de stock realizados por ese usuario.

**Prueba independiente**: Se prueba creando un usuario, asignándole rol Operario, desactivándolo y confirmando que no puede iniciar sesión pero sus movimientos pasados siguen siendo consultables.

**Escenarios de aceptación**:

1. **Dado** un Administrador autenticado, **Cuando** crea un usuario con correo, nombre, rol y estado activo, **Entonces** el usuario queda registrado y puede iniciar sesión.
2. **Dado** dos intentos de creación con el mismo correo, **Cuando** se envía el segundo, **Entonces** se rechaza por unicidad de correo con mensaje claro.
3. **Dado** un usuario activo con sesión abierta, **Cuando** el Administrador lo desactiva, **Entonces** su sesión actual queda invalidada (o su siguiente request es rechazada) y no puede volver a iniciar sesión.
4. **Dado** un usuario desactivado, **Cuando** el Administrador lo reactiva, **Entonces** puede volver a iniciar sesión con sus mismas credenciales.
5. **Dado** un Operario autenticado, **Cuando** intenta acceder a la gestión de usuarios, **Entonces** se deniega el acceso.

---

### User Story 3 - Catálogo de productos y categorías (Prioridad: P2)

Como **Administrador**, quiero registrar productos con datos comerciales (SKU, precio, costo, unidad, categoría, stock mínimo) y darlos de baja lógica si ya tienen historial, para que el catálogo sea la fuente de verdad para caja y almacén.

**Por qué esta prioridad**: Sin productos no hay ventas ni movimientos; es el catálogo base sobre el que se construirán los módulos de stock.

**Prueba independiente**: Se prueba creando un producto con SKU único, buscándolo por nombre y SKU, y desactivándolo; entrega valor usable para el Administrador sin depender de depósitos completos.

**Escenarios de aceptación**:

1. **Dado** un Administrador autenticado, **Cuando** crea un producto con nombre, SKU, unidad de medida, costo, precio de venta, stock mínimo y categoría, **Entonces** el producto queda activo en el catálogo.
2. **Dado** un producto con SKU "SKU-001", **Cuando** se intenta crear otro con el mismo SKU, **Entonces** se rechaza por unicidad de SKU.
3. **Dado** un producto sin código de barras, **Cuando** se crea, **Entonces** es válido; si se asigna un código de barras ya usado por otro producto, se rechaza.
4. **Dado** un producto sin referencias de historial ni stock, **Cuando** el Administrador lo elimina (o desactiva), **Entonces** sale del catálogo activo.
5. **Dado** un producto con movimientos de stock o líneas de venta, **Cuando** se intenta borrado físico, **Entonces** se impide y se ofrece/sugiere baja lógica; el producto queda inactivo y no aparece en ventas nuevas.
6. **Dado** el Operario en interfaz de caja, **Cuando** busca por SKU o código de barras completo, **Entonces** el resultado relevante aparece en el orden de un segundo o menos (ver SC-002).

---

### User Story 4 - Búsqueda y filtrado rápido para caja y almacén (Prioridad: P2)

Como **Operario/Cajero**, quiero buscar productos por SKU, código de barras, nombre o categoría con resultados inmediatos, para no frenar la atención al cliente.

**Por qué esta prioridad**: La velocidad de búsqueda impacta directamente la experiencia de caja y la productividad del almacén.

**Prueba independiente**: Se prueba ingresando búsquedas parciales y exactas y midiendo tiempo de resultado; no requiere el módulo de depósitos completo.

**Escenarios de aceptación**:

1. **Dado** el catálogo con productos, **Cuando** el Operario escribe un SKU completo, **Entonces** se muestra el producto exacto (o lista mínima).
2. **Dado** un producto con código de barras, **Cuando** se escanea o pega el código completo, **Entonces** se localiza el producto asociado.
3. **Dado** el Operario escribe un fragmento de nombre, **Cuando** hay coincidencias, **Entonces** se listan ordenadas de forma predecible (por relevancia o nombre).
4. **Dado** un Administrador, **Cuando** filtra por categoría, **Entonces** ve solo productos de esa categoría; el Operario en caja puede filtrar o buscar sin necesitar la pantalla completa de administración del catálogo.

---

### User Story 5 - Depósitos y ubicaciones jerárquicas (Prioridad: P3)

Como **Administrador**, quiero registrar depósitos y ubicaciones anidadas (por ejemplo Depósito → Estantería → Nivel → Pasillo) para saber dónde está físicamente cada unidad de stock.

**Por qué esta prioridad**: La trazabilidad espacial es necesaria para el módulo de movimientos y transferencias, pero el sistema puede operar un tiempo con un depósito por defecto.

**Prueba independiente**: Se prueba creando un depósito, agregando ubicaciones jerárquicas y validando que no se puedan crear ubicaciones sin depósito.

**Escenarios de aceptación**:

1. **Dado** un Administrador autenticado, **Cuando** crea un depósito con nombre e identificador/dirección, **Entonces** el depósito queda disponible.
2. **Dado** un depósito existente, **Cuando** se crea una ubicación hija (estantería, nivel, etc.) asociada a ese depósito, **Entonces** la ubicación queda registrada bajo el depósito.
3. **Dado** un intento de crear una ubicación sin depósito padre o con un depósito inexistente, **Cuando** se envía, **Entonces** se rechaza (sin ubicaciones huérfanas).
4. **Dado** un depósito con ubicaciones y productos asignados, **Cuando** se intenta eliminar el depósito, **Entonces** se impide el borrado físico mientras haya dependencias; se ofrece baja lógica si aplica.
5. **Dado** un producto, **Cuando** el Administrador le asigna ubicaciones iniciales permitidas, **Entonces** el producto solo puede recibir stock en esas ubicaciones (regla que consumed los módulos de stock; aquí se define el registro de la asignación).

---

### User Story 6 - Control de acceso por roles en catálogo y usuarios (Prioridad: P2)

Como **Gerente/Administrador**, quiero que las acciones sensibles (CRUD de usuarios, precios de venta, bajas de productos) queden reservadas a mi rol, para que el Operario no altere la configuración del negocio.

**Por qué esta prioridad**: El RBAC evita fraudes y errores operativos; se prueba junto a cada área funcional pero se declara como criterio transversal.

**Prueba independiente**: Se prueba denegando cada acción restringida a un Operario autenticado y confirmándola para el Administrador.

**Escenarios de aceptación**:

1. **Dado** un Operario autenticado, **Cuando** intenta crear o editar un producto, **Entonces** se deniega.
2. **Dado** un Operario autenticado, **Cuando** consulta el catálogo para caja, **Entonces** la lectura está permitida si su rol lo habilita.
3. **Dado** un Administrador, **Cuando** realiza cualquiera de las acciones de gestión, **Entonces** se ejecuta y queda registrada la auditoría básica (quién, qué, cuándo) en la medida definida en Supuestos.

---

### Casos Borde

- **SKU duplicado**: segundo alta con mismo SKU → rechazo con mensaje que no revele datos sensibles de más; la búsqueda por SKU no debe devolver ambigüedad.
- **Código de barras duplicado / vacío**: vacío es válido (opcional); duplicado se rechaza; se permite limpiar el código de un producto si no se usa en historial.
- **Desactivación de usuario activo en sesión**: al desactivar, la sesión vigente queda inválida en el siguiente request (o de inmediato); no debe quedar una sesión "fantasma" que siga moviendo stock.
- **Último Administrador activo**: el sistema no permite desactivar (ni eliminar lógicamente) al único Administrador activo, para no dejar el sistema sin administración.
- **Ubicación huérfana**: crear/modificar una ubicación sin depósito válido → rechazo; mover una ubicación a un depósito distinto solo si no tiene hijos o stock asociado (regla por defecto: se prohíbe si hay dependencias).
- **Producto con referencias**: borrado físico bloqueado si hay movimientos, ventas o asignaciones; se ofrece desactivación.
- **Búsqueda sin resultados / término muy corto**: la búsqueda por nombre exige un mínimo de caracteres (por defecto 2) para no saturar; SKU y código de barras admiten valor exacto completo desde el primer carácter.
- **Correo inválido o ya registrado**: validación de formato y unicidad en el momento del guardado, con mensaje claro.
- **Credenciales en claro**: nunca se muestran ni se guardan en logs; el restablecimiento de contraseña (si se habilita fuera de este alcance mínimo) no forma parte de este módulo salvo decisión explícita en Supuestos.

## Requisitos *(obligatorio)*

### Requisitos Funcionales

**Autenticación y usuarios**

- **FR-001**: El sistema DEBE permitir iniciar sesión con correo electrónico y contraseña; la sesión se mantiene hasta cierre explícito o expiración por inactividad (ver Supuestos: tiempo por defecto).
- **FR-002**: El sistema DEBE permitir cerrar sesión, invalidando la sesión del cliente.
- **FR-003**: El sistema DEBE aplicar control de acceso por rol (RBAC): Administrador/Gerente (gestión completa) y Operario/Cajero (operación de caja/consulta de catálogo, sin gestión de usuarios ni precios).
- **FR-004**: El sistema DEBE garantizar unicidad del correo electrónico de usuario.
- **FR-005**: El sistema DEBE soportar estado Activo/Inactivo mediante baja lógica; los usuarios inactivos no pueden iniciar sesión y sus sesiones activas se invalidan al desactivarse.
- **FR-006**: El sistema DEBE impedir que se desactive al último Administrador activo.
- **FR-007**: El CRUD de usuarios DEBE ser exclusivo del rol Administrador/Gerente.
- **FR-008**: El sistema DEBE validar el formato del correo electrónico y rechazar guardados inválidos con mensaje claro.

**Productos y categorías**

- **FR-009**: El sistema DEBE permitir CRUD de productos con: nombre, SKU (obligatorio y único), código de barras (opcional y único si se informa), categoría, unidad de medida, costo, precio de venta y stock mínimo de seguridad.
- **FR-010**: El sistema DEBE soportar categorías de productos (al menos alta/listado básicos) para organización y filtrado.
- **FR-011**: El sistema DEBE buscar y filtrar productos por SKU, código de barras, nombre y categoría, optimizado para caja y almacén (tiempos ver SC-002).
- **FR-012**: El sistema DEBE impedir el borrado físico de productos con historial (movimientos, ventas u otras referencias) y ofrecer en su lugar baja lógica (estado inactivo).
- **FR-013**: Los productos inactivos NO DEBEN aparecer en búsquedas de venta/carga de movimientos nuevos, pero SÍ deben conservarse para reportes históricos.
- **FR-014**: El cambio de precio de venta y la baja de productos DEBE restringirse al rol Administrador/Gerente.

**Depósitos y ubicaciones**

- **FR-015**: El sistema DEBE permitir CRUD de depósitos con nombre e identificador o dirección.
- **FR-016**: El sistema DEBE permitir CRUD de ubicaciones jerárquicas bajo un depósito (ej. Depósito → Estantería → Nivel → Pasillo), impidiendo ubicaciones huérfanas (sin depósito válido).
- **FR-017**: El sistema DEBE registrar la asignación de ubicaciones iniciales permitidas por producto; las ubicaciones de un producto DEBEN pertenecer a un depósito existente.
- **FR-018**: No DEBE permitirse el borrado físico de un depósito con ubicaciones, productos o movimientos asociados.

### Entidades Clave *(si la funcionalidad implica datos)*

- **Usuario**: identidad de persona en el sistema; atributos clave: nombre, correo (único), credenciales (no expuestas), rol (Administrador/Gerente u Operario/Cajero), estado (Activo/Inactivo).
- **Rol**: definición de permisos; en este módulo, dos familias operativas: gestión (admin/gerente) y operación (operario/cajero).
- **Sesión**: estado de autenticación de un usuario en un momento dado; vincula usuario + vigencia + invalidación.
- **Producto**: artículo comercializable; atributos: nombre, SKU (único), código de barras (opcional, único), categoría, unidad de medida, costo, precio de venta, stock mínimo, estado activo/inactivo.
- **Categoría**: agrupación de productos para filtrado y reportes.
- **Depósito**: recinto físico o lógico de stock; nombre e identificador/dirección.
- **Ubicación**: nodo jerárquico dentro de un depósito (pasillo, estantería, nivel, etc.).
- **Asignación Producto–Ubicación**: registro de ubicaciones permitidas para un producto.

## Criterios de Éxito *(obligatorio)*

### Resultados Mensurables

- **SC-001**: Un Administrador puede crear un usuario operativo y este puede iniciar sesión en menos de 2 minutos desde el alta (incluyendo la primera autenticación).
- **SC-002**: El 95% de las búsquedas de producto por SKU o código de barras completo en interfaz de caja muestran el resultado en 1 segundo o menos.
- **SC-003**: El 100% de los intentos de inicio de sesión de usuarios inactivos o con credenciales inválidas son rechazados, sin diferenciar en el mensaje si el correo existe.
- **SC-004**: Cero sesiones activas de usuarios desactivados pueden ejecutar acciones posteriores a la desactivación (validado en pruebas de seguridad).
- **SC-005**: Cero ubicaciones existen sin depósito asociado (integridad referencial verificable).
- **SC-006**: Cero SKUs duplicados y cero códigos de barras duplicados entre productos activos.
- **SC-007**: El 100% de los intentos de borrado físico de productos o depósitos con dependencias son bloqueados con alternativa de baja lógica ofrecida al Administrador.
- **SC-008**: Ningún usuario con rol Operario puede completar (no solo intentar) una operación de gestión de usuarios, edición de precios o baja de productos.

## Supuestos

- **Autenticación**: se usa autenticación por correo y contraseña con sesión del servidor; MFA/SSO/OAuth quedan fuera de este módulo a menos que se agreguen como requisito posterior.
- **Contraseñas**: almacenamiento con hash irreversible y política básica (longitud mínima 8, al menos una mayúscula y un dígito) salvo que se defina política corporativa distinta.
- **Sesión**: expiración por inactividad por defecto de 8 horas hábiles (ajustable); cierre de sesión del lado del cliente invalida el token/sesión en el servidor.
- **Roles**: se modelan dos familias (Administrador/Gerente y Operario/Cajero). Si Gerente y Administrador difieren en permisos, la diferencia se documenta en planificación; por defecto se tratan como permiso de gestión completo.
- **Auditoría**: se registra al menos autor y timestamp de altas/bajas/cambios sensibles de usuarios y productos; un log de auditoría rico es un candidato a módulo posterior.
- **Categorías**: alta y listado incluidos; árbol de categorías anidadas no es requisito de v1 (lista plana).
- **Unidad de medida**: catálogo fijo o lista configurable simple (ej. unidad, kg, caja); no se requiere conversión entre unidades en este módulo.
- **Stock mínimo**: atributo de producto para alertas posteriores; este módulo solo lo persiste y muestra, no genera alertas.
- **Idioma de UI**: español (es-AR por defecto), alineado con el resto del proyecto.
- **Fuera de alcance de este módulo**: movimientos de stock, transferencias entre depósitos, facturación, reportes financieros, notificaciones de stock mínimo, integración con balanzas/impresoras de etiquetas, importación masiva de productos (CSV/Excel).
- **Depósito por defecto**: es aceptable crear un depósito inicial "Principal" en la puesta en marcha para no bloquear operación mientras se cargan los depósitos reales.
