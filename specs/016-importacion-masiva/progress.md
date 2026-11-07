# Progreso de implementación — Importación masiva (016)

**Spec**: [spec.md](./spec.md) | **Plan**: [plan.md](./plan.md) | **Actualizado**: 2026-11-07 |
**Estado**: implementado (T001–T016); T017 (gates) en el cierre transversal; T018 (quickstart
manual) requiere el backend reiniciado con este código.

## Evidencia RED (2026-11-07)

| Test | Antes de implementar |
|---|---|
| `test_importacion_validacion` | no compila: `detectar_dialecto`, `parsear_archivo`, `validar_fila`, `repetidos`, `DetalleImportacion` y `AppError::ImportacionInvalida` no existían |
| `test_importacion_http` | 4/4 FAILED: `aplicar` con errores escribía parcialmente y respondía 422 con el resumen crudo (sin sobre ni `detalles`), sin detección de `;`/decimal `,`, sin depósito por nombre/«Principal», límite de 2 MB de Axum antes del de negocio, `OPERACION` vía chequeo manual |
| `api/client.test.ts` (FormData, detalles) y `api/importaciones.test.ts` | FAILED: `apiFormData`, `ApiError.detalles` e `importarProductos` no existían |

## Cambios

- `AppError::ImportacionInvalida(Vec<DetalleImportacion>)` (422 `importacion_invalida`) con
  `detalles` en el sobre solo para esa variante; `ErrorDetalle` del OpenAPI con `detalles` opcional.
- `services/importaciones.rs` reescrito: parser y validación puros (reutiliza los validadores de
  003), duplicados en archivo y en BD con fila/columna, categoría y depósito por nombre, y
  persistencia atómica (productos, stock, `ENTRADA`/`IMPORTACION`, `stock_actual` y auditoría).
- Handler con `RequiereGestion`, `DefaultBodyLimit` de 16 MB, 200/201/422.
- Frontend: `apiFormData` y `ApiError.detalles` en `client.ts`, schemas, `importarProductos`,
  pantalla `/admin/importacion` (Validar → Aplicar, resumen y tabla de errores) y enlace «Importar».

## Resultado

`test_importacion_http` 4/4 (+ `sc024` ignorado), `test_importacion_validacion` 7/7,
`test_apperror` 4/4, `test_openapi` 11/11; frontend 327/327, `svelte-check` 0/0.

**SC-024** (`sc024`, corrido a mano el 2026-11-07): 5.000 filas aplicadas en **10,3 s** (< 60 s),
5.000 movimientos `IMPORTACION`; el test limpia lo que crea.
