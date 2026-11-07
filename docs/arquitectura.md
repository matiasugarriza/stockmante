# Arquitectura

## Vista general

```text
Navegador (SPA SvelteKit + Tailwind)
   │  fetch tipado (cookie HttpOnly, credentials: include)
   ▼
Axum (backend/)
   ├── handlers/     HTTP, DTOs, extractores, autorización por extractor
   ├── services/     reglas de negocio y transacciones SQLx
   ├── models/       structs de dominio + validaciones puras
   ├── auth/         sesión, RBAC, contraseñas, gate de cambio obligatorio
   ├── mail/         envío SMTP o stub de consola
   └── docs.rs       OpenAPI + Swagger UI
   ▼
PostgreSQL 18 (migrations/ 0001–0015)
   ▲
   │  API Rust con la cookie del usuario
Asistente Node 24 (LangChain.js)
   └── tools de lectura + proponer_movimiento
```

## Backend

- **Un solo `AppEstado`** (`pool`, `config`, `reloj`, `correo`, `salud_sso`, `asistente`)
  compartido por todos los routers. `Reloj`, `EnvioCorreo` y `SondaSso` son traits para testear
  sin depender del reloj, SMTP ni red.
- **Errores**: todo handler devuelve `Result<_, AppError>`; `IntoResponse` traduce a
  `{ error: { codigo, mensaje } }`. Los errores de infraestructura se loguean y salen como 500
  genérico (nunca se filtra `sqlx::Error`).
- **Transacciones**: las escrituras que tocan más de una fila usan `pool.begin()`. Los
  movimientos de stock, transferencias, compras, ventas, conteos e importaciones son atómicos.
- **Unicidad**: la base impone los índices únicos y se mapea `23505` al error de dominio; no hay
  pre-chequeos TOCTOU.
- **Autorización**: `AuthUsuario` valida la sesión contra la fila actual (un cambio de rol aplica
  en el siguiente request); `RequiereGestion` exige familia `GESTION` y contraseña ya cambiada.
- **Stock**: `stock_por_deposito` es la fuente de verdad; `productos.stock_actual` es un cache
  global que se recalcula en la misma transacción. Nunca se permite stock negativo.
- **Migraciones**: numeración global en `migrations/`; un número no se reutiliza. El ledger vive
  en [`specs/README.md`](../specs/README.md).
- **Documentación**: `utoipa` deriva el OpenAPI de las anotaciones de handlers y modelos.
- **Tests**: unitarios puros (validaciones, password, configuración, cálculo de reportes) e
  integración HTTP con `tower::ServiceExt::oneshot` contra `stockmante_test`.

### Módulos implementados

| Módulo | Specs | Servicios/handlers |
|---|---|---|
| Autenticación y usuarios | 001, 002 | `auth`, `usuarios`, `registro`, `sso` |
| Catálogo | 003, 004 | `categorias`, `productos`, `busqueda` |
| Depósitos y ubicaciones | 005 | `depositos`, `ubicaciones` |
| RBAC y auditoría | 006, 015 | `auditoria` |
| Stock | 007, 008, 009, 010, 011 | `movimientos`, `transferencias`, `conteos`, `stock`, `alertas` |
| Compras y ventas | 012, 013 | `proveedores`, `compras`, `clientes`, `ventas` |
| Reportes e importación | 014, 016 | `reportes`, `importaciones` |
| Asistente | 017 | `asistente` (proxy) + `asistente/` (LangChain) |
| Rediseño visual | 018, 019 | solo frontend |

### Alertas y scheduler

`services/alertas.rs` corre un ciclo cada `ALERTAS_INTERVALO_MINUTOS` (default 60; `0` lo apaga)
y también se puede disparar con `POST /api/alertas/enviar`. Detecta productos activos con
`stock_minimo > 0` y `stock_actual < stock_minimo`, arma un digest por gestor activo y registra
la deduplicación diaria en `notificaciones_stock`. Un fallo SMTP no rompe el ciclo: informa
`fallidos > 0` y reintenta.

### Asistente

