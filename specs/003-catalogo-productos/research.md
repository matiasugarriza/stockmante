# Research: Catálogo de productos y categorías

**Date**: 2026-10-26 | **Spec**: [spec.md](./spec.md) | **Plan**: [plan.md](./plan.md)

> R1–R8 (sesión, RBAC, monorepo, correo, reloj) son decisiones de la base compartida y viven en
> [001/research.md](../001-autenticacion-sesion/research.md). Acá se registran solo las
> decisiones propias de US3. R11, R12 y R13 fueron **revisadas** en el replanteo del 2026-10-26;
> el resto se conserva.

## R9 — Representación del dinero

- **Decision**: `rust_decimal::Decimal` con la feature `serde-float` para serializar como número
  JSON, y la feature `rust_decimal` de SQLx para mapear `NUMERIC(12,2)`.
- **Rationale**: el contrato publica `costo` y `precio_venta` como números, no cadenas; `f64`
  perdería precisión y `serde` por defecto serializa `Decimal` como string. `serde-float`
  deserializa con `visit_f64` → `Decimal::from_str(&value.to_string())`: `10.1` llega con escala 1
  y no se convierte en `10.100000000000001`.
- **Alternatives considered**: `f64` (errores de redondeo en precios); `i64` de centavos (obliga a
  convertir en cada capa y rompe el contrato numérico); serializar como string (cambia el contrato).
- **Costo asumido**: una dependencia nueva y una feature de SQLx. Justificado en el plan
  (Principio I admite dependencias con justificación registrada).

## R10 — Unicidad sin pre-chequeo (TOCTOU)

- **Decision**: delegar la unicidad al índice y mapear `23505` por **nombre de constraint**:
  `productos_sku_key` → `SkuDuplicado`; `productos_codigo_barras_key` →
  `CodigoBarrasDuplicado`; `uq_categorias_nombre_normalizado` (y `categorias_nombre_key` como
  respaldo) → `CategoriaDuplicada`.
- **Rationale**: un `SELECT` previo no es atómico y agrega una consulta; el mapeo por constraint es
  exacto y funciona bajo concurrencia.
- **Alternatives considered**: `EXISTS` antes de insertar (carrera); `ON CONFLICT DO NOTHING` (no
  distingue qué constraint falló ni devuelve el motivo al usuario).
- **Nota**: `uq_categorias_nombre_normalizado` es el índice funcional de R16. La constraint
  `categorias_nombre_key` de `0002` se conserva, así que el mapeo debe aceptar ambos nombres: dos
  categorías que difieren solo en mayúsculas violan el índice funcional, y dos idénticas violan
  cualquiera de los dos.

## R11 — Solo baja lógica: no existe borrado físico

- **Decision**: **no se registra ninguna ruta `DELETE`** para productos ni categorías. Axum responde
  `405 Method Not Allowed` con cabecera `Allow` cuando el path existe con otros métodos. La única
  vía de retiro es `POST /{id}/desactivar` (`activo = FALSE`) y la de vuelta
  `POST /{id}/reactivar`.
- **Rationale**: el requerimiento dice "NUNCA" para ambos recursos, y el historial futuro
  (movimientos de stock, comprobantes, reportes) referencia `id` de producto. Un endpoint de
  borrado que siempre falla agrega una rama muerta y confunde a los clientes.
- **Alternatives considered**: (a) `DELETE` que responde 409 permanente — se descarta porque no
  existe condición de éxito y el 409 sugiere un conflicto transitorio; (b) `DELETE` con baja lógica
  interna — mentiría sobre el verbo HTTP y rompería "nunca se elimina"; (c) `DELETE` bloqueado por
  referencias (diseño del WIP) — el requerimiento vigente prohíbe también el borrado de productos
  sin historial y de categorías vacías.
- **Consecuencia**: la variante `AppError::ProductoConReferencias` queda sin uso y se elimina;
  `test_apperror` se ajusta en la misma unidad de trabajo.

## R12 — Código de barras opcional y `NULL`

- **Decision**: normalizar el código vacío o en blanco a `NULL`; la migración
  `0006_catalogo_descripcion_codigo_barras.sql` reemplaza `UNIQUE NULLS NOT DISTINCT
  (codigo_barras)` por `UNIQUE (codigo_barras)` (comportamiento `NULLS DISTINCT` por defecto). En
  `PUT`, la cadena vacía limpia el código; omitir el campo lo conserva.
- **Rationale**: la spec dice que el código es opcional; con `NULLS NOT DISTINCT` solo un producto
  podía quedar sin código. La `0004` del WIP nunca se publicó, así que su contenido se absorbe en la
  `0006`.
- **Alternatives considered**: índice único parcial `WHERE codigo_barras IS NOT NULL` (equivalente
  y más explícito, pero el constraint ya existe con nombre propio y reemplazarlo mantiene el mapeo
  `23505` simple); usar cadena vacía en lugar de `NULL` (rompe la semántica de "sin código").
- **Estado de las bases**: el constraint ya está corregido a mano en `stockmante` y
  `stockmante_test`, sin fila en `_sqlx_migrations`. La migración verifica el nombre real antes de
  reemplazar y deja ambas rutas convergentes.

