# Research: Catálogo de productos y categorías

**Date**: 2026-10-25 | **Spec**: [spec.md](./spec.md) | **Plan**: [plan.md](./plan.md)

> R1–R8 (sesión, RBAC, monorepo, correo, reloj) son decisiones de la base compartida y viven en
> [001/research.md](../001-autenticacion-sesion/research.md). Acá se registran solo las
> decisiones propias de US3.

## R9 — Representación del dinero

- **Decision**: `rust_decimal::Decimal` con la feature `serde-float` para serializar como número
  JSON, y la feature `rust_decimal` de SQLx para mapear `NUMERIC(12,2)`.
- **Rationale**: el contrato publica `costo` y `precio_venta` como números, no cadenas; `f64`
  perdería precisión y `serde` por defecto serializa `Decimal` como string. Verificado que
  `serde-float` deserializa con `visit_f64`, que hace `Decimal::from_str(&value.to_string())`:
  `10.1` llega con escala 1 y no se convierte en `10.100000000000001`.
- **Alternatives considered**: `f64` (errores de redondeo en precios); `i64` de centavos
  (obliga a convertir en cada capa y rompe el contrato numérico); serializar como string
  (cambia el contrato acordado).

## R10 — Unicidad sin pre-chequeo (TOCTOU)

- **Decision**: delegar la unicidad al índice y mapear la violación `23505` por **nombre de
  constraint**: `productos_sku_key` → `SkuDuplicado`; `productos_codigo_barras_key` →
  `CodigoBarrasDuplicado`; `categorias_nombre_key` → `CategoriaDuplicada`.
- **Rationale**: un `SELECT` previo no es atómico y agrega una consulta; el mapeo por constraint
  es exacto y funciona bajo concurrencia.
- **Alternatives considered**: verificar con `EXISTS` antes de insertar (carrera); `ON CONFLICT
  DO NOTHING` (no distingue qué constraint falló ni devuelve el motivo al usuario).

## R11 — Borrado físico vs baja lógica

- **Decision**: `DELETE /productos/{id}` solo si no hay filas en
  `producto_ubicacion_permitida`; si las hay → 409 `producto_con_referencias` (se sugiere
  desactivar). El pre-check se hace dentro de una transacción con `SELECT … FOR UPDATE` sobre el
  producto; además se mapea `23503` como red de seguridad. `DELETE /categorias/{id}` se bloquea si
  existe algún producto en esa categoría (`CategoriaConProductos`).
- **Rationale**: FR-012 exige bloquear el borrado con historial; la asignación de ubicaciones es
  hoy la única referencia real (movimientos/ventas son módulos posteriores). El `FOR UPDATE`
  serializa el borrado contra altas concurrentes de asignaciones.
- **Alternatives considered**: confiar solo en `ON DELETE RESTRICT` (la FK de
  `producto_ubicacion_permitida.producto_id` es `CASCADE`, así que no protege); borrar siempre en
  cascada (perdería asignaciones sin aviso).

## R12 — Código de barras opcional y `NULL`

- **Decision**: normalizar el código vacío o en blanco a `NULL`; migración
  `0004_productos_codigo_barras_null_distinct.sql` reemplaza el `UNIQUE NULLS NOT DISTINCT`
  original por `UNIQUE (codigo_barras)` (comportamiento `NULLS DISTINCT` por defecto).
- **Rationale**: la spec dice que el código es opcional; con `NULLS NOT DISTINCT` solo un
  producto podía quedar sin código. En `PUT`, una cadena vacía limpia el código
  (`NULL`), mientras que omitir el campo lo conserva.
- **Alternatives considered**: índice único parcial `WHERE codigo_barras IS NOT NULL`
  (equivalente, pero la migración `0004` ya está aplicada y verificada); usar cadena vacía en
  lugar de `NULL` (rompe la semántica de "sin código" y el único).

## R13 — `parsear_activo` compartido

- **Decision**: mover `parsear_activo` de `handlers/usuarios.rs` a `handlers/mod.rs` como
  `pub(crate)` y reutilizarlo en usuarios, productos y categorías.
- **Rationale**: los tres listados aceptan `?activo=true|false|all` con la misma semántica;
  duplicarlo generaría mensajes y bordes distintos.
- **Alternatives considered**: un tipo `QueryActivo` con `Deserialize` (agrega ceremonia y no
  mejora el mensaje de error 422).

## R14 — Aislamiento de tests con claves únicas

- **Decision**: los tests de integración derivan SKU y código de barras de un UUID para correr en
  paralelo sin colisiones; los tests de servicio usan datos únicos y no limpian si un
  `expect_err` falla.
- **Rationale**: `cargo test` ejecuta targets en paralelo contra la misma BD de test; un SKU fijo
  produce 409 espurios.
- **Alternatives considered**: serializar los tests con un lock global (más lento y frágil);
  truncar tablas entre tests (interfiere con otros targets en paralelo).