`backend/src/services/asistente.rs` es un proxy con límite por usuario (20/min), timeout (30 s) y
reenvío de la cookie de sesión. El servicio `asistente/` (Node + LangChain.js) usa tools de
lectura contra la API y una tool `proponer_movimiento` que devuelve una propuesta; **nunca
escribe**. La escritura la hace el frontend con `POST /api/movimientos`, que valida permisos,
transacción y auditoría. Sin `ASISTENTE_URL` o sin LLM, el endpoint responde 503.

## Frontend

- **SPA**: `ssr = false`, `prerender = false`; el backend resuelve la sesión.
- **Acceso a la API**: solo a través de `src/lib/api/*.ts`, que usa `client.ts`
  (`credentials: 'include'`, `ApiError` con `codigo` y `mensaje`).
- **Validación**: Zod en `src/lib/schemas/*.ts` como espejo del backend; el backend sigue siendo
  la autoridad. Los formularios muestran errores por campo.
- **Sesión**: store `sesion` (`cargando | activa | sin_sesion`) y guardas de layout:
  - sin sesión → `/login`;
  - contraseña temporal → `/login` (cambio obligatorio);
  - rutas `/admin/*` → solo `GESTION`; `OPERACION` va a `/caja`.
- **Tema y diseño (018)**: tokens semánticos claro/oscuro en `src/lib/diseno/tokens.css`,
  primitivas en `src/lib/components/ui`, marco en `src/lib/components/marco`, navegación por rol
  en `src/lib/navegacion/navegacion.ts` y evidencia en
  [`specs/018-rediseno-visual-desktop/evidencia/`](../specs/018-rediseno-visual-desktop/evidencia/).
- **Móvil (019, parcial)**: `src/lib/components/movil/`, `src/lib/diseno/movil.css` y
  `src/lib/dispositivo/`; layout cyberpunk oscuro con navegación táctil.
- **Alias**: `#lib/*` (SvelteKit 3), no `$lib`.

## Datos

| Tabla | Propósito | Spec dueña |
|---|---|---|
| `usuarios`, `sesiones` | cuentas, roles, sesión única | 001 |
| `categorias`, `productos` | catálogo, SKU/código único, baja lógica | 003 |
| `depositos`, `ubicaciones`, `producto_ubicacion_permitida` | trazabilidad espacial | 005 |
| `auditoria` | autor + timestamp de acciones sensibles | 006/015 |
| `stock_por_deposito` | fuente de verdad del stock por depósito/ubicación | 007 |
| `movimientos`, `movimiento_lineas` | entradas, salidas, mermas, ajustes | 007 |
| `transferencias` | lazo entre los dos movimientos de una transferencia | 008 |
| `conteos`, `conteo_lineas` | inventario físico y ajuste | 009 |
| `notificaciones_stock` | deduplicación diaria de alertas | 011 |
| `proveedores`, `compras`, `compra_lineas` | abastecimiento | 012 |
| `clientes`, `ventas`, `venta_lineas` | punto de venta | 013 |

Reglas transversales:

- No hay borrado físico de usuarios, productos ni categorías: baja lógica (`activo = false`).
- Depósitos y ubicaciones admiten `DELETE` solo sin dependencias; si las hay, `409` y se ofrece
  la baja lógica.
- Los productos inactivos se conservan para reportes históricos y no aparecen en la búsqueda de
  caja.
- Los montos son `NUMERIC(12,2)` y viajan como número JSON; las fechas son ISO-8601 UTC.
- Las fechas de reportes son días locales `AAAA-MM-DD` (UTC−03:00), inclusivos.

## Seguridad

- Cookie `HttpOnly`, `SameSite=Lax`, `Secure` en producción; una sola sesión vigente por usuario.
- Argon2id para contraseñas; cambio obligatorio de la temporal.
- RBAC de dos familias (`GESTION` y `OPERACION`) aplicado en extractores; ocultar un enlace no
  reemplaza el 403.
- El asistente no pide tokens ni escribe en la base; las tools usan la sesión del usuario y las
  escrituras se confirman por la API normal.
- El DSN de MCP se lee de `DATABASE_URL`; `dbhub.toml` no contiene credenciales.

## Decisiones de documentación

- `specs/*/contracts/api.md` es la fuente de reglas y mensajes de negocio.
- Swagger/OpenAPI es la vista navegable y el contrato legible por máquina; se genera del código
  para no desincronizarse.
- `DOCS_ENABLED` permite apagar la documentación en producción sin tocar el router.
