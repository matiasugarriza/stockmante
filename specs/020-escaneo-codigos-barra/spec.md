# Feature Specification: Escaneo y generación de códigos de barras/SKU

**Feature Branch**: `020-escaneo-codigos-barra`

**Created**: 2026-11-07

**Status**: Clarified

**Input**: User description: "Generar / Escanear Código de Barras o SKU: agilizar la lectura e
identificación de productos en almacén usando lector de código de barras o cámara."

## Clarifications

### Session 2026-11-07

- Q: Sin un lector físico disponible, ¿con qué medios se lee un código? → A: la cámara del
  celular es el medio principal; el lector tipo pistola (USB/Bluetooth, emula teclado) se
  soporta sin configuración y la entrada manual queda como respaldo.
- Q: La cámara solo funciona en un contexto seguro: ¿cómo se abre la app desde el celular? → A:
  por HTTPS en la red local, con certificado autofirmado del servidor de desarrollo y la API
  servida desde el mismo origen; el celular acepta el certificado una vez.
- Q: ¿Qué formatos se pueden elegir al generar/imprimir? → A: EAN-13, Code 128 y QR; al
  escanear con cámara se reconocen además EAN-8, UPC-A y Code 39.
- Q: ¿Qué valor lleva la etiqueta y dónde se elige el formato? → A: el código de barras del
  producto (o el SKU si no tiene, solo en Code 128 o QR); el formato se elige al imprimir y no se
  guarda en el producto.
- Q: ¿En qué pantallas aparece el escaneo por cámara? → A: en todos los buscadores de producto:
  caja, movimientos, transferencias, conteos y catálogo.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Escanear un producto con la cámara del celular para identificarlo (Priority: P1)

Un operario o cajero abre un buscador de producto desde el celular, activa la cámara y apunta al
código de barras o QR de un producto. El sistema reconoce el código, identifica el producto y
muestra nombre, SKU y la cantidad disponible en el depósito seleccionado. Si el código no
existe, se lo informa sin inventar datos.

**Why this priority**: es el corazón del caso de uso y el único medio de lectura disponible hoy
(no hay lector físico): reduce la carga manual y los errores de tipeo en el almacén y en la caja.

**Independent Test**: escanear con la cámara del celular un código de un producto cargado
devuelve el producto y su stock; escanear un código desconocido devuelve un aviso de «no
encontrado».

**Acceptance Scenarios**:

1. **Dado** un producto activo con código de barras `7791234567898`, **Cuando** el operario lo
   escanea con la cámara, **Entonces** ve el producto con su nombre, SKU y stock en el depósito
   elegido.
2. **Dado** un código que no pertenece a ningún producto, **Cuando** se escanea, **Entonces** la
   aplicación muestra «No se encontró un producto con ese código» y ofrece la búsqueda manual.
3. **Dado** un producto inactivo, **Cuando** se escanea su código, **Entonces** no aparece en la
   búsqueda de caja ni se agrega a la operación; a un usuario de gestión se le informa que está
   dado de baja y a uno de operación, que no hay un producto activo con ese código (operación no
   ve productos inactivos, regla de 004).
4. **Dado** un escaneo exitoso, **Cuando** el operario agrega el producto a una operación,
   **Entonces** no se escribe nada hasta confirmar la operación normalmente.
5. **Dado** una etiqueta QR o Code 128 generada por Stockmante, **Cuando** se escanea con la
   cámara, **Entonces** se resuelve al mismo producto que su código lineal o su SKU.
6. **Dado** cualquiera de los buscadores de caja, movimientos, transferencias, conteos o
   catálogo, **Cuando** el usuario abre la pantalla, **Entonces** encuentra el control
   «Escanear con cámara» junto al buscador.

---

### User Story 2 - Escanear con un lector físico o ingresar el código a mano (Priority: P1)

Un operario usa un lector de código de barras tipo pistola conectado por USB o Bluetooth que se
comporta como teclado. El código se escribe en el buscador y dispara la búsqueda. Si no tiene
lector ni cámara, puede pegar o escribir el código o el SKU manualmente.

**Why this priority**: es el camino de menor fricción en almacenes con lectores y garantiza el
funcionamiento cuando la cámara no está disponible.

**Independent Test**: con el foco en el buscador, un lector que envía `7791234567898` + Enter
dispara la búsqueda y devuelve el producto; el ingreso manual del mismo valor produce el mismo
resultado.

**Acceptance Scenarios**:

1. **Dado** un lector físico que emula teclado, **Cuando** escanea un código con el foco en el
   buscador, **Entonces** el código se ingresa y la búsqueda se ejecuta.
