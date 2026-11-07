# Tasks: 017-asistente-inventario

**Input**: Design documents from `/specs/017-asistente-inventario/`

**Prerequisites**: plan.md ✓, spec.md ✓

**Revisado**: 2026-11-06. Las marcas `[X]` anteriores se retiraron: solo existía un handler stub
(503 sin `LLM_API_KEY`, 200 con texto fijo en otro caso); no había servicio, tools, tests,
pantalla ni documentación. El stub se reemplazó en T005.

**Implementado**: 2026-11-07 (`/speckit implement`). Ver «Registro de RED» al final.

## Format: `[ID] [P?] [Story] Description`

## Path Conventions

- Backend: `backend/src/`, `backend/tests/`
- Servicio de asistente: `asistente/` (Node + LangChain.js en TypeScript, tests con Vitest)
- Frontend: `frontend/src/`

---

## Phase 1: Backend (proxy, límites, disponibilidad)

Tests primero: cada test DEBE observarse en RED antes de T005 (Principio IV, sin waiver).

- [X] T001 [US17] Test RED en `backend/tests/integration/test_asistente_http.rs`: sin
  `ASISTENTE_URL` → 503 `asistente_no_disponible`; servicio mock caído → 503 (escenario 17.4,
  FR-068)
- [X] T002 [US17] Test RED: 21.ª consulta del mismo usuario dentro de 1 minuto → 429
  `asistente_limite_excedido`; otro usuario no se ve afectado (17.7, FR-068). La ventana con reloj
  controlado está en `backend/tests/unit/test_asistente_limite.rs`.
- [X] T003 [US17] Test RED: servicio mock que tarda más que el timeout → 504
  `asistente_tiempo_agotado` (17.8, FR-068; timeout inyectable en el test)
- [X] T004 [US17] Test RED: el backend reenvía la sesión y la familia del usuario al servicio mock,
  nunca la toma del mensaje; sin sesión → 401; con 12 mensajes de historial reenvía solo los 10
  más recientes (17.5, 17.6, FR-068, FR-077)
- [X] T004b [US17] Test RED: el log de una consulta contiene `usuario_id`, duración y código, y
  no contiene el texto del mensaje ni del historial (FR-078) —
  `backend/tests/integration/test_asistente_log.rs` (binario propio, ver registro)
- [X] T005 [US17] Implementar `backend/src/services/asistente.rs` (`ConfigAsistente` cargada al
  arrancar en `AppEstado`, límite por usuario, cliente con timeout), variantes
  `AsistenteCaido`, `AsistenteLimiteExcedido` y `AsistenteTiempoAgotado` en `error.rs`, y
  reemplazar el stub de `handlers/asistente.rs` + OpenAPI. Sin `unwrap`/`expect`/`panic!`.

## Phase 2: Servicio de asistente (LangChain)

- [X] T006 [US17] Esqueleto del servicio en `asistente/` con configuración `LLM_PROVIDER`,
  `LLM_API_KEY`, `LLM_MODEL`; sin `LLM_API_KEY` responde no disponible (17.4) —
  `src/config.ts`, `src/servidor.ts`, `src/main.ts`
- [X] T007 [US17] Tests con LLM mock + implementación de las tools de lectura de FR-066
  (`consultar_stock`, `buscar_producto`, `listar_depositos`, `resumen_ventas`,
  `reporte_agregado`) llamando a la API Rust con la sesión recibida; el log del servicio cumple
  FR-078 (17.1, 17.6, FR-078)
- [X] T008 [US17] Tests con LLM mock + `accion_propuesta` con el formato de `POST /api/movimientos`;
  `AJUSTE` solo para familia `GESTION`; con varias coincidencias de producto no propone y lista
  hasta 5 candidatos con nombre y SKU (17.2, 17.3, 17.10, FR-067)
- [X] T009 [US17] Test de matriz tools × propuestas × familia de rol (GESTION/OPERACION): cero
  escrituras y cero propuestas fuera de permiso (SC-025, FR-069) — `src/matriz.test.ts`
- [X] T010 [US17] Test de instrucciones inyectadas en datos de una tool: no hay escritura y, como
  mucho, una propuesta (17.9) — `src/agente.test.ts`

## Phase 3: Frontend

- [X] T011 [P] [US17] `frontend/src/lib/api/asistente.ts` + `asistente.test.ts`: tipos del
  contrato, manejo de 429/503/504/401 con el mensaje de la API (Principios II y IV)
- [X] T012 [US17] `frontend/src/routes/asistente/+page.svelte`: chat con historial solo en memoria
  (últimos 10 mensajes, sin `localStorage`/`sessionStorage`, FR-077), tarjeta de propuesta
  Confirmar/Cancelar (`lib/components/asistente/TarjetaPropuesta.svelte`) que usa
  `$lib/api/movimientos.ts`; al confirmar muestra producto, depósito y stock resultante; al
  cancelar no llama a ninguna escritura (17.2, FR-067, FR-076)

## Phase 4: Integración y cierre

- [X] T013 [US17] Test de integración: propuesta confirmada → `POST /api/movimientos` deja
  auditoría con el usuario real como autor; con stock insuficiente se muestra el error real
  (17.2, FR-069)
- [X] T014 [P] Documentar `/api/asistente/consulta` (contrato, 429/503/504) en `docs/api.md` y
  `ASISTENTE_URL` en `.env.example`
- [X] T015 Quality gates: `cargo test`, `cargo sqlx prepare --check`, `npm test`, `npm run check`,
  `npm run build`, y tests del servicio de asistente en verde

## Dependencias

- T001–T004b → T005. T006 → T007–T010. T005 y T008 → T012. T012 → T013. T015 al final.

## Registro de RED (Principio IV)

- **Backend**: con el esqueleto previo, 8 de 10 tests HTTP fallaron por comportamiento (200
  "stub" en vez de 422/429/503/504, historial no reenviado, sin log). `sin_configuracion_responde_503`
  y `sin_sesion_responde_401` pasaron desde el inicio porque el stub ya cubría esos casos.
  Los tests de servicio con 503 o 500 se agregaron después; el de 503 (17.4) se vio en RED.
- **Test de logs**: pasaba aislado pero fallaba junto a otros tests en paralelo porque se cacheaba
  el interés de los callsites de `tracing`; se movió a su propio binario (`test_asistente_log`).
- **Servicio `asistente/`** y **frontend `$lib/api/asistente`**: RED por módulos inexistentes
  (los tests se escribieron antes que el código).
- **T013**: verifica comportamiento existente de 007 (alta y auditoría); no tuvo RED propio.
- **Ajuste de spec**: 17.4 distingue «no configurado» de «no responde» (mismo código 503).
