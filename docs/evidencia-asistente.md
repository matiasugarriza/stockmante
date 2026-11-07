# Evidencia del asistente con un LLM real

**Fecha**: 2026-11-07 (hora local America/Argentina/Buenos_Aires).
**Objetivo**: cerrar el pendiente de la spec 017 («falta el recorrido manual con un LLM real») y
demostrar que el chat integrado funciona de punta a punta.

## Entorno

- Backend Axum en `http://localhost:3000` (base `stockmante`).
- Frontend SvelteKit en `http://localhost:5173`.
- Servicio `asistente/` en `http://127.0.0.1:3100` (Node 24).
- Proveedor: **Groq**; modelo: `openai/gpt-oss-20b` (`LLM_PROVIDER=groq`, `LLM_MODEL`).
- Usuario de demo: cuenta local `GESTION`/`ADMIN` (no versionada). La contraseña y la API key no
  se incluyen en este documento.

## 1. Consulta de stock (UI)

Pregunta en `/asistente`:

> «¿Cuánto stock hay de Aceite girasol 1,5 L?»

Respuesta del asistente (texto plano):

> «Hay 13 unidades de Aceite girasol 1,5 L disponibles, todas en el depósito Principal.»

La captura está en
[`specs/017-asistente-inventario/evidencia/asistente-demo.png`](../specs/017-asistente-inventario/evidencia/asistente-demo.png).

## 2. Propuesta de movimiento (sin escribir)

Pregunta:

> «Registrá una entrada de 1 unidad de Aceite girasol 1,5 L en Principal»

Respuesta:

> «Propuesta lista: ENTRADA de 1 unidad de Aceite girasol 1,5 L (SKU ACEITE-GIR-15) en depósito
> Principal, motivo COMPRA. No se ejecuta automáticamente; el usuario debe confirmarla en la
> pantalla.»

`accion_propuesta` devuelta por `POST /api/asistente/consulta`:

```json
{
  "tipo": "ENTRADA",
  "deposito_id": 1,
  "deposito_nombre": "Principal",
  "producto_id": 25005,
  "producto_nombre": "Aceite girasol 1,5 L",
  "sku": "ACEITE-GIR-15",
  "cantidad_delta": 1,
  "motivo": "COMPRA",
  "observacion": null
}
```

**Verificación de que no escribió**: antes de confirmar no se creó ningún movimiento; la
respuesta solo contiene la propuesta.

## 3. Confirmación por la API normal

`POST /api/movimientos` con la propuesta (el frontend hace exactamente esto al presionar
**Confirmar**):

```json
{
  "tipo": "ENTRADA",
  "deposito_id": 1,
  "motivo": "COMPRA",
  "observacion": null,
  "lineas": [{ "producto_id": 25005, "ubicacion_id": null, "cantidad_delta": 1 }]
}
```

Respuesta `201`:

```json
{
  "id": 10,
  "tipo": "ENTRADA",
  "deposito": { "id": 1, "nombre": "Principal" },
  "motivo": "COMPRA",
  "observacion": null,
  "usuario": { "id": 9, "nombre": "Demo TP" },
  "lineas": [{
    "producto_id": 25005,
    "producto_nombre": "Aceite girasol 1,5 L",
    "sku": "ACEITE-GIR-15",
    "cantidad_delta": 1,
    "stock_resultante": 13
  }]
}
```

## 4. Auditoría

`GET /api/auditoria?tabla_ref=movimientos&por_pagina=5` devuelve la fila del movimiento
confirmado:

```json
{
  "id": 30,
  "tabla_ref": "movimientos",
  "registro_id": 10,
  "accion": "CREAR",
  "usuario_id": 9,
  "usuario_nombre": "Demo TP",
  "cuando": "2026-11-07T22:04:31.942078Z"
}
```

## Conclusión

- El chat integrado responde con datos reales obtenidos por las tools.
- El asistente **no** escribe: devuelve una propuesta.
- La confirmación se ejecuta por la API autenticada, con el usuario real, stock resultante y
  auditoría.
- Los tests automáticos del servicio (54) y del backend (`test_asistente_*`) siguen en verde.
- Queda documentado el recorrido manual que faltaba en `specs/017-asistente-inventario/spec.md`.

## Reproducción

1. Configurar `.env` con `LLM_PROVIDER`, `LLM_MODEL`, `LLM_API_KEY` y
   `ASISTENTE_URL=http://127.0.0.1:3100`.
2. Levantar backend, frontend y `cd asistente && npm start`.
3. Iniciar sesión como `GESTION`, abrir `/asistente` y repetir las preguntas.
4. Para la captura automatizada:
   ```bash
   CAPTURE_EMAIL=... CAPTURE_PASSWORD=... node scripts/capturar-asistente.mjs
   ```
