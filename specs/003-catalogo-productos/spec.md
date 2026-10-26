# Especificación de Funcionalidad: Catálogo de productos y categorías

**Spec**: `003-catalogo-productos` | **Caso de uso**: US3 | **Prioridad**: P2

**Creado**: 2026-10-25 | **Estado**: Replanteo pendiente — WIP de backend deshecho (ver [progress.md](./progress.md))

**Origen**: dividida de la spec original del Módulo 1 (`001-user-auth-catalog`, 2026-10-12).
Los textos de escenarios, requisitos y criterios se conservan **verbatim**; solo se separaron
por caso de uso.

**Entrada**: US3 de la especificación original: "Como Administrador, quiero registrar productos
con datos comerciales (SKU, precio, costo, unidad, categoría, stock mínimo) y darlos de baja
lógica si ya tienen historial, para que el catálogo sea la fuente de verdad para caja y almacén."

**Depende de**: [001-autenticacion-sesion](../001-autenticacion-sesion/spec.md) — identidad,
sesiones, `AppError` y guards RBAC.

## Clarifications

### Session 2026-10-12

- Q: ¿Qué formato debe cumplir el SKU de un producto para ser considerado válido? → A: Mayúsculas, dígitos y guiones; 3–30 caracteres; se normaliza a mayúsculas antes de validar unicidad (ej. `SKU-001`, `POL-CAJA-10`).

> La clarificación de contraseña inicial pertenece a 002; las de login/sesión/roles a 001.

## Escenarios de Usuario y Pruebas *(obligatorio)*

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

> El escenario 6 de US3 en la spec original ("busca por SKU o código de barras en caja en ≤1 s")
> es FR-011/SC-002 y se implementa en [004-busqueda-caja](../004-busqueda-caja/spec.md).

### Casos Borde

- **SKU duplicado o mal formado**: segundo alta con mismo SKU → rechazo; SKU con espacios, minúsculas (tras normalizar igual a existente), caracteres fuera de mayúsculas/dígitos/guiones o longitud fuera de 3–30 → rechazo con mensaje claro; la búsqueda por SKU no debe devolver ambigüedad.
- **Código de barras duplicado / vacío**: vacío es válido (opcional); duplicado se rechaza; se permite limpiar el código de un producto si no se usa en historial.
- **Producto con referencias**: borrado físico bloqueado si hay movimientos, ventas o asignaciones; se ofrece desactivación.

> Los casos borde de búsqueda pertenecen a 004 y los de ubicaciones a 005.

## Requisitos *(obligatorio)*

### Requisitos Funcionales

- **FR-009**: El sistema DEBE permitir CRUD de productos con: nombre, SKU (obligatorio, único y con formato mayúsculas/dígitos/guiones, 3–30 caracteres, normalizado a mayúsculas), código de barras (opcional y único si se informa), categoría, unidad de medida, costo, precio de venta y stock mínimo de seguridad.
- **FR-010**: El sistema DEBE soportar categorías de productos (al menos alta/listado básicos) para organización y filtrado.
- **FR-012**: El sistema DEBE impedir el borrado físico de productos con historial (movimientos, ventas u otras referencias) y ofrecer en su lugar baja lógica (estado inactivo).
- **FR-014**: El cambio de precio de venta y la baja de productos DEBE restringirse al rol Administrador/Gerente.

> FR-003 (modelo RBAC de dos familias) es propiedad de 001; FR-011/FR-013 (búsqueda y
> exclusión de inactivos en venta) pertenecen a 004. La verificación transversal de la matriz
> es de [006](../006-rbac-auditoria/spec.md).

### Entidades Clave

- **Producto**: artículo comercializable; atributos: nombre, SKU (único), código de barras (opcional, único), categoría, unidad de medida, costo, precio de venta, stock mínimo, estado activo/inactivo.
- **Categoría**: agrupación de productos para filtrado y reportes.

## Criterios de Éxito

- **SC-006**: Cero SKUs duplicados y cero códigos de barras duplicados entre productos activos.

## Supuestos

- **Categorías**: alta y listado incluidos; árbol de categorías anidadas no es requisito de v1 (lista plana).
- **Unidad de medida**: catálogo fijo o lista configurable simple (ej. unidad, kg, caja); no se requiere conversión entre unidades en este módulo.
- **Stock mínimo**: atributo de producto para alertas posteriores; este módulo solo lo persiste y muestra, no genera alertas.
- **Idioma de UI**: español (es-AR por defecto), alineado con el resto del proyecto.

**Fuera de alcance global**: ver [specs/README.md](../README.md). En particular, movimientos de
stock, búsqueda para caja y asignación de ubicaciones pertenecen a 004/005.

## Trazabilidad

- Tareas originales: T032–T036 → `T001–T005` en [tasks.md](./tasks.md).
- Migración propia: `0004_productos_codigo_barras_null_distinct.sql` (T003; archivo eliminado al
  deshacer el WIP, el replanteo define su forma final).
- Tablas: `categorias`, `productos` (creadas en la fase foundational de 001, migración `0002`;
  el esquema es propiedad de esta spec).