## R13 — `parsear_activo` compartido con default explícito

- **Decision**: mover `parsear_activo` de `handlers/usuarios.rs` a `handlers/mod.rs` como
  `pub(crate) fn parsear_activo(valor: Option<&str>, por_defecto: Option<bool>)`. Usuarios pasa
  `None`; productos y categorías pasan `Some(true)`.
- **Rationale**: los tres listados aceptan `?activo=true|false|all`, pero con semánticas distintas
  cuando el parámetro falta: usuarios devuelve todas, catálogo devuelve solo activas. Un default
  explícito evita heredar el comportamiento equivocado y deja la diferencia a la vista.
- **Alternatives considered**: duplicar la función por módulo (mensajes y bordes divergentes);
  tipo `QueryActivo` con `Deserialize` (agrega ceremonia y no mejora el 422).
- **Riesgo cubierto**: test de regresión de `GET /api/usuarios` sin filtro → devuelve activos e
  inactivos.

## R14 — Aislamiento de tests con claves únicas

- **Decision**: los tests de integración derivan SKU, código de barras y nombre de categoría de un
  UUID para correr en paralelo sin colisiones; los tests de servicio usan datos únicos y no limpian
  si un `expect_err` falla.
- **Rationale**: `cargo test` ejecuta targets en paralelo contra la misma BD de test; una clave fija
  produce 409 espurios.
- **Alternatives considered**: serializar los tests con un lock global (más lento y frágil); truncar
  tablas entre tests (interfiere con otros targets en paralelo).

## R15 — 405 sin handler propio

- **Decision**: no registrar `DELETE` en `/api/productos/{id}` ni `/api/categorias/{id}`; dejar que
  Axum emita `405` con `Allow`.
- **Rationale**: es la respuesta HTTP estándar para un método no soportado, no requiere código ni
  una variante nueva de `AppError`, y la spec solo exige el status y la ausencia de efectos. El
  cliente real (SPA) nunca invoca `DELETE`.
- **Alternatives considered**: handler explícito con sobre `{error:{codigo,mensaje}}` — descartado
  por Principio I: agrega una ruta que solo sirve para fallar y una variante de error para un caso
  que ningún consumidor provoca. Si un cliente externo necesita el sobre JSON, se reevalúa con un
  requisito concreto.
- **Trade-off aceptado**: la respuesta 405 no lleva cuerpo JSON; los tests afirman status y que la
  fila no cambió.

## R16 — Unicidad de nombre de categoría sin distinguir mayúsculas

- **Decision**: índice único funcional `uq_categorias_nombre_normalizado` sobre
  `(lower(btrim(nombre)))`, conservando la `UNIQUE (nombre)` de `0002`.
- **Rationale**: el requerimiento pide nombre único y la spec clarifica que la comparación ignora
  mayúsculas y espacios sobrantes. La normalización en aplicación no alcanza: dos escrituras
  concurrentes con distinta capitalización pasarían. El índice lo resuelve en la base. Conservar la
  constraint exacta evita una migración destructiva y actúa como guarda redundante barata.
- **Alternatives considered**: normalizar a minúsculas en una columna extra (duplica el dato y hay
  que mantenerlo); `citext` (extensión adicional para un solo campo); validar solo en la aplicación
  (carrera y datos históricos inconsistentes).
- **Límite documentado**: no pliega acentos; «Almacén» y «Almacen» son nombres distintos.

## R17 — Frontend SvelteKit 3: alias `#lib` y runes

- **Decision**: imports con `#lib/...`, componentes en modo runes, `adapter-static` con fallback
  `index.html`.
- **Rationale**: verificado en el código existente (`frontend/src/routes/login/+page.svelte` y
  `admin/usuarios/+page.svelte` usan `#lib`), en `vite.config.ts` (runes forzado, adapter-static) y
  en `package.json` (`@sveltejs/kit ^3`, `svelte ^5`).
- **Alternatives considered**: `$lib` (era el alias de SvelteKit 1/2; el progreso de 003 ya registró
  que en la 3 es `#lib`). La constitución dice `$lib/api/*.ts` como **ejemplo**; el requisito
  normativo es que el acceso HTTP esté centralizado y tipado, que se cumple con `#lib/api/*.ts`.
- **Acción derivada**: no se enmienda la constitución en esta feature; queda anotado como
  redacción desactualizada del ejemplo.

## R18 — Numeración de migraciones y ledger

- **Decision**: la migración nueva es `0006_catalogo_descripcion_codigo_barras.sql`.
- **Rationale**: en `/migrations` existen `0001`, `0002`, `0003` y `0005`; `0004` corresponde al WIP
  deshecho y no se reutiliza porque la numeración de SQLx es lineal y ya fue aplicada en entornos
  locales. El ledger de [specs/README.md](../README.md) reserva `0005` para 004 y `0006` para 006,
  pero `0005` ya es `sesion_unica`: ambas reservas están desactualizadas.
- **Alternatives considered**: renumerar `0005` (rompe `_sqlx_migrations` en bases existentes);
  usar `0004` (colisiona con el WIP y con cualquier base donde se haya aplicado).
- **Seguimiento**: actualizar el ledger de `specs/README.md` en la fase de tareas, no acá.
