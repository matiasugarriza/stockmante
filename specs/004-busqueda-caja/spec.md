# Especificación de Funcionalidad: Búsqueda y filtrado rápido para caja

**Spec**: `004-busqueda-caja` | **Caso de uso**: US4 | **Prioridad**: P2

**Creado**: 2026-10-25 | **Estado**: Implementada (2026-10-26); ampliación de clarificaciones 2026-11-07 implementada y
verificada (aviso de truncados; SC-002 medido a 10.000 productos, p95 máximo 86 ms)

**Origen**: dividida de la spec original del Módulo 1 (`001-user-auth-catalog`, 2026-10-12).
Los textos de escenarios, requisitos y criterios se conservan **verbatim**.

**Entrada**: US4 de la especificación original: "Como Operario/Cajero, quiero buscar productos por
SKU, código de barras, nombre o categoría con resultados inmediatos, para no frenar la atención al
cliente."

**Depende de**: [001-autenticacion-sesion](../001-autenticacion-sesion/spec.md) (auth) y
[003-catalogo-productos](../003-catalogo-productos/spec.md) (productos y categorías).

## Clarifications

### Session 2026-11-07

- Q: ¿En qué punto se mide el tiempo de ≤1 s de SC-002? → A: Respuesta de la API vista desde el frontend: desde que `/caja` envía el request hasta que recibe y parsea la respuesta, sin contar el debounce de 250 ms.
- Q: ¿De qué tamaño es el catálogo de referencia de SC-002? → A: 10.000 productos.
- Q: ¿Qué indica `/caja` cuando la búsqueda devuelve exactamente el límite de resultados? → A: Un aviso de resultados truncados que invita a refinar la búsqueda.
- Q: ¿Cómo se compara el texto con `campo=categoria`? → A: Igual que nombre: similitud por trigramas (`%`) o subcadena (`ILIKE '%q%'`) sobre `categorias.nombre`, ordenado por similitud.
- Q: ¿Qué parte de FR-013 le corresponde a 004? → A: Solo la exclusión en la búsqueda (`/api/productos/buscar` y `/caja`); el rechazo en movimientos y ventas es de 007 (FR-028) y 013 (escenario 13.4), y la conservación histórica es de 003 (FR-022, baja lógica) y 014 (kardex de inactivos).

## Escenarios de Usuario y Pruebas *(obligatorio)*

### User Story 4 - Búsqueda y filtrado rápido para caja y almacén (Prioridad: P2)

Como **Operario/Cajero**, quiero buscar productos por SKU, código de barras, nombre o categoría con resultados inmediatos, para no frenar la atención al cliente.

**Por qué esta prioridad**: La velocidad de búsqueda impacta directamente la experiencia de caja y la productividad del almacén.

**Prueba independiente**: Se prueba ingresando búsquedas parciales y exactas y midiendo tiempo de resultado; no requiere el módulo de depósitos completo.

**Escenarios de aceptación**:

1. **Dado** el catálogo con productos, **Cuando** el Operario escribe un SKU completo, **Entonces** se muestra el producto exacto (o lista mínima).
2. **Dado** un producto con código de barras, **Cuando** se escanea o pega el código completo, **Entonces** se localiza el producto asociado.
3. **Dado** el Operario escribe un fragmento de nombre, **Cuando** hay coincidencias, **Entonces** se listan con el orden determinista del contrato: `similarity(nombre, q) DESC`, `nombre ASC`, `id ASC`.
4. **Dado** un usuario de caja o administración, **Cuando** filtra por categoría, **Entonces** ve solo productos de esa categoría; el Operario puede filtrar o buscar sin necesitar la pantalla completa de administración del catálogo. El filtro admite dos formas: elegir la categoría (`categoria_id`, coincidencia exacta, combinable con un término) o escribir parte de su nombre (`campo=categoria`, misma comparación que nombre: `%` o `ILIKE '%q%'`, orden por similitud).

