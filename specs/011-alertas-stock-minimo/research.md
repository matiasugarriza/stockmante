# Research: Alertas de stock mínimo (011)

Decisiones que resolvieron los `NEEDS CLARIFICATION` del contexto técnico. Las cinco
preguntas de negocio quedaron cerradas en `## Clarifications` de `spec.md` (sesión 2026-11-07);
acá se registra cómo se implementan y qué alternativas se descartaron.

## R1 — Superficie de consulta: endpoint propio, no el filtro de 010

- **Decisión**: `GET /api/alertas` (solo `GESTION`) devuelve `producto_id`, `nombre`, `sku`,
  `stock_actual`, `stock_minimo` y `faltante`. `GET /api/stock?bajo_minimo=true` no cambia.
- **Rationale**: el escenario 11.7 exige 403 para `OPERACION`, y `GET /api/stock` es legible por
  cualquier sesión (`handlers/stock.rs` solo restringe escrituras). Agregar `faltante` al contrato
  de 010 rompería una API ya cerrada. El costo son dos queries parecidas en `services/alertas.rs`,
  no una abstracción nueva.
- **Alternativas consideradas**: reusar `/api/stock` + agregar `faltante` (descartada: cambia 010 y
  contradice 11.7); guard `GESTION` sobre el parámetro `bajo_minimo` (descartada: rompe la consulta
  de stock del operario).

## R2 — `faltante` se calcula en SQL, no en Rust

- **Decisión**: `(p.stock_minimo - p.stock_actual)::bigint AS faltante`.
- **Rationale**: `productos.stock_actual` y `productos.stock_minimo` son `INT` (verificado en
  `0002` y `0009`); el cast a `bigint` mantiene el JSON coherente con `stock_actual: i64` que ya
  usa `StockItem`. Calcularlo en Rust obligaría a un segundo struct y a repetir la regla en el
  frontend.
- **Alternativas consideradas**: campo derivado en `StockItem` (descartada: toca 010); calcular en
  el componente Svelte (descartada: duplica la regla de negocio en el cliente).

## R3 — Reclamo de filas sin doble envío bajo ciclos concurrentes

- **Decisión**: primero `INSERT ... ON CONFLICT (producto_id, usuario_id, fecha) DO NOTHING` para
  cada par (producto, usuario bajo mínimo), después
  `SELECT ... WHERE fecha = $1 AND estado <> 'enviada' FOR UPDATE SKIP LOCKED`, envío y `UPDATE` a
  `enviada`/`fallida` dentro de la misma transacción.
- **Rationale**: el ciclo programado y el `POST` manual pueden solaparse. `INSERT ... DO NOTHING`
  garantiza una sola fila por par y día (SC-019); `FOR UPDATE SKIP LOCKED` evita que dos ciclos
  tomen las mismas filas pendientes. Sin esto, dos envíos simultáneos duplican el correo, que es
  exactamente lo que SC-019 prohíbe.
- **Alternativas consideradas**: advisory lock global del ciclo (descartada: serializa todo y
  oculta el problema en vez de resolverlo); `ON CONFLICT DO UPDATE ... RETURNING` como reclamo
  (descartada: no distingue "reclamo" de "actualización" y reenvía lo ya enviado).

## R4 — Día local con `FixedOffset`, sin dependencia nueva

- **Decisión**: `fecha` = `reloj.ahora().with_timezone(&FixedOffset::west_opt(3 * 3600))`
  `.date_naive()`.
- **Rationale**: Argentina no aplica horario de verano desde 2009, así que el offset fijo −03:00 es
  exacto. `chrono` ya es dependencia; `chrono-tz` sería una dependencia nueva sin variación real
  (Principio I). El reloj entra por `AppEstado.reloj`, así que el test fija la hora con
  `RelojFalso`.
- **Alternativas consideradas**: `chrono-tz` (descartada: dependencia por un solo offset sin DST);
  fecha UTC (descartada en la clarificación: desalinea el día de negocio).

## R5 — Scheduler en `main.rs`, no en el router

- **Decisión**: `AppConfig` gana `alertas_intervalo_minutos: u64` (default 60, `0` desactiva). Si
  es `> 0`, `main.rs` hace `tokio::spawn` de un bucle con `tokio::time::interval` y
  `MissedTickBehavior::Skip`; `construir_router` no arranca nada.
- **Rationale**: si el timer viviera en `construir_router`, cada test HTTP levantaría un ciclo
  periódico contra la BD y la suite se volvería no determinista (Principio IV). El bucle llama a la
  misma función que el `POST` manual, sin lógica duplicada.
- **Alternativas consideradas**: proceso/cron externo (descartada: otra pieza operativa para un
  intervalo simple); `spawn` dentro del handler o del router (descartada: fuga de tareas en tests).

## R6 — Extender `EnvioCorreo` con un tercer método

- **Decisión**: agregar
  `async fn enviar_alerta_stock(&self, destino: &str, nombre: &str, cuerpo: &str) -> Result<(), String>`
  al trait, con implementación en `CorreoDev`, `CorreoSmtp` y el `CorreoQueFalla` de
  `tests/unit/test_usuarios_alta.rs`.
- **Rationale**: el cuerpo lo arma `services/alertas.rs` (lógica pura, testeable); el trait solo
  transporta. Sin implementación por defecto: un default silencioso haría que un transporte nuevo
  "pase" los tests sin enviar nada.
- **Alternativas consideradas**: método con default `Ok(())` (descartada: oculta transportes sin
  implementar); un segundo trait `EnvioAlertas` (descartada: dos transportes para el mismo SMTP).

## R7 — Frontend sin polling, badge solo para `GESTION`

- **Decisión**: `+layout.svelte` consulta `GET /api/alertas` únicamente cuando
  `familia_rol === 'GESTION'`, al montar y al cambiar la ruta. `admin/stock` muestra la sección
  "Bajo mínimo" con mínimo, stock y faltante.
- **Rationale**: `GET /api/alertas` responde 403 a `OPERACION`; llamarlo siempre generaría un error
  de red en cada carga de sesión de operario. SC-019 mide la frescura del endpoint, no la del
  badge, así que no hace falta un temporizador (clarificación 2026-11-07).
- **Alternativas consideradas**: polling cada 5/60 minutos (descartada en la clarificación:
  complejidad sin criterio que la exija).

## R8 — Número de migración: `0015`

- **Decisión**: `migrations/0015_alertas_stock.sql`.
- **Rationale**: `migrations/` tiene hasta `0014_ventas.sql`; `0012` ya es `registro_sso`. La
  reserva previa de 011 (`0012`) y la de 015 (`0015`) estaban desactualizadas; `specs/README.md`
  dice que las reservas son orientativas hasta que la spec genera su `plan.md`, así que 011 toma
  `0015` y 015 se corre a `0016`.
- **Alternativas consideradas**: 011 en `0016` dejando `0015` para 015 (descartada: obligaría a
  aplicar `0015` después de `0016` cuando 015 se implemente).
