# Implementation Plan: Escaneo y generación de códigos de barras/SKU

**Branch**: `020-escaneo-codigos-barra` | **Date**: 2026-11-07 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `specs/020-escaneo-codigos-barra/spec.md` (clarificada
2026-11-07, 5 preguntas).

## Summary

Identificar productos leyendo su código con la **cámara del celular** (medio principal: el usuario
no tiene lector), con un lector tipo pistola que emula teclado o a mano, en los 5 buscadores de
producto (caja, movimientos, transferencias, conteos y catálogo). Gestión puede además **generar y
asignar un EAN-13** (automático o desde 12 dígitos) y **imprimir o descargar etiquetas** de
50 × 30 mm en EAN-13, Code 128 o QR.

Enfoque técnico ([research.md](./research.md)):

- **Lectura**: ponyfill `barcode-detector` (zxing-wasm, wasm servido local, carga diferida) para
  EAN-13, EAN-8, UPC-A, Code 128, Code 39 y QR; un solo componente `EscanerCamara` reutilizable.
- **Resolución**: `resolverLectura` en el frontend sobre el `GET /api/productos/buscar` existente
  con coincidencia exacta (código y luego SKU). Sin endpoint nuevo de lectura.
- **Generación**: funciones puras de EAN-13 en Rust + `POST /api/productos/{id}/codigo-barras`
  transaccional, con confirmación de reemplazo en el servidor y auditoría específica. La validación
  del dígito verificador se suma a `validar_codigo_barras` (ficha, alta e importación).
- **Etiquetas**: SVG generado en el navegador con `jsbarcode` + `qrcode-generator`; impresión del
  navegador y descarga `.svg`. El formato no se persiste.
- **Celular**: `npm run dev:movil` sirve por HTTPS autofirmado en la red local y reenvía `/api` al
  backend por el mismo origen.

## Technical Context

**Language/Version**: Rust (edición del crate actual) + Axum + SQLx; TypeScript 6 + Svelte 5
(runes) sobre SvelteKit 3 en modo SPA, Vite 8 y Tailwind CSS 4.

**Primary Dependencies**: existentes. Nuevas, todas en `frontend/` (justificadas en research):
`barcode-detector` 3.x (dependencia, lectura), `jsbarcode` 3.12.x y `qrcode-generator` 2.0.x
(dependencias, etiquetas), `@vitejs/plugin-basic-ssl` 2.3.x (devDependency, HTTPS local). Backend
sin crates nuevos. Versiones exactas fijadas en el lockfile con al menos dos semanas de publicadas.

**Storage**: PostgreSQL, sin migración. Se reutilizan `productos.codigo_barras` y `auditoria`.

**Testing**: `cargo test` (unit `test_ean13`, HTTP `test_codigo_barras_http`, ajustes en
`test_producto_validacion` e importación); Vitest `node` para `ean13.ts`, `escaner.ts`,
`etiqueta.ts` y el cliente `productos.ts`; `svelte-check`; validación manual con celular según
[quickstart.md](./quickstart.md) y evidencia con capturas.

**Target Platform**: navegadores móviles (Chrome Android, Safari iOS) y de escritorio; cámara solo
en contexto seguro (localhost o HTTPS en la red local).

**Project Type**: aplicación web (backend Rust/Axum + frontend SvelteKit SPA).

**Performance Goals**: producto visible < 2 s desde la captura (SC-050); detección cada ~150 ms;
el wasm y las librerías de etiqueta no pesan en la carga inicial (import diferido).

**Constraints**: sin escrituras de stock por escanear (FR-150); sin servicios externos ni CDN (el
wasm se sirve local); sin `unwrap/expect/panic!` en backend; mensajes P-III; área de toque 44 px y
foco atrapado en el overlay (019/FR-156).

**Scale/Scope**: 4 historias (2 P1, 2 P2), 28 FR (FR-134 … FR-161), 12 SC (SC-050 … SC-061);
1 endpoint nuevo; 4 componentes nuevos; 5 pantallas integradas.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principio | Evaluación | Estado |
|---|---|---|
| I. Simplicidad | Sin migración ni endpoint de lectura; reutiliza la búsqueda exacta de 004. 4 dependencias de frontend, cada una con necesidad concreta y alternativa descartada (R1, R6, R7). Sin modo continuo, sin persistir lecturas ni formatos. | ✅ |
| II. Mantenibilidad | Un componente de escáner reutilizable; lógica pura en `$lib/codigos/*`; HTTP solo por `$lib/api/productos.ts`; el servicio nuevo vive en `services/productos.rs` con funciones puras en `models/codigo_barras.rs`. | ✅ |
| III. Mensajes | Catálogo de mensajes de cámara y de API con qué pasó / por qué / qué hacer; resultado concreto al asignar. Se mejora el mensaje de `CodigoBarrasDuplicado` para incluir el siguiente paso. | ✅ |
| IV. Testabilidad | EAN-13, normalización, clasificación de errores y formatos de etiqueta son funciones puras deterministas (sin reloj ni aleatoriedad: el EAN automático deriva del `id`). Tests RED antes de implementar. La cámara se aísla en el componente. | ✅ |
| V. Integridad | No hay movimientos de stock. La asignación es transaccional con `FOR UPDATE`, unicidad por constraint (`23505`) y auditoría en la misma transacción. Esquema contrastado con `migrations/`; T000 lo verifica contra la BD real antes del backend. | ✅ (T000 pendiente) |
| Seguridad / RBAC | Endpoint nuevo con `RequiereGestion`; botones de gestión ocultos para operación; la cámara solo con acción explícita del usuario. | ✅ |

