# Plan de implementación: Asistente de inventario (017)

**Spec**: `spec.md` · **Migración**: ninguna · **Revisado**: 2026-11-06 (remediación de
`/speckit analyze`).

## Contexto técnico

- Stack IA fijado por la constitución: **LangChain + API de LLM con Tools**, en un servicio
  separado del backend (supuesto de la spec). Se descartan "MCP servers como tools" y el cliente
  HTTP directo al LLM desde Rust.
- El backend Rust actúa como proxy autenticado: `POST /api/asistente/consulta` →
  `ASISTENTE_URL`, reenviando la sesión del usuario (cookie de sesión) al servicio.
- El servicio llama a la API Rust con esa sesión; nunca toca la base ni tiene credenciales propias.
- El LLM nunca escribe: devuelve propuestas que el usuario confirma y el frontend ejecuta contra
  `POST /api/movimientos` (FR-067), que ya devuelve 201 con autor y stock resultante (FR-076).

## Decisiones de diseño

1. **Configuración al arrancar.** `ConfigAsistente { url, timeout }` se lee una vez de
   `ASISTENTE_URL` y se guarda como `Option` en `AppEstado`. `None` → 503
   `asistente_no_disponible`. Se elimina la lectura de `std::env::var` por request y la respuesta
   200 del stub actual.
2. **Límite por usuario.** Ventana de 1 minuto, 20 consultas, en memoria por `usuario_id` dentro de
   `AppEstado`, usando el reloj inyectable de `auth/reloj.rs` (tests deterministas). Excedido → 429
   `asistente_limite_excedido` (nueva variante de `AppError`).
3. **Timeout.** 30 s sobre la llamada al servicio; vencido → 504 `asistente_tiempo_agotado` (nueva
   variante de `AppError`). Servicio caído o 5xx → 503.
4. **Contrato.** Request `{ mensaje: string, historial: { rol: "usuario" | "asistente", texto:
   string }[] }` (el backend reenvía solo los 10 más recientes, FR-077); respuesta
   `{ respuesta: string, accion_propuesta: AccionPropuesta | null }`, con `AccionPropuesta =
   { tipo, producto_id, deposito_id, cantidad, motivo }` en el mismo formato que acepta
   `POST /api/movimientos`.
5. **Tools del servicio.** Lista cerrada de FR-066, todas de lectura. La propuesta de `AJUSTE`
   solo se ofrece con familia `GESTION` (la familia la informa el backend junto con la sesión); la
   API sigue siendo la autoridad.
6. **Datos no confiables.** El prompt de sistema trata la salida de las tools como datos; la
   defensa real es estructural: no hay tool de escritura (escenario 17.9).
7. **Logs (FR-078).** Una línea estructurada por consulta con `usuario_id`, `duracion_ms`,
   `tools`, `hubo_propuesta` y `codigo`, emitida por el handler del backend y por el servicio. El
   texto del mensaje, el historial, la respuesta y la salida de las tools nunca se pasan al logger.
8. **Frontend.** `$lib/api/asistente.ts` (tipado, con tests) y `routes/asistente/+page.svelte` con
   el chat y una tarjeta de propuesta Confirmar/Cancelar que reutiliza `$lib/api/movimientos.ts`.

## Runtime del servicio (resuelto en clarify 2026-11-06)

- **Node + LangChain.js en TypeScript**, en `asistente/` con `package.json` propio y Vitest.
- Justificación de dependencias (Principio I): Node y npm ya son parte del proyecto (frontend,
  Playwright), así que no se agrega runtime. Se agregan solo LangChain.js y el paquete del
  proveedor LLM elegido, necesarios para cumplir el stack fijado por la constitución. Se descartó
  Python porque sumaba un segundo stack de despliegue y testing (venv/pip, pytest).

## Archivos

- `backend/src/services/asistente.rs` (config, límite por usuario, cliente del servicio),
  `backend/src/handlers/asistente.rs`, `backend/src/error.rs`, `backend/src/app.rs`
- `backend/tests/test_asistente.rs`
- `asistente/` (servicio Node + LangChain.js en TypeScript, con tests Vitest)
- `frontend/src/lib/api/asistente.ts`, `frontend/src/lib/api/asistente.test.ts`
- `frontend/src/routes/asistente/+page.svelte`
- `docs/api.md`, `.env.example`

## Riesgos

- Sin `ASISTENTE_URL` o con el servicio caído el endpoint devuelve 503; el resto del sistema no se
  ve afectado.
- Respuestas del LLM no deterministas: los tests usan un LLM mock en el servicio y un servicio
  mock (servidor HTTP local) en los tests del backend.
- El límite en memoria se reinicia al reiniciar el backend y no se comparte entre réplicas;
  aceptable para un despliegue de una sola instancia.
