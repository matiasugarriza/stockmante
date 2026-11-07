# Progreso de implementación — Alertas de stock mínimo (011)

**Spec**: [spec.md](./spec.md) | **Plan**: [plan.md](./plan.md) | **Actualizado**: 2026-11-07 |
**Estado**: implementado (T001–T025, T027); T026 (quickstart manual) y T029 (re-análisis) pendientes.

## Esquema (T001, T002)

Verificado con `psql`: `productos.stock_actual/stock_minimo INT NOT NULL`, `activo BOOLEAN`;
`usuarios.email/familia_rol/activo`; `notificaciones_stock` no existía. `0015_alertas_stock.sql`
aplicada con `cargo sqlx migrate run` en `stockmante` y `stockmante_test`.

**Regularización previa del historial de migraciones** (necesaria para poder aplicar `0015`):
`0013` y `0014` estaban aplicadas a mano (tablas presentes, sin fila en `_sqlx_migrations`) y las
versiones 1, 2, 3, 5, 11 y 12 tenían checksums distintos a los archivos actuales. Se aplicaron todas
las migraciones a una base temporal (`stockmante_verif`), se comparó el esquema completo (columnas,
constraints e índices) con `stockmante` y `stockmante_test` — **idéntico** — y entonces se copiaron
las filas 13 y 14 con sus checksums reales y se alinearon los checksums desalineados (SHA-384 del
archivo). `cargo sqlx migrate info` queda sin diferencias en ambas bases; la base temporal se borró.

## Evidencia RED (2026-11-07)

`test_alertas_validacion` y `test_alertas_http` no compilaban (`services::alertas`,
`AppConfig::parsear_intervalo_alertas` y `EnvioCorreo::enviar_alerta_stock` no existían);
`api/alertas.test.ts` falló por módulo inexistente.

## Decisión de diseño: `AlcanceCiclo`

La BD de test compartida tiene ~1.000 gestores activos y ~540 productos bajo mínimo (≈560.000 filas
por ciclo). `services::alertas::ciclo` recibe un `AlcanceCiclo` (productos y usuarios opcionales):
producción y el `POST` usan `AlcanceCiclo::default()` (todo); los tests del ciclo lo acotan a sus
propios datos y usan un reloj falso en 2030 para no chocar con la deduplicación diaria. El test HTTP
del `POST` completo (`enviar_por_http`) queda `#[ignore]`: corrido a mano el 2026-11-07, **verde en
33 s** e idempotente en la segunda llamada (`enviados: 0`).

`T003` (casos de `ALERTAS_INTERVALO_MINUTOS`) quedó en `test_alertas_validacion.rs` en lugar de
`test_config.rs`: el parseo es una función pura `AppConfig::parsear_intervalo_alertas`.

## Resultado

`test_alertas_http` 4/4 (+1 ignorado), `test_alertas_validacion` 4/4, `test_config` 5/5,
`test_usuarios_alta` 5/5 (fake ampliado con `enviar_alerta_stock`); frontend `api/alertas.test.ts`
5/5, `svelte-check` 0/0. Badge del layout desde `/api/alertas` solo para GESTION, refrescado por
ruta; sección "Bajo mínimo" con botón de envío en `/admin/stock`.