> El escenario 6 de US3 en la spec original ("el Operario busca por SKU o código completo y el
> resultado aparece en ≤1 s") se ejecuta en esta spec.

### Casos Borde

- **Búsqueda sin resultados / término muy corto**: la búsqueda por nombre exige un mínimo de caracteres (por defecto 2) para no saturar; SKU y código de barras admiten valor exacto completo desde el primer carácter.
- **Resultados truncados**: si la búsqueda devuelve exactamente `limite` resultados, `/caja` muestra
  «Se muestran los primeros N resultados. Refiná la búsqueda para encontrar el producto.» (con N =
  límite) en lugar de presentar el contador como total absoluto.

## Requisitos *(obligatorio)*

### Requisitos Funcionales

- **FR-011**: El sistema DEBE buscar y filtrar productos por SKU, código de barras, nombre y categoría, optimizado para caja y almacén (tiempos ver SC-002).
- **FR-013**: Los productos inactivos NO DEBEN aparecer en búsquedas de venta/carga de movimientos nuevos, pero SÍ deben conservarse para reportes históricos.

> FR-003 (RBAC) es de 001 y FR-009/FR-010/FR-012/FR-014 de 003.
>
> **Alcance de FR-013 en 004**: esta spec cubre solo la exclusión de inactivos en
> `GET /api/productos/buscar` y en `/caja`. El rechazo de movimientos y ventas sobre inactivos lo
> cubren [007](../007-movimientos-stock/spec.md) (FR-028) y [013](../013-ventas-caja/spec.md)
> (escenario 13.4); la conservación para reportes históricos, [003](../003-catalogo-productos/spec.md)
> (FR-022, baja lógica) y [014](../014-reportes-kardex/spec.md) (kardex de productos inactivos).

### Entidades Clave

- **Producto** (propiedad de 003): artículo comercializable; esta spec lo consume para buscar y
  filtrar.
- **Categoría** (propiedad de 003): agrupación usada como filtro.

## Criterios de Éxito

- **SC-002**: El 95% de las búsquedas de producto por SKU o código de barras completo en interfaz de caja muestran el resultado en 1 segundo o menos. La misma meta de 1 s aplica a la búsqueda por nombre/categoría, medida sobre el catálogo de referencia (10.000 productos) como peor caso (la similitud es más costosa que la igualdad exacta). El tiempo se mide como respuesta de la API vista desde `/caja`: desde el envío del request hasta recibir y parsear la respuesta; el debounce de 250 ms no se cuenta.

## Supuestos

- **Búsqueda**: se resuelve en PostgreSQL con `pg_trgm` + índice GIN sobre el nombre y consultas
  exactas para SKU/código; la migración es `0007_busqueda_trgm.sql` (la reserva `0005` quedó
  inválida cuando 001 la usó).
- **Inactivos**: la búsqueda de caja excluye `activo = FALSE` por defecto; la gestión puede
  incluirlos con `incluir_inactivos=true`.
- **Idioma de UI**: español (es-AR por defecto).
- **Superficie de UI**: `/caja` es la pantalla compartida para caja y almacén; no se agrega una ruta
  `/almacen` separada. El enlace de navegación se rotula "Caja".

## Contrato

El contrato definitivo está en [contracts/api.md](./contracts/api.md) y reemplaza al preliminar.
Resumen:

| Method | Path | Auth | Query | Éxito |
|---|---|---|---|---|
| GET | `/api/productos/buscar` | sesión | `?q=&campo=sku\|codigo\|nombre\|categoria\|todos&incluir_inactivos=&categoria_id=&limite=` | 200 `{ items: [...] }` orden estable |

- `campo` por defecto es `todos`.
- `nombre` y `categoria` exigen `q` de al menos 2 caracteres; si no, 422. Con `todos` y 1
  carácter solo se evalúan las igualdades exactas de SKU y código (sin 422).
- SKU y código de barras admiten el valor exacto desde el primer carácter.
- `q` vacío solo se admite con `categoria_id`.
- `incluir_inactivos=true` solo para familia GESTION.

## Fuera de alcance

- CRUD del catálogo → 003.
- Movimientos de stock y ventas → módulos posteriores.
- La verificación transversal de RBAC → 006.
