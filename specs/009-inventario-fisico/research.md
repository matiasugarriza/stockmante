# Research: Inventario físico por conteo (009)

## Decisiones

### 1. Tablas `conteos` + `conteo_lineas`

**Decisión**: cabecera `conteos` (estado, depósito, autor, fechas) y detalle `conteo_lineas`
(producto, ubicación, cantidad_sistema, cantidad_contada, diferencia).

**Rationale**: permite consultar el conteo como unidad con sus líneas y diferencias, y auditar
la operación. La combinación (conteo, producto, ubicación) es única (FR-034).

**Alternativas descartadas**:
- *Solo `movimiento_lineas` con un flag*: no captura el estado del conteo ni la cantidad contada.

### 2. Servicio dedicado vs. reutilizar `movimientos::crear`

**Decisión**: servicio dedicado `services::conteos.rs`.

**Rationale**: el cierre necesita N líneas con diff y un único movimiento `AJUSTE`; reutilizar
`crear` por línea generaría N movimientos y rompería la atomicidad.

### 3. Recálculo de `cantidad_sistema` al cerrar

**Decisión**: al cerrar, leer el stock vigente por línea y recalcular la diferencia.

**Rationale**: evita ajustar contra un snapshot viejo si hubo movimientos durante el conteo
(supuesto de la spec).

### 4. Idempotencia del cierre

**Decisión**: el primer cierre muta; un reintento devuelve 409 `conteo_cerrado` sin re-aplicar
el ajuste.

**Rationale**: evita duplicar el AJUSTE; el efecto es idempotente aunque la respuesta no sea 200.

### 5. Concurrencia

**Decisión**: bloquear la fila del conteo con `FOR UPDATE` en cierre y cancelación.

**Rationale**: evita doble cierre/cancelación concurrente.

### 6. Migración

**Decisión**: `0011_inventario_fisico.sql` crea `conteos` y `conteo_lineas`.

**Rationale**: reserva de migración anunciada en `specs/README.md`.