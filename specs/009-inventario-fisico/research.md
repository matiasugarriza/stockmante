# Research: Inventario físico por conteo (009)

## Decisiones

### 1. Tablas `conteos` + `conteo_lineas`

**Decisión**: cabecera `conteos` (estado, depósito, autor, fechas) y detalle `conteo_lineas`
(producto, ubicación, cantidad_sistema, cantidad_contada, diferencia).

**Rationale**: permite consultar el conteo como unidad con sus líneas y diferencias, y auditar la
operación. La combinación (conteo, producto, ubicación) es única (FR-034).

**Alternativas descartadas**:
- *Solo `movimiento_lineas` con un flag*: no captura el estado del conteo ni la cantidad contada.

### 2. Servicio dedicado vs. reutilizar `movimientos::crear`

**Decisión**: servicio dedicado `services::conteos.rs`.

**Rationale**: el cierre necesita N líneas con diff y un único movimiento `AJUSTE`; reutilizar
`crear` por línea generaría N movimientos y rompería la atomicidad.

### 3. Recálculo de `cantidad_sistema` al cerrar

**Decisión**: al cerrar, leer el stock vigente por línea y recalcular la diferencia.

**Rationale**: evita ajustar contra un snapshot viejo si hubo movimientos durante el conteo.

### 4. Idempotencia del cierre

**Decisión**: el primer cierre muta; un reintento sobre un conteo `CERRADO` devuelve 409
`conteo_cerrado` y sobre uno `CANCELADO` devuelve 409 `conteo_cancelado`, sin re-aplicar el ajuste.

**Rationale**: evita duplicar el AJUSTE; el efecto es idempotente aunque la respuesta no sea 200.

### 5. Concurrencia

**Decisión**: bloquear la fila del conteo con `FOR UPDATE` en cierre y cancelación.

**Rationale**: evita doble cierre/cancelación concurrente.

### 6. Migración

**Decisión**: `migrations/0011_inventario_fisico.sql` (raíz del repo) crea `conteos` y
`conteo_lineas`.

**Rationale**: reserva de migración anunciada en `specs/README.md`; las migraciones del proyecto
viven en `/migrations`, no en `backend/migrations/` (corrige la ruta que figuraba en el plan
anterior).

### 7. Códigos de error específicos (aclaración 1)

**Decisión**: rechazo de cierre por líneas sin contar → **422 `lineas_sin_contar`**; alta sobre
depósito inactivo → **422 `deposito_inactivo`**.

**Rationale**: son los códigos que la tabla de errores y el contrato ya prometían; permiten al
frontend distinguir "faltó contar" de "depósito mal elegido" y cumplen el Principio III (salidas
programáticas estables). Requiere dos variantes nuevas en `AppError`.

**Alternativas descartadas**:
- *Un único 422 `validacion`*: obliga a parsear el mensaje en el cliente y contradice el contrato.
- *409 para líneas sin contar*: el problema es de datos incompletos, no de conflicto de estado.

### 8. Base de cálculo de `diferencia` (aclaración 3)

**Decisión**: al cargar líneas, `diferencia = cantidad_contada − cantidad_sistema` usando el
**snapshot de apertura**; el recálculo contra el stock vigente ocurre solo en el cierre.

**Rationale**: la diferencia que ve el gerente debe ser estable respecto de lo que se le mostró al
abrir; el cierre ya define el recálculo oficial (FR-035) y es el único punto que muta stock.

**Alternativas descartadas**:
- *Recalcular contra stock vigente en cada `PATCH`*: la diferencia cambiaría sin que el usuario
  toque la línea, y obligaría a reescribir `cantidad_sistema` en cada carga.

### 9. Alta sobre depósito sin stock (aclaración 5)

**Decisión**: **422 `validacion`** con mensaje específico, sin crear el conteo.

**Rationale**: un conteo sin líneas se cerraría de inmediato como `CERRADO` sin haber contado
nada; el mensaje accionable permite detectar que se eligió el depósito equivocado.

**Alternativas descartadas**:
- *201 con cero líneas*: genera registros inútiles y un cierre trivial que parece exitoso.
