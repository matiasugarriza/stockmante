# Especificación de Funcionalidad: Búsqueda y filtrado rápido para caja

**Spec**: `004-busqueda-caja` | **Caso de uso**: US4 | **Prioridad**: P2

**Creado**: 2026-10-25 | **Estado**: Especificado (sin `plan.md` ni `tasks.md`; se generan al arrancar)

**Origen**: dividida de la spec original del Módulo 1 (`001-user-auth-catalog`, 2026-10-12).
Los textos de escenarios, requisitos y criterios se conservan **verbatim**.

**Entrada**: US4 de la especificación original: "Como Operario/Cajero, quiero buscar productos por
SKU, código de barras, nombre o categoría con resultados inmediatos, para no frenar la atención al
cliente."

**Depende de**: [001-autenticacion-sesion](../001-autenticacion-sesion/spec.md) (auth) y
[003-catalogo-productos](../003-catalogo-productos/spec.md) (productos y categorías).

## Escenarios de Usuario y Pruebas *(obligatorio)*

### User Story 4 - Búsqueda y filtrado rápido para caja y almacén (Prioridad: P2)

Como **Operario/Cajero**, quiero buscar productos por SKU, código de barras, nombre o categoría con resultados inmediatos, para no frenar la atención al cliente.

**Por qué esta prioridad**: La velocidad de búsqueda impacta directamente la experiencia de caja y la productividad del almacén.

**Prueba independiente**: Se prueba ingresando búsquedas parciales y exactas y midiendo tiempo de resultado; no requiere el módulo de depósitos completo.

**Escenarios de aceptación**:

1. **Dado** el catálogo con productos, **Cuando** el Operario escribe un SKU completo, **Entonces** se muestra el producto exacto (o lista mínima).
2. **Dado** un producto con código de barras, **Cuando** se escanea o pega el código completo, **Entonces** se localiza el producto asociado.
3. **Dado** el Operario escribe un fragmento de nombre, **Cuando** hay coincidencias, **Entonces** se listan ordenadas de forma predecible (por relevancia o nombre).
4. **Dado** un Administrador, **Cuando** filtra por categoría, **Entonces** ve solo productos de esa categoría; el Operario en caja puede filtrar o buscar sin necesitar la pantalla completa de administración del catálogo.

> El escenario 6 de US3 en la spec original ("el Operario busca por SKU o código completo y el
> resultado aparece en ≤1 s") se ejecuta en esta spec.

### Casos Borde

- **Búsqueda sin resultados / término muy corto**: la búsqueda por nombre exige un mínimo de caracteres (por defecto 2) para no saturar; SKU y código de barras admiten valor exacto completo desde el primer carácter.

## Requisitos *(obligatorio)*

### Requisitos Funcionales

- **FR-011**: El sistema DEBE buscar y filtrar productos por SKU, código de barras, nombre y categoría, optimizado para caja y almacén (tiempos ver SC-002).
- **FR-013**: Los productos inactivos NO DEBEN aparecer en búsquedas de venta/carga de movimientos nuevos, pero SÍ deben conservarse para reportes históricos.

> FR-003 (RBAC) es de 001 y FR-009/FR-010/FR-012/FR-014 de 003.

### Entidades Clave

- **Producto** (propiedad de 003): artículo comercializable; esta spec lo consume para buscar y
  filtrar.
- **Categoría** (propiedad de 003): agrupación usada como filtro.

## Criterios de Éxito

- **SC-002**: El 95% de las búsquedas de producto por SKU o código de barras completo en interfaz de caja muestran el resultado en 1 segundo o menos.

## Supuestos

- **Búsqueda**: se resuelve en PostgreSQL con `pg_trgm` + índice GIN sobre el nombre y consultas
  exactas para SKU/código; la migración `0005` está reservada para esta spec en el ledger.
- **Inactivos**: la búsqueda de caja excluye `activo = FALSE` por defecto; la gestión puede
  incluirlos con `incluir_inactivos=true`.
- **Idioma de UI**: español (es-AR por defecto).

## Contrato preliminar

| Method | Path | Auth | Query | Éxito |
|---|---|---|---|---|
| GET | `/api/productos/buscar` | sesión | `?q=&campo=sku\|codigo\|nombre\|categoria&incluir_inactivos=true\|false` | 200 `{ items: [...] }` orden estable |

- `q` mínimo 2 caracteres si `campo=nombre`; 422 en caso contrario.
- SKU y código de barras admiten el valor exacto desde el primer carácter.
- `incluir_inactivos=true` solo para familia GESTION.

El contrato definitivo, el plan y las tareas se generan al arrancar la spec.

## Fuera de alcance

- CRUD del catálogo → 003.
- Movimientos de stock y ventas → módulos posteriores.
- La verificación transversal de RBAC → 006.