**Re-check post-diseño (Phase 1)**: sin violaciones. Complejidad registrada abajo solo por las
dependencias nuevas.

## Project Structure

### Documentation (this feature)

```text
specs/020-escaneo-codigos-barra/
├── spec.md              # clarificada 2026-11-07
├── plan.md              # este archivo
├── research.md          # R1–R9 + verificación de esquema
├── data-model.md        # reglas de codigo_barras, auditoría y entidades transitorias
├── quickstart.md        # validación con celular
├── contracts/
│   ├── api.md           # POST /api/productos/{id}/codigo-barras y reutilizados
│   └── ui.md            # componentes, mensajes e integración por pantalla
├── checklists/requirements.md
└── tasks.md             # /speckit-tasks
```

### Source Code (repository root)

```text
backend/
├── src/
│   ├── error.rs                       # + CodigoBarrasExistente(String); mensaje de duplicado
│   ├── docs.rs                        # + path OpenAPI
│   ├── models/
│   │   ├── mod.rs                     # + pub mod codigo_barras
│   │   ├── codigo_barras.rs           # NUEVO: dígito verificador, validación, EAN automático
│   │   └── producto.rs                # validar_codigo_barras + regla EAN-13
│   ├── services/
│   │   ├── auditoria.rs               # + ACCION_ASIGNAR/REEMPLAZAR_CODIGO_BARRAS
│   │   └── productos.rs               # + asignar_codigo_barras (transacción)
│   └── handlers/productos.rs          # + POST /{id}/codigo-barras
├── tests/
│   ├── unit/test_ean13.rs             # NUEVO
│   ├── unit/test_producto_validacion.rs   # ajuste del ejemplo EAN
│   └── integration/test_codigo_barras_http.rs  # NUEVO
└── Cargo.toml                         # + [[test]] x2

frontend/
├── package.json                       # + deps y script dev:movil
├── vite.config.ts                     # modo movil: basic-ssl + proxy /api + PUBLIC_API_URL vacío
└── src/
    ├── lib/
    │   ├── codigos/                   # NUEVO: ean13.ts, escaner.ts, etiqueta.ts (+ tests)
    │   ├── api/productos.ts           # + resolverLectura, asignarCodigoBarras (+ tests)
    │   ├── schemas/productos.ts       # + esquemas de request/resultado
    │   └── components/codigos/        # NUEVO: BotonEscanear, EscanerCamara,
    │                                  #        AsignarCodigo, EtiquetaProducto
    └── routes/
        ├── caja/+page.svelte
        ├── movimientos/+page.svelte
        ├── transferencias/+page.svelte
        ├── conteos/+page.svelte
        └── admin/productos/+page.svelte   # + asignar código y etiqueta

docs/api.md, docs/casos-de-uso.md     # actualizar
specs/README.md                       # estado de 020 y registro FR-157 … FR-161, SC-060/061
```

**Structure Decision**: web app existente (backend + frontend). La lógica nueva de dominio va en
`models/codigo_barras.rs` (pura) y `services/productos.rs` (transacción); en el frontend, en
`$lib/codigos/` (pura) y `$lib/components/codigos/` (UI).

## Complexity Tracking

| Violation | Why Needed | Simpler Alternative Rejected Because |
|---|---|---|
| Dependencia `barcode-detector` (+ zxing-wasm) | decodificar EAN/UPC/Code 128/Code 39/QR desde video en iOS, Android y escritorio | `BarcodeDetector` nativo no existe en Safari iOS ni Firefox; implementar un decodificador es inviable |
| Dependencias `jsbarcode` y `qrcode-generator` | generar símbolos escaneables EAN-13, Code 128 y QR | implementar Code 128 y QR a mano es propenso a etiquetas ilegibles; `bwip-js` es mucho más pesado |
| devDependency `@vitejs/plugin-basic-ssl` | contexto seguro para la cámara del celular en la red local | `mkcert` requiere instalar una CA en el teléfono; un túnel expone la app (descartado en clarify) |