2. **Dado** un código escrito a mano con espacios o en minúsculas, **Cuando** se busca,
   **Entonces** el sistema lo normaliza y encuentra el producto.
3. **Dado** un SKU válido, **Cuando** se busca por SKU, **Entonces** se encuentra el producto
   aunque no tenga código de barras.
4. **Dado** un lector que envía un código inválido, **Cuando** se busca, **Entonces** la UI
   informa que no hay resultados sin romper la pantalla.

---

### User Story 3 - Generar y asignar un código de barras a un producto (Priority: P2)

Un administrador o gerente abre la ficha de un producto sin código de barras y genera un EAN-13
válido a partir de 12 dígitos o de forma automática. El sistema calcula el dígito verificador,
verifica que no exista en otro producto y lo asigna. Si el producto ya tiene código, se pide
confirmación antes de reemplazarlo. Los códigos internos alfanuméricos se siguen pudiendo cargar
a mano en la ficha, como hasta ahora.

**Why this priority**: completa el ciclo «generar» del caso de uso y permite etiquetar productos
que llegan sin código del proveedor.

**Independent Test**: generar un EAN-13 para un producto sin código, guardarlo y verificarlo con
la cámara o con la búsqueda por código; intentar asignar un código ya usado devuelve un error.

**Acceptance Scenarios**:

1. **Dado** un producto sin código de barras, **Cuando** el gestor genera un EAN-13 y lo guarda,
   **Entonces** el producto queda con un código válido y único.
2. **Dado** un EAN-13 de 13 dígitos con dígito verificador incorrecto, **Cuando** se intenta
   guardar, **Entonces** el sistema lo rechaza y explica cómo corregirlo.
3. **Dado** un código ya asignado a otro producto, **Cuando** se intenta asignar, **Entonces** el
   sistema responde con un error de duplicado y no cambia nada.
4. **Dado** un producto que ya tiene código, **Cuando** el gestor intenta reemplazarlo, **Entonces**
   el sistema pide confirmación explícita antes de guardar.
5. **Dado** un usuario de operación, **Cuando** intenta generar o asignar un código, **Entonces**
   el sistema no le permite la acción.

---

### User Story 4 - Generar una etiqueta imprimible en el formato elegido (Priority: P2)

Un administrador o gerente genera una etiqueta para un producto eligiendo el formato (EAN-13,
Code 128 o QR), con el código, el nombre y el SKU, y la imprime o la descarga para pegarla en el
estante o en el producto. La etiqueta debe ser legible por la cámara del celular y por un lector
físico.

**Why this priority**: cierra el ciclo físico del inventario: el código generado sirve para
identificar el producto en el depósito.

**Independent Test**: generar la etiqueta de un producto en cada formato, imprimirla o
descargarla, y escanearla con la cámara del celular; el valor leído debe resolver al mismo
producto.

**Acceptance Scenarios**:

1. **Dado** un producto con código, **Cuando** el gestor genera la etiqueta, **Entonces** elige
   el formato y ve una vista previa con el código, el nombre y el SKU.
2. **Dado** la vista previa, **Cuando** el gestor imprime o descarga, **Entonces** obtiene una
   etiqueta legible y escaneable.
3. **Dado** un producto sin código de barras, **Cuando** elige Code 128 o QR, **Entonces** la
   etiqueta codifica el SKU; **Cuando** elige EAN-13, **Entonces** el sistema lo guía a generar
   el código primero.
4. **Dado** un producto con un código interno que no es un EAN-13 válido, **Cuando** abre la
   etiqueta, **Entonces** EAN-13 no se ofrece y se explica por qué.
5. **Dado** un usuario de operación, **Cuando** intenta generar etiquetas, **Entonces** el
   sistema no le permite la acción.

---

### Edge Cases

- La cámara no está disponible, el navegador no la soporta, la página no es segura (HTTP en la
  red local) o el usuario niega el permiso: se explica el motivo, se ofrece el lector físico y la
  entrada manual, sin bloquear la pantalla.
- El código está borroso, mal iluminado o inclinado: se permite reintentar y se muestra un aviso
  claro; no se inventa un producto.
- El mismo código aparece dos veces en el campo de visión: se toma una sola lectura; cada
  activación de la cámara produce una lectura y se cierra.
- Se escanea un código que no existe: se informa y se ofrece buscarlo por nombre/SKU; un usuario
  de gestión puede crear el producto con el código precargado, uno de operación no.
- El código escaneado corresponde a un producto inactivo: se informa la baja y no se agrega a la
  operación.
