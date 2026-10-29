# Research: Transferencias entre depósitos (008)

## Decisiones

### 1. Tabla `transferencias` vs. reconstrucción desde `movimientos.transferencia_id`

**Decisión**: tabla `transferencias` como cabecera de la operación.

**Rationale**: la spec (FR-032) pide `GET /api/transferencias/{id}` con los dos movimientos
asociados. Sin cabecera propia, habría que hacer self-join sobre `movimientos.transferencia_id`
y filtrar por `motivo = 'TRANSFERENCIA'`, lo cual es frágil y no escala a futuras columnas
(especialidad, usuario_aprobador, etc.). La tabla `transferencias` es un registro explícito
de la operación y facilita la auditoría (`tabla_ref = 'transferencias'`).

**Alternativas descartadas**:
- *Solo `movimientos.transferencia_id` sin tabla*: imposible consultar la transferencia como
  unidad sin self-join complejo.
- *Tabla `transferencias` con `lineas` propias*: duplicaría `movimiento_lineas`; los dos
  movimientos generados ya tienen sus líneas.

### 2. Servicio dedicado vs. reutilizar `movimientos::crear`

**Decisión**: servicio dedicado `services::transferencias.rs`.

**Rationale**: `movimientos::crear` crea un solo movimiento por llamada con un solo depósito.
Una transferencia necesita dos movimientos en depósitos distintos dentro de la misma
transacción, con bloqueo de ambas filas de stock y recálculo del cache al final. Reutilizar
`crear` dos veces exigiría dos transacciones separadas (rompería atomicidad) o refactorizar
`crear` para aceptar transacciones externas (complejidad innecesaria, Principio I).

**Alternativas descartadas**:
- *Llamar `crear` dos veces*: no es atómico; si la segunda falla, la primera ya se committeó.
- *Refactorizar `crear` para aceptar `&mut Transaction`*: agregaría una capa de abstracción
  que solo beneficia a una llamada; YAGNI.

### 3. Ubicación origen y destino

**Decisión**: ambos opcionales; si se omite, el stock se imputa al depósito.

**Rationale**: consistente con 007 donde `ubicacion_id` es opcional. Las reglas de
`producto_ubicacion_permitida` (005) aplican igual. La spec no exige ubicaciones obligatorias.

**Alternativas descartadas**:
- *Ubicaciones obligatorias*: excedería la spec y rompería la simetría con 007.

### 4. Recálculo de `productos.stock_actual`

**Decisión**: recalcular como `SUM(stock_por_deposito.cantidad)` después de aplicar ambas
direcciones, igual que 007.

**Rationale**: como la suma global es constante (se quita en un depósito y se suma en otro),
el cache no debería cambiar. Pero recalcular (en vez de confiar en eso) es gratis y elimina
cualquier posibilidad de deriva.

### 5. Orden de bloqueo

**Decisión**: bloquear depósitos por id ascendente, luego productos por id ascendente, luego
`stock_por_deposito`.

**Rationale**: mismo patrón que 007; dos transferencias concurrentes que involucran los
mismos depósitos en orden inverso no generan deadlock porque el orden es determinístico.

### 6. Migración

**Decisión**: `0010_transferencias.sql` crea la tabla `transferencias` y agrega
`movimientos.transferencia_id` (nullable, FK a `transferencias`).

**Rationale**: reserva de migración ya anunciada en `specs/README.md`. Las filas existentes
de `movimientos` quedan con `transferencia_id = NULL`.
