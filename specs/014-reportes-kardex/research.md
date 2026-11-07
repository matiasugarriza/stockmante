# Research: Reportes, kardex y valorización (014)

Decisiones que cierran los hallazgos del análisis del 2026-11-07 (C1–C4, I1–I4, A1–A2, F1,
U1–U4, D1–D2, L1–L2). Las cuatro preguntas de negocio quedaron cerradas en `## Clarifications` de
`spec.md` (sesión 2026-11-07); acá se registra cómo se implementan y qué se descartó.

Punto de partida, el código real: `backend/src/services/reportes.rs` y
`backend/src/handlers/reportes.rs` existen y están montados en `/api/reportes` (`app.rs`) y en
OpenAPI (`docs.rs`); **no hay** tests de reportes, ni `$lib/api/reportes.ts`, ni pantalla
`/admin/reportes`, ni sección en `docs/api.md`, aunque el `tasks.md` anterior las marcaba como
hechas. Esquema contrastado con `migrations/0009`, `0010`, `0013` y `0014` (no con la BD real:
lo hace T001).

## R1 — Todos los reportes exigen `GESTION` (I1, FR-059, SC-031)

- **Decisión**: los cuatro handlers usan el extractor `RequiereGestion` (ya existente en
  `auth/extractors.rs`) en lugar de `AuthUsuario` + chequeo manual. Se elimina el
  `if u.familia_rol != "GESTION"` de `stock_valorizado`.
- **Rationale**: un solo mecanismo, el mismo que 012/013; el 403 se produce antes de parsear
  parámetros, así que el CSV queda cubierto sin código extra.
- **Alternativas**: kardex abierto a `OPERACION` (descartada en la clarificación).

## R2 — Fechas: día local UTC-03:00, inclusivo, sin reloj en el servicio (C4, I4, U2)

- **Decisión**: un módulo puro `services::reportes::rango` con
  `parsear_dia(texto, campo) -> Result<Option<NaiveDate>, AppError>` (acepta `YYYY-MM-DD`; también
  RFC 3339, del que toma la fecha en UTC-03:00) y
  `limites_utc(desde: Option<NaiveDate>, hasta: Option<NaiveDate>) -> Result<(Option<DateTime<Utc>>, Option<DateTime<Utc>>), AppError>`
  que devuelve `[desde 00:00 -03:00, (hasta + 1 día) 00:00 -03:00)` como límites **semiabiertos**
  (`>= inicio AND < fin`). Valida `desde <= hasta` con el mensaje de 14.6.
  `exigir_rango` agrega el 422 de "Indicá las fechas desde y hasta…" para rotación y ventas.
  El offset es `FixedOffset::west_opt(3 * 3600)`, igual que 011 (R de 011-alertas).
- **Rationale**: sin `Utc::now()` el servicio es determinista (P-IV); el límite semiabierto evita
  perder el último día (bug actual: `hasta=2026-10-31` corta a las 00:00 UTC del 31).
- **Alternativas**: default de 30 días con `estado.reloj` (descartada en la clarificación); zona
  configurable (fuera de alcance).

## R3 — Rotación: una consulta agregada, salidas sin transferencias (A2, U4, FR-058)

- **Decisión**: reemplazar el bucle N+1 (3 consultas por producto) por una sola consulta con
  `SUM(...) FILTER (WHERE ...)` agrupada por `ml.producto_id`:
  - `stock_inicial` = Σ `cantidad_delta` con `m.creado_en < inicio`.
  - `neto` = Σ `cantidad_delta` en `[inicio, fin)`; `stock_final = stock_inicial + neto`.
  - `unidades_salidas` = Σ `-cantidad_delta` en `[inicio, fin)` con `m.tipo = 'SALIDA' AND
    m.motivo <> 'TRANSFERENCIA'`.
  Se listan productos con `unidades_salidas > 0 OR stock_inicial > 0 OR stock_final > 0`, activos
  o no. El cálculo `promedio = (inicial + final) / 2`, `indice = salidas / promedio` (0 si
  `promedio = 0`) vive en una función pura `calcular_rotacion` con test unitario.
- **Rationale**: los valores de `movimientos.tipo`/`motivo` existen tal cual en los `CHECK` de
  0009; las transferencias (0010) generan `SALIDA`+`ENTRADA` con motivo `TRANSFERENCIA`.
- **Alternativas**: incluir mermas o solo `VENTA` (descartadas en la clarificación).

## R4 — Stock valorizado desde `stock_por_deposito` (FR-057)

- **Decisión**: `SUM(s.cantidad)` de `stock_por_deposito` por producto, `JOIN productos` para
  `nombre`, `sku`, `costo`; `HAVING SUM(s.cantidad) > 0`, activos o no; `valor = costo × cantidad`
  en `Decimal`. Orden por nombre.
- **Rationale**: la constitución (P-V) define `stock_por_deposito` como fuente de verdad y
  `stock_actual` como cache; un producto dado de baja con stock físico sigue teniendo valor.
- **Alternativas**: `productos.stock_actual` de activos (código actual; descartada por P-V y porque
  oculta stock de inactivos).

## R5 — Kardex (I4, FR-056, SC-022)

