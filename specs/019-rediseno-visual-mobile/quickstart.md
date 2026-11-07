# Quickstart: validación del rediseño móvil

**Feature**: 019-rediseno-visual-mobile | **Date**: 2026-11-07

Guía para verificar la funcionalidad de punta a punta. No reemplaza a `tasks.md` ni contiene
implementación.

## Prerrequisitos

- Backend Rust/Axum y PostgreSQL en marcha con datos sembrados de 003–017 (al menos 20 productos, 2
  depósitos, stock bajo mínimo y uno en cero, y movimientos del día).
- `frontend/` con dependencias instaladas.
- Playwright disponible en la raíz (`npx playwright install chromium` si hace falta).
- Variables para la evidencia: `CAPTURE_EMAIL` y `CAPTURE_PASSWORD` de un usuario `GESTION`, y
  `CAPTURE_EMAIL_OPERACION` / `CAPTURE_PASSWORD_OPERACION` de un usuario `OPERACION`.
- Servidor de desarrollo en `http://localhost:5173` (o el que indique Vite).

## Comandos

```bash
# Suite de lógica pura (incluye el guard de contraste)
cd frontend
npm run test

# Tipos y plantillas
npm run check

# Build de producción (SPA)
npm run build

# Evidencia móvil (desde la raíz, con el servidor de desarrollo en marcha)
node scripts/capturar-movil.mjs
```

## Escenarios de validación

Cada escenario indica el criterio que comprueba. Los pasos manuales usan emulación de dispositivo
(Chrome DevTools o Playwright) con 390×844.

### V1. Oscuro obligatorio y sin destello (FR-104, SC-048)

1. Con preferencia `light` guardada en `localStorage` (`stockmante:tema = light`), abrir la app a 390
   px.
2. Recargar 10 veces cinco rutas distintas.
3. **Esperado**: todas las cargas se pintan oscuras desde el primer frame; no hay destello claro ni
   salto de layout. A 1440 px, la preferencia `light` vuelve a aplicarse y el selector de tema
   aparece.

### V2. Accesibilidad del oscuro (FR-103, FR-133, SC-042, SC-049)

1. Ejecutar `npm run test` y confirmar que el guard de contraste pasa y falla si se altera un token
   por debajo del mínimo (probar cambiando un valor y revirtiéndolo).
2. Activar `prefers-contrast: more` en el sistema operativo o en las DevTools (`Emulate CSS media
   feature`); recorrer Inicio, Stock y una hoja.
3. Con Playwright, emular `forcedColors: 'active'` y recorrer las mismas pantallas.
4. **Esperado**: con `more`, textos, bordes y estados suben de contraste y siguen oscuros; con
   `forced-colors`, no hay glow ni sombras, los badges conservan texto y borde, y nada queda
   ilegible.

### V3. Marco y navegación (FR-107…FR-110, SC-046)

1. A 390 px, abrir Inicio: cabecera compacta, barra inferior con 4 destinos + "Más".
2. Navegar a cada destino de la barra; luego abrir "Más" y recorrer los 17 grupos.
3. Abrir Auditoría: "Más" queda activo y la ruta se marca en el panel.
4. Cerrar el panel con fondo, Escape y eligiendo un destino; verificar foco devuelto y fondo sin
   scroll.
5. Repetir con un usuario `OPERACION`: sin Conteos ni Administración.
6. **Esperado**: todo destino permitido a ≤ 2 toques; sin desborde horizontal; `aria-current`
   correcto.

### V4. Búsqueda rápida (FR-111, SC-041)

1. Abrir el buscador de la cabecera e ingresar un SKU exacto, parte de un nombre y un código de
   barras.
2. **Esperado**: resultados en tarjetas con SKU, nombre y badge de stock; sin resultados, mensaje con
   acción para limpiar; el caso común responde en menos de 300 ms percibidos y no deja respuestas
   viejas.

### V5. Tablero móvil (FR-114…FR-116)

1. Abrir Inicio con alertas y sin alertas.
2. **Esperado**: carrusel de una tarjeta por vista, alertas primero cuando existen, valores iguales a
   la API, navegación por teclado y botones con nombre accesible; un endpoint caído solo rompe su
   tarjeta.

