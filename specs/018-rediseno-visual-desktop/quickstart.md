# Quickstart: validar el rediseño visual

**Feature**: `018-rediseno-visual-desktop` | **Fecha**: 2026-11-07

Guía para levantar la aplicación, recorrer las 18 pantallas y comprobar que el rediseño cumple la
spec sin regresiones. No incluye código de implementación; eso vive en `tasks.md`.

## 1. Prerrequisitos

- Node 24 y npm 11 (verificado en el entorno).
- PostgreSQL 15+ con la base `stockmante` migrada.
- Backend Rust compilado y corriendo en `http://localhost:3000`.
- Playwright con Chromium ya instalado (`node_modules/@playwright/test` y
  `%LOCALAPPDATA%\ms-playwright\chromium-1248`).

## 2. Puesta en marcha

```powershell
# 1) Backend (si no está corriendo)
cd backend
cargo run --bin stockmante-backend

# 2) Frontend
cd frontend
npm install
npm run dev
# http://localhost:5173
```

Credenciales: ejecutar `cargo run --bin bootstrap` e iniciar sesión con el admin que imprime la
consola; el primer ingreso obliga a cambiar la contraseña (ese mismo flujo se valida en US18.3).
Para datos de tablero, sembrar productos, depósitos y algunos movimientos; los scripts
`scripts/seed_catalogo_5000.sql` y `seed_catalogo_10000.sql` sirven para probar volumen.

## 3. Validación por historia

### US18.1 — Marco, navegación y tema (P1)

1. Iniciar sesión como `GESTION` y recorrer los 14 ítems de la barra lateral.
2. Verificar que el ítem activo se resalta, que la URL coincide y que `aria-current="page"` está en
   el ítem (Inspector).
3. Cerrar sesión e iniciar como `OPERACION`: la barra solo muestra Inicio, Caja, Movimientos,
   Transferencias y Asistente; `/admin/usuarios` redirige a `/caja`.
4. Con stock bajo mínimo, verificar el contador en Stock y su desaparición cuando el total es 0.
5. Elegir `Light`, recargar; elegir `Dark`, recargar; elegir `System` y cambiar la preferencia del
   sistema operativo. En las tres, comprobar que no hay destello y que `data-tema` coincide.
6. Reducir la ventana a 1024, 900 y 320 px: sin scroll horizontal del `body`; la sidebar se vuelve
   panel superpuesto y se cierra con `Escape` o backdrop.
7. Recorrer todo el marco solo con `Tab`: foco visible, orden lógico, sin trampas.

**Criterio**: SC-032, SC-034 y FR-079/080/084/085/088/089/095.

### US18.2 — Tablero de Inicio (P2)

1. Abrir `/` con datos. Anotar los cuatro indicadores.
2. Contrastar cada número con la API en DevTools:
   - `GET /api/stock?bajo_minimo=true&por_pagina=1` → `total`.
   - `GET /api/movimientos?desde=<hoy>T00:00:00Z&por_pagina=1` → `total`.
   - `GET /api/productos?activo=true&por_pagina=1` → `total` (solo GESTION).
   - `GET /api/reportes/stock-valorizado` → `total` (solo GESTION).
3. Verificar el gráfico de barras por tipo y la dona bajo mínimo/total; leer la leyenda y el texto
   alternativo.
4. Con un usuario `OPERACION`, confirmar que no se ven ni se solicitan los widgets de gestión.
5. Simular un endpoint caído (bloquear la URL en DevTools): solo ese widget muestra error con
   reintento; el resto sigue usable.
6. Vaciar los datos (o usar una base limpia): ceros y estados vacíos, sin errores.

**Criterio**: SC-035 y FR-090.

### US18.3 — Acceso (P3)

1. `/login` con credenciales inválidas: mensaje real de la API, sin perder el email.
2. `/login` con credenciales válidas: redirección a `/`; con contraseña temporal, modo de cambio
   obligatorio y política visible.
3. `/registro`: alta válida, email duplicado y validaciones de contraseña.
4. `/verificar-correo` con token válido, sin token y con token inválido.
5. En un entorno sin SSO, verificar que no aparecen botones de proveedor y sí el aviso.
6. Repetir con tema claro y oscuro; comprobar foco, `role="alert"` y el selector de tema.

**Criterio**: SC-033 y FR-081/083/094.