- El código escaneado no es EAN-13 (por ejemplo, un código interno alfanumérico o el contenido
  de un QR): se busca por coincidencia exacta y, si no existe, se trata como desconocido.
- Un QR ajeno contiene texto largo o una URL: se busca igual por coincidencia exacta; si supera
  el largo admitido por la búsqueda se informa que el código no pertenece a ningún producto.
- El producto ya tiene un código y se intenta reemplazar: se pide confirmación explícita.
- El código generado ya existe en otro producto: se rechaza y se sugiere generar otro.
- La red falla durante la búsqueda o la asignación: se muestra un error accionable y no se
  pierde el código escaneado.
- La etiqueta se imprime fuera de escala: se advierte y se ofrece una vista previa antes de
  imprimir.
- El usuario no tiene permisos para generar/asignar/imprimir: la acción no se muestra y la API
  responde con un error de permisos.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-134**: El sistema DEBE permitir leer un código con la cámara del dispositivo (incluida la
  cámara trasera del celular) bajo demanda explícita del usuario.
- **FR-135**: El sistema DEBE permitir leer un código de barras con un lector físico tipo pistola
  que se comporte como teclado, sin configuración especial.
- **FR-136**: El sistema DEBE permitir ingresar el código o el SKU manualmente como alternativa
  a la cámara y al lector.
- **FR-137**: El sistema DEBE normalizar el código/SKU leído (espacios, mayúsculas/minúsculas)
  antes de buscar.
- **FR-138**: El sistema DEBE resolver una lectura a un producto por coincidencia exacta de
  código de barras y, si no hay, por SKU normalizado.
- **FR-139**: El sistema DEBE mostrar, tras una lectura exitosa, el nombre, el SKU y la cantidad
  disponible del producto en el depósito seleccionado.
- **FR-140**: El sistema DEBE informar cuando el código no corresponde a ningún producto y
  ofrecer la búsqueda manual; no DEBE inventar ni aproximar un producto.
- **FR-141**: El sistema DEBE informar cuando el producto está inactivo y no DEBE agregarlo a
  una operación de venta.
- **FR-142**: El sistema DEBE permitir a un usuario de gestión generar un código EAN-13 válido a
  partir de 12 dígitos o de forma automática.
- **FR-143**: El sistema DEBE calcular y validar el dígito verificador del EAN-13.
- **FR-144**: El sistema DEBE rechazar un código ya asignado a otro producto y no DEBE modificar
  nada en ese caso.
- **FR-145**: El sistema DEBE pedir confirmación explícita antes de reemplazar el código de un
  producto que ya tiene uno.
- **FR-146**: El sistema DEBE permitir a un usuario de gestión generar una etiqueta imprimible
  con el código, el nombre y el SKU.
- **FR-147**: La etiqueta generada DEBE ser legible por la cámara del celular y por un lector
  físico en tamaño estándar.
- **FR-148**: El sistema DEBE aplicar permisos: operación puede escanear y buscar; gestión puede
  además generar, asignar e imprimir.
- **FR-149**: El sistema DEBE registrar en la auditoría la generación, asignación o reemplazo de
  un código de barras, con el usuario y la fecha.
- **FR-150**: El sistema NO DEBE escribir movimientos de stock como consecuencia de un escaneo;
  el escaneo solo identifica y, si corresponde, agrega a una operación que se confirma aparte.
- **FR-151**: El sistema DEBE manejar la denegación de permiso de cámara, la ausencia de cámara,
  el contexto no seguro y la falta de soporte del navegador con un mensaje accionable y las
  alternativas manuales.
- **FR-152**: El sistema DEBE permitir reintentar una lectura fallida sin recargar la pantalla.
- **FR-153**: El sistema DEBE mantener la búsqueda disponible mientras la cámara está activa y
  DEBE apagarla al salir de la pantalla, al cancelar o tras una lectura exitosa.
- **FR-154**: El sistema DEBE mostrar una vista previa de la etiqueta antes de imprimir o
  descargar.
- **FR-155**: El sistema DEBE conservar el código escaneado si la búsqueda falla por red, para
  permitir reintentar sin volver a escanear.
- **FR-156**: El sistema DEBE cumplir las reglas de accesibilidad: la cámara se activa con un
  control etiquetado, se puede cancelar con teclado y los avisos se anuncian a lectores de
  pantalla.
- **FR-157**: La lectura por cámara DEBE reconocer como mínimo EAN-13, EAN-8, UPC-A, Code 128,
  Code 39 y QR; el contenido de texto de cualquiera de ellos se resuelve con la misma regla de
  FR-138.
