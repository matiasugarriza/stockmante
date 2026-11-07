# Feature Specification: Escaneo y generaciÃ³n de cÃ³digos de barras/SKU

**Feature Branch**: `020-escaneo-codigos-barra`

**Created**: 2026-11-07

**Status**: Draft

**Input**: User description: "Generar / Escanear CÃ³digo de Barras o SKU: agilizar la lectura e
identificaciÃ³n de productos en almacÃ©n usando lector de cÃ³digo de barras o cÃ¡mara."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Escanear un producto con la cÃ¡mara para identificarlo (Priority: P1)

Un operario o cajero abre la bÃºsqueda de inventario, activa la cÃ¡mara y apunta al cÃ³digo de
barras de un producto. El sistema reconoce el cÃ³digo, identifica el producto y muestra nombre,
SKU y la cantidad disponible en el depÃ³sito seleccionado. Si el cÃ³digo no existe, se lo informa
sin inventar datos.

**Why this priority**: es el corazÃ³n del caso de uso: reduce la carga manual y los errores de
tipeo en el almacÃ©n y en la caja.

**Independent Test**: escanear con la cÃ¡mara un cÃ³digo de un producto cargado devuelve el
producto y su stock; escanear un cÃ³digo desconocido devuelve un aviso de Â«no encontradoÂ».

**Acceptance Scenarios**:

1. **Dado** un producto activo con cÃ³digo de barras `7791234567890`, **Cuando** el operario lo
   escanea con la cÃ¡mara, **Entonces** ve el producto con su nombre, SKU y stock en el depÃ³sito
   elegido.
2. **Dado** un cÃ³digo que no pertenece a ningÃºn producto, **Cuando** se escanea, **Entonces** la
   aplicaciÃ³n muestra Â«No se encontrÃ³ un producto con ese cÃ³digoÂ» y ofrece la bÃºsqueda manual.
3. **Dado** un producto inactivo, **Cuando** se escanea su cÃ³digo, **Entonces** no aparece en la
   bÃºsqueda de caja y se informa que el producto estÃ¡ dado de baja.
4. **Dado** un escaneo exitoso, **Cuando** el operario agrega el producto a una operaciÃ³n,
   **Entonces** no se escribe nada hasta confirmar la operaciÃ³n normalmente.

---

### User Story 2 - Escanear con un lector fÃ­sico o ingresar el cÃ³digo a mano (Priority: P1)

Un operario usa un lector de cÃ³digo de barras conectado por USB o Bluetooth que se comporta como
teclado. El cÃ³digo se escribe en el buscador y dispara la bÃºsqueda. Si no tiene lector ni cÃ¡mara,
puede pegar o escribir el cÃ³digo o el SKU manualmente.

**Why this priority**: es el camino de menor fricciÃ³n en almacenes con lectores existentes y
garantiza el funcionamiento cuando la cÃ¡mara no estÃ¡ disponible.

**Independent Test**: con el foco en el buscador, un lector que envÃ­a `7791234567890` + Enter
dispara la bÃºsqueda y devuelve el producto; el ingreso manual del mismo valor produce el mismo
resultado.

**Acceptance Scenarios**:

1. **Dado** un lector fÃ­sico que emula teclado, **Cuando** escanea un cÃ³digo con el foco en el
   buscador, **Entonces** el cÃ³digo se ingresa y la bÃºsqueda se ejecuta.
2. **Dado** un cÃ³digo escrito a mano con espacios o en minÃºsculas, **Cuando** se busca,
   **Entonces** el sistema lo normaliza y encuentra el producto.
3. **Dado** un SKU vÃ¡lido, **Cuando** se busca por SKU, **Entonces** se encuentra el producto
   aunque no tenga cÃ³digo de barras.
4. **Dado** un lector que envÃ­a un cÃ³digo invÃ¡lido, **Cuando** se busca, **Entonces** la UI
   informa que no hay resultados sin romper la pantalla.

---

### User Story 3 - Generar y asignar un cÃ³digo de barras a un producto (Priority: P2)

Un administrador o gerente abre la ficha de un producto sin cÃ³digo de barras y genera uno vÃ¡lido
a partir de 12 dÃ­gitos o de forma automÃ¡tica. El sistema calcula el dÃ­gito verificador, verifica
que no exista en otro producto y lo asigna. Si el producto ya tiene cÃ³digo, se pide confirmaciÃ³n
antes de reemplazarlo.

