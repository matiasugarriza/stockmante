# Especificación de Funcionalidad: Depósitos y ubicaciones jerárquicas

**Spec**: `005-depositos-ubicaciones` | **Caso de uso**: US5 | **Prioridad**: P3

**Creado**: 2026-10-25 | **Estado**: Especificado (sin `plan.md` ni `tasks.md`; se generan al arrancar)

**Origen**: dividida de la spec original del Módulo 1 (`001-user-auth-catalog`, 2026-10-12).
Los textos de escenarios, requisitos y criterios se conservan **verbatim**.

**Entrada**: US5 de la especificación original: "Como Administrador, quiero registrar depósitos y
ubicaciones anidadas (por ejemplo Depósito → Estantería → Nivel → Pasillo) para saber dónde está
físicamente cada unidad de stock."

**Depende de**: [001-autenticacion-sesion](../001-autenticacion-sesion/spec.md) (auth y bootstrap
del depósito "Principal") y, para la asignación producto–ubicación,
[003-catalogo-productos](../003-catalogo-productos/spec.md).

## Escenarios de Usuario y Pruebas *(obligatorio)*

### User Story 5 - Depósitos y ubicaciones jerárquicas (Prioridad: P3)

Como **Administrador**, quiero registrar depósitos y ubicaciones anidadas (por ejemplo Depósito → Estantería → Nivel → Pasillo) para saber dónde está físicamente cada unidad de stock.

**Por qué esta prioridad**: La trazabilidad espacial es necesaria para el módulo de movimientos y transferencias, pero el sistema puede operar un tiempo con un depósito por defecto.

**Prueba independiente**: Se prueba creando un depósito, agregando ubicaciones jerárquicas y validando que no se puedan crear ubicaciones sin depósito.

**Escenarios de aceptación**:

1. **Dado** un Administrador autenticado, **Cuando** crea un depósito con nombre e identificador/dirección, **Entonces** el depósito queda disponible.
2. **Dado** un depósito existente, **Cuando** se crea una ubicación hija (estantería, nivel, etc.) asociada a ese depósito, **Entonces** la ubicación queda registrada bajo el depósito.
3. **Dado** un intento de crear una ubicación sin depósito padre o con un depósito inexistente, **Cuando** se envía, **Entonces** se rechaza (sin ubicaciones huérfanas).
4. **Dado** un depósito con ubicaciones y productos asignados, **Cuando** se intenta eliminar el depósito, **Entonces** se impide el borrado físico mientras haya dependencias; se ofrece baja lógica si aplica.
5. **Dado** un producto, **Cuando** el Administrador le asigna ubicaciones iniciales permitidas, **Entonces** el producto solo puede recibir stock en esas ubicaciones (regla que consumen los módulos de stock; aquí se define el registro de la asignación).

### Casos Borde

- **Ubicación huérfana**: crear/modificar una ubicación sin depósito válido → rechazo; mover una ubicación a un depósito distinto solo si no tiene hijos o stock asociado (regla por defecto: se prohíbe si hay dependencias).

## Requisitos *(obligatorio)*

### Requisitos Funcionales

- **FR-015**: El sistema DEBE permitir CRUD de depósitos con nombre e identificador o dirección.
- **FR-016**: El sistema DEBE permitir CRUD de ubicaciones jerárquicas bajo un depósito (ej. Depósito → Estantería → Nivel → Pasillo), impidiendo ubicaciones huérfanas (sin depósito válido).
- **FR-017**: El sistema DEBE registrar la asignación de ubicaciones iniciales permitidas por producto; las ubicaciones de un producto DEBEN pertenecer a un depósito existente.
- **FR-018**: No DEBE permitirse el borrado físico de un depósito con ubicaciones, productos o movimientos asociados.

### Entidades Clave

- **Depósito**: recinto físico o lógico de stock; nombre e identificador/dirección.
- **Ubicación**: nodo jerárquico dentro de un depósito (pasillo, estantería, nivel, etc.).
- **Asignación Producto–Ubicación**: registro de ubicaciones permitidas para un producto.

> Las tablas `depositos`, `ubicaciones` y `producto_ubicacion_permitida` existen desde la fase
> foundational de 001 (migración `0003`); **la propiedad del esquema es de esta spec**. Si al
> arrancar hace falta una migración nueva, se toma el siguiente número libre del ledger
> (`0005` es de 004 y `0006` de 006).

## Criterios de Éxito

- **SC-005**: Cero ubicaciones existen sin depósito asociado (integridad referencial verificable).

## Supuestos

- **Depósito por defecto**: es aceptable crear un depósito inicial "Principal" en la puesta en
  marcha para no bloquear operación mientras se cargan los depósitos reales. El bootstrap de 001
  ya lo crea.
- **Jerarquía**: profundidad libre dentro de un depósito; el tipo de ubicación es un enum simple
  (`PASILLO`, `ESTANTERIA`, `NIVEL`, `POSICION`).
- **Stock**: los movimientos de stock son un módulo posterior; acá solo se registra la
  asignación que esos módulos consumirán.
- **Idioma de UI**: español (es-AR por defecto).

## Contrato preliminar

| Method | Path | Body | Éxito | Errores |
|---|---|---|---|---|
| GET/POST | `/api/depositos` | POST: `{ nombre, identificador, direccion? }` | 200/201 | 409 nombre/identificador únicos |
| PUT/DELETE | `/api/depositos/{id}` | PUT body; DELETE solo sin dependencias | 200/204 | 409 `deposito_con_dependencias` |
| GET | `/api/depositos/{id}/ubicaciones` | — | 200 árbol o lista plana con `padre_id` | 404 |
| POST | `/api/ubicaciones` | `{ deposito_id, padre_id?, nombre, tipo }` | 201 | 422 `ubicacion_huerfana` si `deposito_id` inválido o ≠ padre |
| PUT/DELETE | `/api/ubicaciones/{id}` | DELETE solo sin hijos/asignaciones | 200/204 | 409 |
| PUT | `/api/productos/{id}/ubicaciones` | `{ ubicacion_ids: number[] }` reemplaza el conjunto | 204 | 404 ubicación |

Todas las rutas con `RequiereGestion`.

El contrato definitivo, el plan y las tareas se generan al arrancar la spec.

## Fuera de alcance

- Movimientos y transferencias de stock.
- Búsqueda de productos → 004.
- Verificación transversal de RBAC → 006.
