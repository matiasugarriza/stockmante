# Research — Búsqueda de caja (004)

**Fecha**: 2026-10-26 | **Spec**: [spec.md](./spec.md) | **Plan**: [plan.md](./plan.md)

## Decisiones

### 1. Motor de búsqueda: PostgreSQL + `pg_trgm`

**Decisión**: usar `pg_trgm` con índice GIN sobre `productos.nombre` y consultas exactas para
SKU/código.

**Alternativas descartadas**:
- `tsvector`/full-text: bueno para palabras completas, malo para fragmentos y errores de tipeo;
  la caja busca subcadenas ("yerb" → "Yerba").
- Búsqueda en el frontend: exigiría traer todo el catálogo; no escala y duplica la autoridad.
- `LIKE` sin índice: aceptable con pocos productos, degrada linealmente.

**Evidencia**: `pg_trgm` está disponible como extensión trusted en PostgreSQL 15+; se creó sin
superusuario en la base local (verificado con `CREATE EXTENSION`). El operador `%` usa el umbral
de similitud (0.3 por defecto) y `ILIKE '%q%'` cubre subcadenas por debajo del umbral.

### 2. Ranking

**Decisión**: `todos` prioriza igualdad exacta de SKU (0) y código (1), luego similitud de
nombre; desempate por `nombre ASC, id ASC`.

**Por qué**: el escenario 1 y 2 exigen que el producto exacto quede primero; el 3 exige orden
predecible. `similarity()` es determinista para el mismo par de textos.

### 3. Longitud mínima

**Decisión**: SKU/código desde 1 carácter (igualdad exacta); nombre/categoría desde 2.

**Por qué**: el caso borde de la spec pide no saturar con términos cortos, pero un SKU puede
tener 1 carácter tipeado y aun así resolverse exacto. En `todos`, un término de 1 carácter solo
compara igualdad.

### 4. Inactivos

**Decisión**: `incluir_inactivos=false` por defecto; `true` exige GESTION y se valida en el
handler (403 `sin_permisos`).

**Por qué**: FR-013 separa la búsqueda de venta de la histórica; el permiso es de rol, no de
consulta. Mantener el chequeo en el handler evita que el servicio dependa del usuario.

### 5. Forma de la respuesta

**Decisión**: `{ items: [...] }`, sin `total/pagina/por_pagina`.

**Por qué**: el contrato preliminar y `docs/api.md` ya publican esa forma; la caja necesita una
lista acotada por `limite`, no navegar páginas. El listado de gestión conserva su paginación.

### 6. Frontend

**Decisión**: página `/caja` para ambas familias, con debounce de 250 ms, selector de campo,
filtro por categoría, checkbox de inactivos solo para GESTION y tabla de resultados con precio.

**Por qué**: el Operario no debe entrar a `/admin/productos` (GESTION). La sesión ya expone
`familia_rol`; el backend sigue siendo la autoridad del 403.

## Ampliación 2026-11-07

### 7. Punto de medición de SC-002

**Decisión**: medir el tiempo de respuesta de la API vista desde el cliente: desde que se envía
el request hasta terminar de parsear el JSON, sin el debounce de 250 ms (clarificación 1).

**Por qué**: es lo que siente el cajero una vez que dejó de tipear y es reproducible; `/caja` ya
calcula ese mismo tramo en `ms`.

**Alternativas descartadas**:
- Solo SQL (`EXPLAIN ANALYZE`): deja fuera red, serialización y Axum; queda como evidencia
  complementaria, no como medida de SC-002.
- Pantalla completa (tecla → render): mezcla los 250 ms fijos del debounce y el tiempo de render,
  difícil de automatizar sin un navegador headless (dependencia nueva).

### 8. Catálogo de referencia

**Decisión**: 10.000 productos (clarificación 2), sembrados con
`scripts/seed_catalogo_10000.sql` (prefijo `SEED4-`), con nombres variados, código de barras y 20
categorías.

**Por qué**: con 112 filas PostgreSQL hace seq scan y el índice GIN nunca se ejercita; 10.000 es
un catálogo realista de comercio mediano y fuerza a evaluar el plan con índice. Reusar el seed de
5.000 de 003 no alcanza: tiene un solo nombre repetido, `codigo_barras` nulo y una sola categoría,
así que no mide ni código ni categoría.

**Alternativas descartadas**: 1.000 (sigue en seq scan), 100.000 (más costoso de preparar sin
cambiar la conclusión para la escala del Módulo 1).

### 9. Herramienta de medición

**Decisión**: script Node `scripts/medir_sc002.mjs` con `fetch` y `performance` nativos, 40
consultas por tipo, p50/p95/máximo.

**Por qué**: sin dependencias nuevas (Principio I); Node 24 está instalado. Las credenciales
llegan por variables de entorno y nunca se escriben en el repo.

**Alternativas descartadas**: `curl` en bucle desde PowerShell (no mide el parseo JSON y el
cálculo de percentiles queda a mano); Playwright/k6 (dependencias nuevas).

### 10. Lógica de debounce fuera del componente

**Decisión**: mover turno y temporizador a `crearBusquedaDiferida(esperaMs)` en
`#lib/utils/busquedaCaja.ts`, junto con `mensajeTruncados`; testear con `vi.useFakeTimers()`.

**Por qué**: el Principio IV pide testear los componentes con lógica y prohíbe dependencias de
reloj ocultas; con timers falsos el reloj queda controlado. Se sigue el patrón de
`#lib/utils/ubicaciones.ts` (003/005).

**Alternativas descartadas**: testear `+page.svelte` con Testing Library (dependencia nueva y
test más frágil); un store genérico de "búsqueda" (abstracción sin una segunda pantalla que la
use).

### 11. Comparación de `campo=categoria`

**Decisión**: igual que `nombre`, `%` o `ILIKE '%q%'` sobre `categorias.nombre` y orden por
similitud (clarificación 4). Es lo que ya implementa `buscar()`; solo se documenta.

## Riesgos abiertos

- Acentos: `pg_trgm` no normaliza diacríticos. "yerba" encuentra "Yerba"; "cafe" no encuentra
  "Café". Se acepta para el Módulo 1 (el catálogo se carga en español y la caja tipea sin
  tilde con frecuencia). `unaccent` queda como mejora futura.
- El umbral de similitud es global; si el catálogo crece mucho, ajustar `pg_trgm.similarity_threshold`
  por sesión o pasar a un ranking explícito.
