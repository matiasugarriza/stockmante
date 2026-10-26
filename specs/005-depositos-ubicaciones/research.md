# Research — Depósitos y ubicaciones (005)

**Fecha**: 2026-10-26 | **Spec**: [spec.md](./spec.md) | **Plan**: [plan.md](./plan.md)

## Decisiones

### 1. ¿Migración nueva?

**Decisión**: no. `0003_depositos_ubicaciones.sql` ya creó `depositos`, `ubicaciones` y
`producto_ubicacion_permitida` con las restricciones que la spec necesita.

**Por qué**: la spec declara que el esquema es propiedad de 005 pero que las tablas vienen de la
fase foundational de 001. Agregar una migración vacía solo para "marcar propiedad" ensucia el
ledger. Si en el futuro hace falta un cambio real (por ejemplo `ON DELETE CASCADE` parcial), se
toma el siguiente número libre (`0008`, hoy reservado a auditoría de 006).

### 2. Representación de la jerarquía

**Alternativas**:
- Materialized path (`/1/4/9`): lecturas de subárbol baratas, escrituras y movimientos caros.
- Nested sets: lecturas baratas, toda escritura reescribe rangos.
- **Lista de adyacencia (`padre_id`)**: simple, ya modelada en `0003`, suficiente para el Módulo 1.

**Decisión**: lista de adyacencia + CTE recursiva solo para detectar ciclos al mover.

### 3. Detección de ciclos

**Decisión**: al cambiar `padre_id` o `deposito_id`, caminar la ascendencia del nuevo padre dentro
de la misma transacción y rechazar si aparece la ubicación que se mueve (o si el padre es ella
misma).

**Por qué**: PostgreSQL no permite un `CHECK` que consulte otras filas; la CTE es acotada y se
ejecuta solo en `PUT`, no en cada lectura.

### 4. Borrado físico vs baja lógica

**Decisión**: `DELETE` permitido solo sin hijos ni asignaciones; si hay dependencias, 409 con
`deposito_con_dependencias` / `ubicacion_con_dependencias`. `POST /desactivar` y `/reactivar`
cubren la baja lógica.

**Por qué**: FR-018 prohíbe el borrado físico con dependencias; la FK `ON DELETE RESTRICT` de
`0003` ya lo impone en la base, y el servicio devuelve un mensaje accionable antes de que el error
crudo llegue al cliente.

### 5. Cambio de ubicación a otro depósito

**Decisión**: solo si no tiene hijos ni asignaciones. Si se omite `padre_id`, la ubicación pasa a
raíz del depósito destino.

**Por qué**: conservar el padre anterior violaría "el padre pertenece al mismo depósito". Mover el
subárbol completo queda fuera del Módulo 1 (movimientos/transferencias).

### 6. Asignación producto–ubicación

**Decisión**: `PUT` con el conjunto completo. Se deduplican ids y se exige que cada ubicación y su
depósito estén activos. `[]` limpia.

**Por qué**: el contrato preliminar ya define "reemplaza el conjunto"; evita operaciones
incrementales y hace idempotente la pantalla.

### 7. UI del árbol

**Decisión**: una sola página `/admin/depositos` con lista de depósitos, formulario de ubicación y
una lista aplanada con sangría por profundidad; la asignación por producto vive en la misma página.

**Por qué**: no se justifica un componente recursivo ni un router anidado para la profundidad
libre del Módulo 1; el backend ya devuelve una lista plana con `padre_id`.

## Riesgos abiertos

- El `UNIQUE NULLS NOT DISTINCT` requiere PostgreSQL 15+. El proyecto ya lo exige.
- La UI no mueve subárboles completos; para el Módulo 1 se documenta como limitación consciente.