### US18.4 — Operación (P4)

| Página | Acción mínima | Resultado esperado |
|---|---|---|
| `/caja` | Buscar por SKU y por categoría | Resultados y aviso de truncados con la nueva jerarquía |
| `/movimientos` | Registrar una entrada | Producto, depósito, cantidad y stock resultante visibles; historial actualizado |
| `/transferencias` | Transferir entre depósitos | Stock resultante en origen y destino; historial paginado |
| `/conteos` | Abrir, cargar líneas y cerrar un conteo | Estados con badge; diferencias y ajuste visibles |
| `/asistente` | Preguntar por stock y confirmar una propuesta | Carga, respuesta y tarjeta de confirmación funcionales |

**Criterio**: FR-091 y SC-036.

### US18.5 — Administración (P5)

Para cada una de las ocho páginas: listar, filtrar, paginar y ejecutar una acción de fila.

| Página | Acción de fila |
|---|---|
| `/admin/usuarios` | Editar, desactivar, reactivar, nueva contraseña temporal |
| `/admin/categorias` | Editar, desactivar, reactivar |
| `/admin/productos` | Editar, desactivar, reactivar; filtros y tabs |
| `/admin/depositos` | Crear depósito, editar ubicaciones, asignar ubicaciones a producto |
| `/admin/stock` | Filtrar por depósito y bajo mínimo; paginar |
| `/admin/proveedores` | Editar, activar, desactivar |
| `/admin/compras` | Crear, confirmar, anular; ver detalle |
| `/admin/auditoria` | Filtrar por tabla/usuario y paginar |

Comprobar en cada una: encabezado + acción principal, tarjeta de filtros, tabla con badges,
paginación, estado vacío y mensajes de error/éxito reales.

**Criterio**: FR-092 y SC-036.

## 4. Verificación automatizada

```powershell
cd frontend
npm run test      # Vitest: suite existente + navegacion, formato, graficos
npm run check     # svelte-check: tipos y plantillas
npm run build     # build SPA sin errores
```

Capturas de evidencia (con backend y frontend levantados):

```powershell
node scripts/capturar-rediseno.mjs
```

El script recorre las 18 rutas en claro y oscuro a 1440×900 y guarda los chequeos angostos en
`specs/018-rediseno-visual-desktop/evidencia/`. Revisar que existan 18 + 18 imágenes y que ninguna
tenga scroll horizontal.

## 5. Contraste y accesibilidad

- Usar el Inspector o axe DevTools sobre cada página en ambos temas.
- Verificar los pares de `contracts/tokens.md` §5; si un acento no alcanza 4.5:1, se ajusta el token
  (variante oscurecida) y se repite la captura.
- Confirmar que los estados (bajo mínimo, confirmada, anulada, inactiva) se leen por texto además
  del color.
- Probar `prefers-reduced-motion: reduce` y verificar que no hay animaciones esenciales.

## 6. Checklist de cierre

- [ ] `npm run test` y `npm run check` en verde.
- [ ] Las 18 rutas navegables desde el marco (salvo acceso) y sin errores de consola.
- [ ] 18 capturas claras + 18 oscuras + chequeos angostos.
- [ ] Sin `scrollWidth > clientWidth` en `body` a 320, 768, 1024 y 1440 px.
- [ ] Contraste AA en ambos temas.
- [ ] Cero regresiones en los flujos de acceso, operación y administración.
- [ ] `specs/README.md` actualizado con el estado de 018.

## 7. Problemas frecuentes

| Síntoma | Causa probable | Acción |
|---|---|---|
| Destello claro al recargar | `data-tema` no se aplica antes del paint | Revisar el script inline de `app.html` y la clave `stockmante:tema` |
| Sidebar no aparece | La ruta se clasificó como pública | Revisar `esRutaPublica` y la condición de `+layout.svelte` |
| Widget de gestión visible a OPERACION | Filtro de rol ausente | Revisar `gruposVisibles` y el guard del tablero |
| Tabla desborda en 320 | Falta `overflow-x: auto` en el contenedor | Ajustar `TablaDatos`, no el `body` |
| Capturas con otro tamaño | Playwright sin viewport fijo | Fijar 1440×900 y `deviceScaleFactor: 1` |
| Números del tablero no coinciden | Filtro de fecha o de rol distinto | Comparar contra la URL exacta de la API |