### V6. Tarjetas de inventario (FR-117…FR-121, SC-043)

1. Abrir Stock, Productos, Caja, Movimientos, Compras y Auditoría a 390 px.
2. Verificar SKU, nombre truncado, badge con texto (Normal/Bajo/Crítico) y acciones de 44×44.
3. Girar a 844×390: las listas pasan a dos columnas o tabla compacta.
4. A 320×568: sin desborde horizontal; título con elipsis.
5. Paginar y filtrar: el total y los filtros activos se conservan.
6. **Esperado**: ninguna lectura esencial exige zoom ni desplazamiento lateral.

### V7. Registro rápido (FR-122…FR-126, SC-046, SC-047)

1. En Stock, tocar una tarjeta → "Ajuste de stock".
2. Verificar hoja a pantalla completa, teclado numérico, fondo bloqueado y foco atrapado.
3. Ingresar una cantidad mayor al stock del depósito: el envío se bloquea con mensaje junto al campo
   y foco en el campo.
4. Ingresar una cantidad válida, confirmar y verificar el aviso con tipo, producto, depósito,
   cantidad y stock resultante; el botón no permite doble envío.
5. Repetir desde el FAB de Movimientos y desde Caja ("Nueva venta").
6. **Esperado**: 4 toques o menos desde la tarjeta; el fondo no se desplaza en 10 intentos; el aviso
   se anuncia como estado y no tapa la acción siguiente.

### V8. Acceso (FR-127, SC-040)

1. A 320, 390 y 844 px, recorrer login, registro, verificación y cambio obligatorio de contraseña.
2. Enfocar cada campo con el teclado en pantalla abierto.
3. **Esperado**: sin desborde horizontal; el campo activo y el botón quedan visibles; mismos campos,
   validaciones y mensajes que 001.

### V9. Administración y reportes (FR-127)

1. Recorrer las 11 pantallas de administración a 390 px.
2. En Reportes, abrir Kardex, Rotación, Valorizado y Ventas.
3. **Esperado**: cada tabla se lee como tarjetas o filas-tarjeta con etiquetas; las acciones de fila
   conservan su resultado y sus mensajes.

### V10. Área segura y teclado (FR-106, SC-048)

1. Probar en un dispositivo o emulación con muesca y barra de gestos.
2. **Esperado**: cabecera y barra inferior respetan `env(safe-area-inset-*)`; el FAB no queda tapado;
   los campos no provocan zoom automático.

### V11. Movimiento reducido (FR-105)

1. Activar `prefers-reduced-motion: reduce`.
2. **Esperado**: sin transición de panel/hoja ni pulsación del destello; el glow estático se
   conserva.

### V12. Sin regresiones (SC-044)

1. A 1440×900, recorrer las rutas principales y comparar con la evidencia de 018.
2. Ejecutar `npm run test`, `npm run check` y `npm run build`.
3. **Esperado**: mismo comportamiento y mensajes; suite y build en verde; el tema claro de escritorio
   sigue disponible.

### V13. Rendimiento de lista (SC-041)

1. Sembrar 10 000 productos y filtrar Stock por una letra a 390 px.
2. **Esperado**: p95 de filtro a lista pintada < 300 ms; scroll fluido; sin saltos al paginar.

## Evidencia

`scripts/capturar-movil.mjs` guarda en `specs/019-rediseno-visual-mobile/evidencia/`:

```text
evidencia/
├── vertical/     # 390×844, 21 pantallas + cambio de contraseña
├── horizontal/   # 844×390, 21 pantallas + cambio de contraseña
├── compacta/     # 320×568, 21 pantallas + cambio de contraseña
└── validacion.md # resultado de V1–V13 y del guard de contraste
```

Cada captura se nombra con el número y la ruta (`04-inicio.png`, `05-caja.png`, …) para comparar con
la evidencia de 018.

## Criterio de cierre

- V1–V13 cumplidos y registrados en `evidencia/validacion.md`.
- `npm run test`, `npm run check` y `npm run build` en verde.
- Cero cambios de backend, esquema o permisos en el diff.
- Las 21 pantallas y el cambio obligatorio de contraseña tienen evidencia vertical y horizontal.
