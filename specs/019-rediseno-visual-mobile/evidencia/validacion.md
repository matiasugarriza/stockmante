# Evidencia de validación — 019 Rediseño móvil

## Línea base (antes de los cambios)

- `npm run test`: 40 archivos, 327 tests en verde.
- `npm run check`: 0 errores, 0 warnings.
- `npm run build`: correcto.

## Estado de la implementación (2026-11-07)

- `npm run test`: 47 archivos, 388 tests en verde.
- `npm run check`: 0 errores, 0 warnings.
- `npm run build`: correcto.
- Guard de contraste: 13 tests sobre la paleta real de `movil.css` (AAA de cuerpo, 3:1 de
  controles, `prefers-contrast` y prohibición de tonos crudos como texto).
- Tareas completadas: 58 de 87 (`tasks.md`).

## Tareas pendientes

- Tarjetas móviles: transferencias, conteos, ventas, clientes, auditoría, depósitos, reportes e
  importación (T050, T051, T057–T062).
- Disposición de dos columnas en horizontal (T063) y verificaciones de historia (T038, T044, T064,
  T074, T079).
- Hoja de venta de Caja (T071), FAB de foco en transferencias/conteos/asistente (T072), cambio
  obligatorio de contraseña (T076), formularios de administración (T077), confirmaciones
  destructivas (T078).
- Auditoría táctil, modos de accesibilidad en dispositivo, rendimiento y evidencia 21×3
  (T081–T087).

## Pendiente de evidencia con backend

- V1–V13 y capturas en 390×844, 844×390 y 320×568. El script está en
  `scripts/capturar-movil.mjs` y requiere `CAPTURE_EMAIL`, `CAPTURE_PASSWORD` y el frontend en
  marcha.
