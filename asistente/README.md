# Asistente de inventario (LangChain)

Servicio Node + **LangChain.js** que atiende el chat de `/asistente`. El backend Rust le reenvía
la consulta y la cookie de sesión; el servicio usa **tools** contra la API y **nunca escribe en la
base**. Las escrituras se proponen y el usuario las confirma en la pantalla, que llama a
`POST /api/movimientos`.

## Arquitectura

```text
Navegador (/asistente)
   │ POST /api/asistente/consulta  { mensaje, historial }
   ▼
Backend Axum (proxy con límites, timeout y sesión)
   │ POST http://127.0.0.1:3100/consulta  + cookie
   ▼
asistente/ (Node 24 + LangChain.js)
   ├── prompt.ts    prompt de sistema
   ├── agente.ts    bucle modelo → tools → modelo (máx. 6 pasos)
   ├── tools.ts     tools de lectura + proponer_movimiento
   ├── modelo.ts    ChatAnthropic / ChatOpenAI (Groq compatible)
   └── servidor.ts  HTTP, validación y logging de metadatos
   │ tools: GET a la API Rust con la cookie
   ▼
API Rust (permisos reales del usuario)
```

## Tools

| Tool | Tipo | Qué hace |
|---|---|---|
| `consultar_stock` | lectura | Stock y faltante de un producto/depósito |
| `buscar_producto` | lectura | Busca por nombre, SKU o código |
| `listar_depositos` | lectura | Depósitos activos |
| `resumen_ventas` | lectura | Ventas del período |
| `reporte_agregado` | lectura | Stock valorizado, rotación o ventas |
| `proponer_movimiento` | propuesta | Devuelve `accion_propuesta`; **no** escribe |

## Configuración

Variables leídas por `npm start` (desde `../.env`):

| Variable | Uso |
|---|---|
| `ASISTENTE_PUERTO` | Puerto local (default `3100`) |
| `API_BASE_URL` | API Rust que consumen las tools (default `http://localhost:3000`) |
| `LLM_PROVIDER` | `anthropic`, `openai` o `groq` |
| `LLM_API_KEY` | Clave del proveedor (sin ella responde 503) |
| `LLM_MODEL` | Modelo; obligatorio con `openai`/`groq` |
| `ASISTENTE_URL` | En el **backend**: URL de este servicio (`http://127.0.0.1:3100`) |
| `ASISTENTE_TIMEOUT_SEGUNDOS` | En el **backend**: timeout (default `30`) |

Con `LLM_PROVIDER=groq`, `LLM_MODEL=openai/gpt-oss-20b` y `ASISTENTE_URL=http://127.0.0.1:3100`
la configuración local queda lista.

## Seguridad

- El servicio solo escucha en `127.0.0.1`; lo consume el backend, no el navegador.
- La sesión la inyecta el backend; el asistente **nunca** pide tokens ni contraseñas.
- Las tools leen con la cookie del usuario y respetan RBAC; el backend valida permisos de nuevo
  al confirmar un movimiento.
- El prompt de sistema trata la salida de las tools como **datos**, no como instrucciones.
- Logs solo con metadatos (`usuario_id`, duración, tools, si hubo propuesta, código); nunca el
  texto del mensaje, la respuesta ni los datos de las tools.
- Límites del backend: 20 consultas/minuto por usuario, 30 s por consulta, historial de 10
  mensajes solo en memoria.

## Desarrollo y pruebas

```bash
cd asistente
npm install
npm test       # 54 tests: config, agente, tools, matriz, servidor
npm run check  # tsc --noEmit
npm start      # Node 24; requiere LLM_API_KEY
```

Tests destacados:

- `tools.test.ts`: lectura, normalización, propuestas y errores.
- `matriz.test.ts`: tools × propuestas × familia de rol; cero escrituras y cero propuestas fuera
  de permiso.
- `agente.test.ts`: instrucciones inyectadas en datos de una tool no producen escritura.
- `servidor.test.ts`: sesión, historial, límites y logging.

## Demo

1. Backend: `cd backend && cargo run --bin stockmante-backend`.
2. Frontend: `cd frontend && npm run dev`.
3. Asistente: `cd asistente && npm start`.
4. Entrar a `/asistente` como `GESTION` y probar:
   - «¿Cuánto stock hay de `<producto>`?»
   - «Buscá `<texto>`»
   - «¿Cómo fueron las ventas de los últimos 7 días?»
   - «Registrá una entrada de 5 unidades de `<producto>` en Principal» → confirmar.

## Errores frecuentes

| Síntoma | Causa / solución |
|---|---|
| `503 asistente_no_disponible` | Sin `ASISTENTE_URL`, sin `LLM_API_KEY` o servicio caído |
| `429 asistente_limite_excedido` | Más de 20 consultas en un minuto; esperar |
| `504 asistente_tiempo_agotado` | Consulta > 30 s; acortar o subir `ASISTENTE_TIMEOUT_SEGUNDOS` |
| `401 no_autenticado` | Sesión vencida; volver a iniciar sesión |
| El servicio no arranca | Node < 24; usar Node 24 (TS nativo y `--env-file-if-exists`) |