- **FR-158**: Al generar la etiqueta, el gestor DEBE poder elegir el formato entre EAN-13,
  Code 128 y QR; EAN-13 solo se ofrece si el código del producto es un EAN-13 válido. El formato
  elegido no se guarda en el producto.
- **FR-159**: La etiqueta DEBE codificar el código de barras del producto; si el producto no
  tiene código, Code 128 y QR DEBEN codificar el SKU.
- **FR-160**: El control de escaneo por cámara DEBE estar disponible en todos los buscadores de
  producto: caja, movimientos, transferencias, conteos y catálogo.
- **FR-161**: La aplicación DEBE poder abrirse desde un celular de la misma red local por HTTPS,
  con la API servida desde el mismo origen, para habilitar la cámara.

### Key Entities *(include if feature involves data)*

- **Producto**: artículo del catálogo; atributos relevantes: nombre, SKU, código de barras,
  estado activo/inactivo y stock por depósito.
- **Código de barras**: valor único asociado a un producto, con formato EAN-13 o interno
  alfanumérico; no puede repetirse entre productos, activos o inactivos.
- **Lectura/escaneo**: evento transitorio de la interfaz con el código leído, el formato
  detectado y el resultado (encontrado, no encontrado, inactivo, error); no se persiste y no
  modifica stock.
- **Etiqueta**: representación imprimible del producto en un formato elegido (EAN-13, Code 128
  o QR) con su código o SKU, nombre y SKU; no se persiste.
- **Usuario y rol**: determina si puede escanear/buscar (operación) o además generar, asignar e
  imprimir (gestión).

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-050**: El 95 % de los escaneos de un código existente muestran el producto en menos de
  2 segundos desde la captura.
- **SC-051**: El 100 % de los códigos leídos se resuelven a lo sumo a un producto; no hay
  ambigüedad ni duplicados.
- **SC-052**: El 100 % de los EAN-13 generados supera la validación del dígito verificador.
- **SC-053**: Un operario completa el recorrido «escanear → identificar → agregar a la
  operación» en menos de 15 segundos en el 90 % de los intentos.
- **SC-054**: Cero movimientos de stock se generan por el solo hecho de escanear.
- **SC-055**: El 100 % de las denegaciones de cámara, ausencias de cámara, contextos no seguros
  o navegadores sin soporte ofrecen una alternativa manual visible.
- **SC-056**: El 100 % de las etiquetas generadas en tamaño estándar se leen correctamente con
  la cámara del celular y con un lector físico.
- **SC-057**: Cero asignaciones de códigos duplicados y cero reemplazos sin confirmación.
- **SC-058**: El 100 % de los intentos de generación/asignación por parte de un usuario de
  operación son rechazados con un mensaje claro.
- **SC-059**: El 100 % de las generaciones, asignaciones y reemplazos quedan registrados en la
  auditoría con usuario y fecha.
- **SC-060**: El 100 % de las etiquetas EAN-13, Code 128 y QR generadas por Stockmante se
  resuelven al producto correcto al escanearlas con la cámara del celular.
- **SC-061**: Las 5 pantallas con buscador de producto (caja, movimientos, transferencias,
  conteos y catálogo) ofrecen el escaneo por cámara.

## Assumptions

- El catálogo ya tiene un campo de código de barras único y un SKU; esta feature lo reutiliza y
  no cambia el modelo de producto (el formato de la etiqueta no se persiste).
- La generación nueva de códigos es EAN-13; los códigos internos alfanuméricos existentes se
  aceptan para búsqueda y se imprimen en Code 128 o QR.
- El usuario no tiene un lector físico hoy: la cámara del celular es el medio principal. Los
  lectores tipo pistola del mercado emulan teclado y escriben el código seguido de Enter.
- La cámara solo se usa con permiso explícito y en un contexto seguro: localhost en el PC o
  HTTPS con certificado autofirmado en la red local para el celular; si no está disponible, se
  usan lector físico o entrada manual.
- La impresión de etiquetas usa la impresión del navegador o una descarga; no se integra un
  driver de impresora ni un servicio externo de etiquetas.
- Operación puede escanear, buscar y agregar a una operación; gestión puede además generar,
  asignar e imprimir.
- La búsqueda y el stock existentes son la fuente de verdad; el escaneo no crea una segunda
  fuente de datos.
- La generación de códigos no reemplaza el código de un producto sin confirmación explícita.
- Quedan fuera de alcance: RFID, NFC, Data Matrix y PDF417, lectura por cámara en navegadores
  sin soporte, escaneo continuo en serie, impresión masiva por lotes, etiquetas con logotipos y
  publicación de la app fuera de la red local (túneles o dominios públicos).
