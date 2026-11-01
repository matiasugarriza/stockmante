# Plan de implementación: Autenticación y sesión segura (001) — enmienda SSO/registro

**Spec**: `spec.md` · **Rama**: `main` · **Migraciones**: `0001_usuarios_sesiones.sql` (base), `0012_registro_sso.sql` (registro y SSO)

## Contexto técnico

- Tablas `usuarios`, `sesiones` existen desde `0001`; `familia_rol`/`password_temporal` desde `0001`.
- Patrón de capas: `models` → `services` (SQLx + transacciones) → `handlers` (HTTP, DTOs, auth extractor) → `app.rs`.
- La auditoría se escribe con `services::auditoria::registrar` dentro de la misma transacción.
- Los errores de dominio viven en `AppError` con `status()` y `codigo()` estables.
- Stack fijo: Rust + Axum + SQLx + PostgreSQL; SvelteKit SPA + Tailwind.

## Decisiones de diseño

1. **Auto-registro por correo**: nuevo endpoint `POST /api/auth/registro` crea usuario con
   contraseña elegida por el usuario, `password_temporal = false`, `email_verificado = false`,
   estado inactivo hasta verificar. Se envía token de verificación (un solo uso, expiración 24h)
   por correo. `GET /api/auth/verificar-correo?token=` activa la cuenta.
   Anti-enumeración: responde 202 con mensaje genérico tanto si el correo es nuevo como si
   ya existe.
2. **Configuración de SSO**: endpoint público `GET /api/auth/sso/config` devuelve
   `{ "google_disponible": bool, "microsoft_disponible": bool }` para que el frontend decida
   qué botones renderizar. Si las credenciales de un proveedor no están configuradas,
   invocar sus rutas de inicio/callback responde 503/422 `sso_no_configurado`.
3. **SSO (Google/Microsoft)**: flujo OAuth 2.0 / OIDC con authorization code. Redirección al
   proveedor (`GET /api/auth/sso/{proveedor}/iniciar`), callback
   (`GET /api/auth/sso/{proveedor}/callback`) intercambia el code por tokens, valida el ID
   token, extrae email verificado, y crea/vincula cuenta local. Credenciales por env
   (`GOOGLE_CLIENT_ID`, `GOOGLE_CLIENT_SECRET`, `MICROSOFT_CLIENT_ID`,
   `MICROSOFT_CLIENT_SECRET`, `SSO_REDIRECT_BASE`).
4. **Tabla `identidades_externas`**: (usuario_id, proveedor, subject, email, creado_en),
   PK compuesta (proveedor, subject). Tabla `tokens_verificacion`: (id, usuario_id,
   token_hash, expira_en, usado, creado_en).
5. **Vinculación/desvinculación**: endpoints para usuarios autenticados; desvincular la
   última identidad (si no hay contraseña) → 409 `ultima_identidad`.
6. **Rol por defecto en SSO**: OPERACION. Promoción a GESTION por admin (002).
7. **Contraseña temporal (admin)**: al crear usuario, el administrador elige generación
   manual o automática (14 caracteres, ≥1 mayúscula, ≥1 dígito). La UI muestra un
   mensaje emergente con correo + contraseña temporal para entrega inmediata.
   `password_temporal = true` bloquea el primer ingreso hasta el cambio.

## Superficie de API

| Método | Ruta | Permiso | Respuesta |
|---|---|---|---|
| POST | `/api/auth/registro` | público | 202 registro pendiente |
| GET | `/api/auth/verificar-correo?token=` | público | 200 cuenta activada |
| GET | `/api/auth/sso/config` | público | 200 `{google_disponible, microsoft_disponible}` (credenciales + salud, FR-071) |
| GET | `/api/auth/sso/{proveedor}/iniciar` | público | 307 redirección al proveedor |
| GET | `/api/auth/sso/{proveedor}/callback` | público | 303 redirige al panel con sesión creada (cookie) |
| GET | `/api/auth/login` | pública | 204 + Set-Cookie (ver handlers existentes) |
| POST | `/api/auth/logout` | sesión | 204 |
| GET | `/api/auth/sesion` | sesión | 200 datos de sesión |
| POST | `/api/auth/cambiar-password` | sesión | 204; `password_temporal=false` |
| GET | `/api/auth/identidades` | sesión | 200 lista |
| POST | `/api/auth/identidades/{proveedor}/vincular` | sesión | 200 vinculada |
| DELETE | `/api/auth/identidades/{proveedor}` | sesión | 204 desvinculada |

**Errores nuevos / específicos**:

- `correo_ya_registrado`: 202 (mensaje genérico anti-enumeración).
- `sso_no_configurado`: 503 / 422 si se invoca `GET /api/auth/sso/{proveedor}/iniciar` o
  `GET /api/auth/sso/{proveedor}/callback` con un proveedor sin credenciales.
- `identidad_ya_vinculada`: 409.
- `ultima_identidad`: 409.

## Archivos

| Archivo | Cambio |
|---|---|
| `migrations/0012_registro_sso.sql` | tablas `identidades_externas`, `tokens_verificacion`; columna `usuarios.email_verificado` |
| `backend/src/models/identidad.rs` | reglas puras de identidad externa |
| `backend/src/services/registro.rs` | registro por correo, anti-enumeración, verificación de correo |
| `backend/src/services/sso.rs` | flujo OAuth/OIDC, endpoint config |
| `backend/src/services/identidades.rs` | vincular/desvincular identidades |
| `backend/src/handlers/auth.rs` | nuevos endpoints (registro, sso/config, sso/iniciar, sso/callback) |
| `backend/src/error.rs` | nuevos errores (`sso_no_configurado`, `correo_ya_registrado`) |
| `frontend/src/lib/schemas/auth.ts` | schemas registro/SSO |
| `frontend/src/lib/api/auth.ts` | clientes HTTP para registro y SSO |
| `frontend/src/routes/registro/+page.svelte` | pantalla de formulario de registro |
| `frontend/src/routes/login/+page.svelte` | botones SSO condicionales + login |
| `docs/api.md` | tabla de rutas actualizada |

## Estrategia de pruebas

1. Unitarias: validación de registro (correo, contraseña), disponibilidad de SSO, anti-enumeración.
2. Integración: registro 202, verificación activa cuenta, sso/config respuesta, SSO callback mock.
3. Frontend: tests de schemas y cliente; prueba manual de botones SSO ocultos/mostrados.

## Orden de ejecución

1. Migración `0012_registro_sso.sql` y aplicación.
2. Modelos, errores, servicios (registro, SSO, identidades).
3. Handlers y router (registro, sso/config, sso/iniciar, sso/callback).
4. Tests backend (unit + integración).
5. Frontend (schemas, clientes, rutas).
6. Quality gates.

## Riesgos

- **SSO requiere credenciales externas**: los tests de integración de SSO real no son ejecutables sin client_id/secret. Se implementa el flujo y se testea con mocks.
- **Dependencia OAuth**: verificar si se requiere una crate adicional (`oauth2` o `openidconnect`) para el intercambio de tokens; justificar en Cargo.toml si se agrega.
- **Anti-enumeración**: el registro debe responder 202 con el mismo mensaje independientemente de si el correo existe, para no revelar información.