- **Decisión**: se conserva la forma actual (saldo inicial + líneas + saldo final) con los límites
  de R2: saldo inicial = Σ deltas con `creado_en < inicio` (0 sin `desde`); líneas en
  `[inicio, fin)`; `deposito_id` opcional filtra ambos. El acumulado se calcula en una función
  pura `acumular_saldos(saldo_inicial, &[delta]) -> Vec<i32>` con test unitario.
- **SC-022**: el test de integración arma entradas, salidas, merma, ajuste y una transferencia y
  compara `saldo_final` con `productos.stock_actual` (sin filtros) y con `stock_por_deposito`
  (con `deposito_id`).

## R6 — Ventas (FR-058)

- **Decisión**: solo `estado = 'CONFIRMADA'` en `[inicio, fin)`. Agrupación por día local con
  `to_char(v.creado_en AT TIME ZONE 'UTC' - INTERVAL '3 hours', 'YYYY-MM-DD')`.
  Total general = Σ `ventas.total`; por producto = Σ `venta_lineas.cantidad` y Σ
  `venta_lineas.subtotal` (antes del descuento; documentado en Supuestos).
- **Rationale**: evita `AT TIME ZONE '-03'`, cuyo signo POSIX es invertido en PostgreSQL y
  produciría +03:00; no depende de la `TimeZone` de la sesión.

## R7 — CSV: un módulo, RFC 4180, BOM, cuatro reportes (F1, U1)

- **Decisión**: `services::reportes::csv` con una función pura
  `a_csv(encabezados: &[&str], filas: impl Iterator<Item = Vec<String>>) -> String` que escapa
  campos con `,`, `"`, `\r` o `\n` (comillas dobles, `"` duplicada), separa con `,`, termina líneas
  en `\r\n` y antepone BOM UTF-8 (`\u{FEFF}`) para que Excel lea bien los acentos. Decimales con
  punto. Encabezados en español:
  - kardex: `fecha,tipo,motivo,deposito,cantidad,saldo` (fecha en RFC 3339 con offset -03:00)
  - stock valorizado: `producto,sku,cantidad,costo_unitario,valor`
  - rotación: `producto,unidades_salidas,stock_inicial,stock_final,stock_promedio,indice_rotacion`
  - ventas `agrupar=dia`: `dia,cantidad_ventas,total`; `agrupar=producto`:
    `producto,unidades,total`
  La respuesta agrega `Content-Disposition: attachment; filename="<reporte>_<desde>_<hasta>.csv"`.
- **Rationale**: sin dependencias nuevas (P-I: el crate `csv` no se justifica para cuatro tablas
  planas); la lógica pura se prueba sin HTTP (P-IV).
- **Alternativas**: crate `csv` (dependencia nueva); CSV de ventas con dos secciones (no es CSV
  tabular válido).

## R8 — Parámetros cerrados: `formato` y `agrupar` (U2, P-III)

- **Decisión**: enums con `serde` (`Formato { Json, Csv }`, `Agrupar { Dia, Producto }`,
  `#[serde(rename_all = "lowercase")]`); un valor no admitido se traduce a 422 `validacion` con
  `"El parámetro formato acepta json o csv."` / `"El parámetro agrupar acepta dia o producto."`.
  Si `QueryValida` no produce ese mensaje, se parsean como `String` y se validan en el handler.

## R9 — Sin paginación (I3)

- **Decisión**: ni JSON ni CSV paginan (Supuestos de la spec); se borra la importación sin uso de
  `normalizar_paginacion` y la frase "Paginación estándar" del plan anterior.

## R10 — Frontend: `/admin/reportes` con cuatro componentes (C2, C3)

- **Decisión**: `$lib/schemas/reportes.ts` (Zod), `$lib/api/reportes.ts` (cuatro funciones JSON +
  `urlCsv(reporte, params)` que arma la URL para descargar con `<a href download>`, la cookie de
  sesión viaja igual) y su `reportes.test.ts`. Página `frontend/src/routes/admin/reportes/+page.svelte`
  con selector de reporte y cuatro componentes en `frontend/src/lib/components/reportes/`
  (`Kardex.svelte`, `StockValorizado.svelte`, `Rotacion.svelte`, `Ventas.svelte`). Enlace
  "Reportes" en `+layout.svelte` dentro del bloque `GESTION`. Los errores 422/403 se muestran con el
  mensaje del backend.
- **Rationale**: un componente por reporte (P-II); el enlace `<a>` evita reimplementar la
  descarga de blobs en `client.ts`.

## R11 — Consultas SQLx en tiempo de ejecución (D1)

- **Decisión**: se mantienen `sqlx::query_as(...)` como el resto de los servicios del proyecto;
  la cobertura la dan los tests de integración contra la BD de test. Migrar a macros `query!` es
  una decisión transversal fuera de 014.

## R12 — Tests RED primero y gates (C1, P-IV)

- **Decisión**: unitarios puros (`test_reportes_calculo`: rango, acumulado, rotación, CSV) e
  integración HTTP (`test_reportes_http`: 14.1–14.7, casos borde, SC-022) declarados como
  `[[test]]` en `backend/Cargo.toml`; SC-031 se agrega a `test_rbac_matriz`. Se escriben antes de
  corregir el servicio y se registra que fallan.
