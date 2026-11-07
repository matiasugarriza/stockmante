# Data Model: Alertas de stock mínimo (011)

**Migración**: `migrations/0015_alertas_stock.sql`

## Verificación de esquema (Principio V)

Contrastado contra `migrations/` antes de escribir DDL (BD de desarrollo accesible por
`DATABASE_URL`; si no, `psql`/MCP de PostgreSQL):

| Objeto | Estado verificado | Fuente |
|---|---|---|
| `productos.id` INT identity | existe | `0002_categorias_productos.sql` |
| `productos.stock_actual` `INT NOT NULL DEFAULT 0 CHECK (>= 0)` | existe | `0009_stock_movimientos.sql` |
| `productos.stock_minimo` `INT NOT NULL DEFAULT 0` | existe | `0002` + `0006` |
| `productos.activo` `BOOLEAN NOT NULL DEFAULT TRUE` | existe | `0002` |
| `usuarios.id` INT identity | existe | `0001_usuarios_sesiones.sql` |
| `usuarios.email`, `usuarios.familia_rol`, `usuarios.activo` | existen | `0001` |
| `notificaciones_stock` | **no existe** | ninguna migración 0001–0014 |

`productos.stock_actual` es `INT`, no `BIGINT`: el API lo expone como `i64` mediante cast y
`faltante` también se castea a `bigint`.

## Entidad `notificaciones_stock`

Registro por destinatario y día. Es la fuente de verdad de la deduplicación (FR-042).

```sql
CREATE TABLE notificaciones_stock (
    id INT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    producto_id INT NOT NULL REFERENCES productos(id) ON DELETE CASCADE,
    usuario_id INT NOT NULL REFERENCES usuarios(id) ON DELETE CASCADE,
    -- día local America/Buenos_Aires (UTC-03:00, sin DST) del ciclo que reclamó la fila
    fecha DATE NOT NULL,
    estado VARCHAR(10) NOT NULL DEFAULT 'pendiente'
        CHECK (estado IN ('pendiente', 'enviada', 'fallida')),
    intentos INT NOT NULL DEFAULT 0 CHECK (intentos >= 0),
    enviada_en TIMESTAMPTZ NULL,
    -- motivo del último fallo; NULL mientras nunca falló
    error TEXT NULL,
    CONSTRAINT uq_notificacion_producto_usuario_fecha
        UNIQUE (producto_id, usuario_id, fecha),
    -- una fila marcada como enviada siempre tiene sello de envío
    CONSTRAINT ck_notificacion_enviada_con_fecha
        CHECK (estado <> 'enviada' OR enviada_en IS NOT NULL)
);

CREATE INDEX idx_notificaciones_pendientes
    ON notificaciones_stock (fecha, estado)
    WHERE estado <> 'enviada';
```

**Por qué `ON DELETE CASCADE`**: si se borra físicamente un producto o un usuario (operaciones
restringidas por otras tablas pero posibles en catálogo limpio), un histórico de alertas no debe
bloquear la operación; la alerta deja de tener sentido sin su par.

**Por qué el índice parcial**: el ciclo solo busca filas reintentables (`estado <> 'enviada'`) del
día; el índice se mantiene chico porque las enviadas quedan fuera.

## Estado de una notificación

```
                 INSERT (ON CONFLICT DO NOTHING)
                              │
                              ▼
                        ┌───────────┐
              ┌────────▶│ pendiente │
              │         └─────┬─────┘
   fallo SMTP │               │ envío OK
              │               ▼
        ┌─────┴─────┐   ┌───────────┐
        │  fallida  │   │  enviada  │  (terminal: nunca se reenvía)
        └─────┬─────┘   └───────────┘
              │ reintento en el ciclo siguiente
              └──────────────┘
```

- `pendiente` → `enviada`: envío exitoso; setea `enviada_en`.
- `pendiente`/`fallida` → `fallida`: fallo de transporte; incrementa `intentos`, guarda `error`.
- `enviada` es terminal aun si el producto sigue bajo mínimo: ese día ya se avisó (FR-042).
- Al día siguiente la fecha cambia y se crea una fila nueva: el aviso se repite mientras el
  producto siga bajo el mínimo.

## Objetos de transferencia (no persistidos)

| Objeto | Campos | Uso |
|---|---|---|
| `AlertaItem` | `producto_id: i32`, `nombre: String`, `sku: String`, `stock_actual: i64`, `stock_minimo: i32`, `faltante: i64` | fila de `GET /api/alertas` |
| `ListaAlertas` | `items: Vec<AlertaItem>`, `total: i64`, `pagina: i64`, `por_pagina: i64` | respuesta de `GET /api/alertas` |
| `NotificacionPendiente` | `id: i32`, `usuario_id: i32`, `email: String`, `nombre: String`, `producto_id: i32`, `sku: String`, `faltante: i64` | filas reclamadas por el ciclo |
| `RespuestaCiclo` | `fecha: NaiveDate`, `productos: i64`, `destinatarios: i64`, `enviados: i64`, `fallidos: i64` | respuesta de `POST /api/alertas/enviar` |

## Reglas de validación derivadas de la spec

| Regla | Origen | Dónde se aplica |
|---|---|---|
| Producto activo, `stock_minimo > 0`, `stock_actual < stock_minimo` | FR-041 | `WHERE` de `services::alertas::detectar` |
| `faltante = stock_minimo - stock_actual` (> 0 por definición) | FR-041 | expresión SQL del listado |
| Una fila por `(producto_id, usuario_id, fecha)` | FR-042 | `UNIQUE` + `ON CONFLICT DO NOTHING` |
| `fecha` = día local UTC−03:00 del reloj inyectado | Clarificación 2026-11-07 | `services::alertas::fecha_local` |
| Solo filas `estado <> 'enviada'` se reintentan | FR-043 | índice parcial + `WHERE` del reclamo |
| Una sola consulta de destinatarios: `usuarios.familia_rol = 'GESTION' AND activo` | FR-042 | `services::alertas::usuarios_gestion` |

## Sin cambios en el esquema existente

`productos`, `usuarios` y `stock_por_deposito` no se alteran. `productos.stock_actual` sigue siendo
el cache denormalizado de 007; esta feature solo lo lee.