**Why this priority**: completa el ciclo Â«generarÂ» del caso de uso y permite etiquetar productos
que llegan sin cÃ³digo del proveedor.

**Independent Test**: generar un EAN-13 para un producto sin cÃ³digo, guardarlo y verificarlo con
un lector o con la bÃºsqueda por cÃ³digo; intentar asignar un cÃ³digo ya usado devuelve un error.

**Acceptance Scenarios**:

1. **Dado** un producto sin cÃ³digo de barras, **Cuando** el gestor genera un EAN-13 y lo guarda,
   **Entonces** el producto queda con un cÃ³digo vÃ¡lido y Ãºnico.
2. **Dado** un EAN-13 de 13 dÃ­gitos con dÃ­gito verificador incorrecto, **Cuando** se intenta
   guardar, **Entonces** el sistema lo rechaza y explica cÃ³mo corregirlo.
3. **Dado** un cÃ³digo ya asignado a otro producto, **Cuando** se intenta asignar, **Entonces** el
   sistema responde con un error de duplicado y no cambia nada.
4. **Dado** un producto que ya tiene cÃ³digo, **Cuando** el gestor intenta reemplazarlo, **Entonces**
   el sistema pide confirmaciÃ³n explÃ­cita antes de guardar.
5. **Dado** un usuario de operaciÃ³n, **Cuando** intenta generar o asignar un cÃ³digo, **Entonces**
   el sistema no le permite la acciÃ³n.

---

### User Story 4 - Generar una etiqueta imprimible con cÃ³digo y SKU (Priority: P2)

Un administrador o gerente genera una etiqueta para un producto con su cÃ³digo de barras, nombre y
SKU, y la imprime o la descarga para pegarla en el estante o en el producto. La etiqueta debe ser
legible por un lector fÃ­sico.

**Why this priority**: cierra el ciclo fÃ­sico del inventario: el cÃ³digo generado sirve para
identificar el producto en el depÃ³sito.

**Independent Test**: generar la etiqueta de un producto, imprimirla o descargarla, y escanearla
con un lector fÃ­sico; el cÃ³digo leÃ­do debe coincidir con el del producto.

**Acceptance Scenarios**:

1. **Dado** un producto con cÃ³digo, **Cuando** el gestor genera la etiqueta, **Entonces** ve una
   vista previa con el cÃ³digo, el nombre y el SKU.
2. **Dado** la vista previa, **Cuando** el gestor imprime o descarga, **Entonces** obtiene una
   etiqueta legible y escaneable.
3. **Dado** un producto sin cÃ³digo, **Cuando** intenta generar la etiqueta, **Entonces** el
   sistema lo guÃ­a a generar el cÃ³digo primero.
4. **Dado** un usuario de operaciÃ³n, **Cuando** intenta generar etiquetas, **Entonces** el
   sistema no le permite la acciÃ³n.

---

### Edge Cases

- La cÃ¡mara no estÃ¡ disponible, el navegador no la soporta o el usuario niega el permiso: se
  ofrece el lector fÃ­sico y la entrada manual, sin bloquear la pantalla.
- El cÃ³digo estÃ¡ borroso, mal iluminado o inclinado: se permite reintentar y se muestra un aviso
  claro; no se inventa un producto.
- El mismo cÃ³digo aparece dos veces en el campo de visiÃ³n: se toma una sola lectura y se pide
  confirmaciÃ³n si hay ambigÃ¼edad.
- Se escanea un cÃ³digo que no existe: se informa y se ofrece buscarlo por nombre/SKU; un usuario
  de gestiÃ³n puede crear el producto con el cÃ³digo precargado, uno de operaciÃ³n no.
- El cÃ³digo escaneado corresponde a un producto inactivo: se informa la baja y no se agrega a la
  operaciÃ³n.
- El cÃ³digo escaneado no es EAN-13 (por ejemplo, un cÃ³digo interno alfanumÃ©rico): se busca por
  coincidencia exacta y, si no existe, se trata como desconocido.
- El producto ya tiene un cÃ³digo y se intenta reemplazar: se pide confirmaciÃ³n explÃ­cita.
- El cÃ³digo generado ya existe en otro producto: se rechaza y se sugiere generar otro.
- La red falla durante la bÃºsqueda o la asignaciÃ³n: se muestra un error accionable y no se
  pierde el cÃ³digo escaneado.
- La etiqueta se imprime fuera de escala: se advierte y se ofrece una vista previa antes de
  imprimir.
- El usuario no tiene permisos para generar/asignar/imprimir: la acciÃ³n no se muestra y la API
  responde con un error de permisos.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-134**: El sistema DEBE permitir leer un cÃ³digo de barras con la cÃ¡mara del dispositivo
  bajo demanda explÃ­cita del usuario.
- **FR-135**: El sistema DEBE permitir leer un cÃ³digo de barras con un lector fÃ­sico que se
  comporte como teclado, sin configuraciÃ³n especial.
- **FR-136**: El sistema DEBE permitir ingresar el cÃ³digo o el SKU manualmente como alternativa
  a la cÃ¡mara y al lector.
- **FR-137**: El sistema DEBE normalizar el cÃ³digo/SKU leÃ­do (espacios, mayÃºsculas/minÃºsculas)
  antes de buscar.
- **FR-138**: El sistema DEBE resolver una lectura a un producto por coincidencia exacta de
  cÃ³digo de barras y, si no hay, por SKU normalizado.
- **FR-139**: El sistema DEBE mostrar, tras una lectura exitosa, el nombre, el SKU y la cantidad
  disponible del producto en el depÃ³sito seleccionado.
- **FR-140**: El sistema DEBE informar cuando el cÃ³digo no corresponde a ningÃºn producto y
  ofrecer la bÃºsqueda manual; no DEBE inventar ni aproximar un producto.
- **FR-141**: El sistema DEBE informar cuando el producto estÃ¡ inactivo y no DEBE agregarlo a
  una operaciÃ³n de venta.
- **FR-142**: El sistema DEBE permitir a un usuario de gestiÃ³n generar un cÃ³digo EAN-13 vÃ¡lido a
  partir de 12 dÃ­gitos o de forma automÃ¡tica.
- **FR-143**: El sistema DEBE calcular y validar el dÃ­gito verificador del EAN-13.
- **FR-144**: El sistema DEBE rechazar un cÃ³digo ya asignado a otro producto y no DEBE modificar
  nada en ese caso.
- **FR-145**: El sistema DEBE pedir confirmaciÃ³n explÃ­cita antes de reemplazar el cÃ³digo de un
  producto que ya tiene uno.
- **FR-146**: El sistema DEBE permitir a un usuario de gestiÃ³n generar una etiqueta imprimible
  con cÃ³digo de barras, nombre y SKU.
- **FR-147**: La etiqueta generada DEBE ser legible por un lector fÃ­sico en tamaÃ±o estÃ¡ndar.
- **FR-148**: El sistema DEBE aplicar permisos: operaciÃ³n puede escanear y buscar; gestiÃ³n puede
  ademÃ¡s generar, asignar e imprimir.
- **FR-149**: El sistema DEBE registrar en la auditorÃ­a la generaciÃ³n, asignaciÃ³n o reemplazo de
  un cÃ³digo de barras, con el usuario y la fecha.
- **FR-150**: El sistema NO DEBE escribir movimientos de stock como consecuencia de un escaneo;
  el escaneo solo identifica y, si corresponde, agrega a una operaciÃ³n que se confirma aparte.
- **FR-151**: El sistema DEBE manejar la denegaciÃ³n de permiso de cÃ¡mara, la ausencia de cÃ¡mara y
  la falta de soporte del navegador con un mensaje accionable y las alternativas manuales.
- **FR-152**: El sistema DEBE permitir reintentar una lectura fallida sin recargar la pantalla.
- **FR-153**: El sistema DEBE mantener la bÃºsqueda disponible mientras la cÃ¡mara estÃ¡ activa y
  DEBE apagarla al salir de la pantalla o al cancelar.
- **FR-154**: El sistema DEBE mostrar una vista previa de la etiqueta antes de imprimir o
  descargar.
- **FR-155**: El sistema DEBE conservar el cÃ³digo escaneado si la bÃºsqueda falla por red, para
  permitir reintentar sin volver a escanear.
- **FR-156**: El sistema DEBE cumplir las reglas de accesibilidad: la cÃ¡mara se activa con un
  control etiquetado, se puede cancelar con teclado y los avisos se anuncian a lectores de
  pantalla.

### Key Entities *(include if feature involves data)*

- **Producto**: artÃ­culo del catÃ¡logo; atributos relevantes: nombre, SKU, cÃ³digo de barras,
  estado activo/inactivo y stock por depÃ³sito.
- **CÃ³digo de barras**: valor Ãºnico asociado a un producto, con formato EAN-13 o interno; no
  puede repetirse entre productos, activos o inactivos.
- **Lectura/escaneo**: evento de identificaciÃ³n con el cÃ³digo leÃ­do, el usuario, el momento y el
  resultado (encontrado, no encontrado, inactivo, error); no modifica stock por sÃ­ mismo.
- **Etiqueta**: representaciÃ³n imprimible del producto con su cÃ³digo, nombre y SKU; se genera a
  partir de un producto con cÃ³digo vÃ¡lido.
- **Usuario y rol**: determina si puede escanear/buscar (operaciÃ³n) o ademÃ¡s generar, asignar y
  imprimir (gestiÃ³n).

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-050**: El 95 % de los escaneos de un cÃ³digo existente muestran el producto en menos de
  2 segundos desde la captura.
- **SC-051**: El 100 % de los cÃ³digos leÃ­dos se resuelven a lo sumo a un producto; no hay
  ambigÃ¼edad ni duplicados.
- **SC-052**: El 100 % de los EAN-13 generados supera la validaciÃ³n del dÃ­gito verificador.
- **SC-053**: Un operario completa el recorrido Â«escanear â†’ identificar â†’ agregar a la
  operaciÃ³nÂ» en menos de 15 segundos en el 90 % de los intentos.
- **SC-054**: Cero movimientos de stock se generan por el solo hecho de escanear.
- **SC-055**: El 100 % de las denegaciones de cÃ¡mara, ausencias de cÃ¡mara o navegadores sin
  soporte ofrecen una alternativa manual visible.
- **SC-056**: El 100 % de las etiquetas generadas en tamaÃ±o estÃ¡ndar se leen correctamente con
  un lector fÃ­sico.
- **SC-057**: Cero asignaciones de cÃ³digos duplicados y cero reemplazos sin confirmaciÃ³n.
- **SC-058**: El 100 % de los intentos de generaciÃ³n/asignaciÃ³n por parte de un usuario de
  operaciÃ³n son rechazados con un mensaje claro.
- **SC-059**: El 100 % de las generaciones, asignaciones y reemplazos quedan registrados en la
  auditorÃ­a con usuario y fecha.

## Assumptions

- El catÃ¡logo ya tiene un campo de cÃ³digo de barras Ãºnico y un SKU; esta feature lo reutiliza y
  no cambia el modelo de producto.
- El formato objetivo es EAN-13; se aceptan cÃ³digos internos alfanumÃ©ricos existentes para
  bÃºsqueda, pero la generaciÃ³n nueva es EAN-13.
- Los lectores fÃ­sicos del mercado emulan teclado y escriben el cÃ³digo seguido de Enter.
- La cÃ¡mara solo se usa con permiso explÃ­cito y en un contexto seguro (HTTPS o localhost); si no
  estÃ¡ disponible, se usan lector fÃ­sico o entrada manual.
- La impresiÃ³n de etiquetas usa la impresiÃ³n del navegador o una descarga; no se integra un
  driver de impresora ni un servicio externo de etiquetas.
- OperaciÃ³n puede escanear, buscar y agregar a una operaciÃ³n; gestiÃ³n puede ademÃ¡s generar,
  asignar e imprimir.
- La bÃºsqueda y el stock existentes son la fuente de verdad; el escaneo no crea una segunda
  fuente de datos.
- La generaciÃ³n de cÃ³digos no reemplaza el cÃ³digo de un producto sin confirmaciÃ³n explÃ­cita.
- Quedan fuera de alcance: RFID, QR, NFC, lectura por cÃ¡mara en navegadores sin soporte,
  impresiÃ³n masiva por lotes y etiquetas con logotipos o cÃ³digos de barras 2D.
