# Plan de implementación: Autenticación y sesión segura (001) — enmienda SSO/registro

**Spec**: `spec.md` · **Rama**: `feat/001-autenticacion-sesion` · **Migraciones**: `0012_registro_sso.sql`

## Contexto técnico

- Tablas `usuarios`, `sesiones` existen desde `0001`; `familia_rol`/`password_temporal` desde `0001`/`0005`.
- Patrón de capas: `models` → `services` (SQLx + transacciones) → `handlers` (HTTP, DTOs, auth extractor) → `app.rs`.
- La auditoría se escribe con `services::auditoria::registrar` dentro de la misma transacción.
- Los errores de dominio viven en `AppError` con `status()` y `codigo()` estables.
- Stack fijo: Rust + Axum + SQLx + PostgreSQL; SvelteKit SPA + Tailwind.

## Decisiones de diseño

1. **Auto-registro por correo**: nuevo endpoint `POST /api/auth/registro` crea usuario con
   `password_temporal = false`? No — el registro por correo crea cuenta con contraseña elegida
   por el usuario, `password_temporal = false`, `email_verificado = false`, estado inactivo hasta
   verificar. Se envía token de verificación (un solo uso, expiración 24h) por correo.
   `GET /api/auth/verificar-correo?token=` activa la cuenta.
2. **SSO (Google/Microsoft)**: flujo OAuth 2.0 / OIDC con authorization code. Se redirige al
   proveedor (`GET /api/auth/sso/{proveedor}/iniciar`), el callback (`GET /api/auth/sso/{proveedor}/callback`)
   intercambia el code por tokens, valida el ID token, extrae email verificado, y crea/vincula
   cuenta local. Las credenciales del proveedor se configuran por env (`GOOGLE_CLIENT_ID`,
   `GOOGLE_CLIENT_SECRET`, `MICROSOFT_CLIENT_ID`, `MICROSOFT_CLIENT_SECRET`, `SSO_REDIRECT_BASE`).
   Se usa la crate `oauth2` (ya en dependencias? verificar) o implementación manual con `reqwest`.
   Decisión: usar `reqwest` (ya presente para SMTP?) — verificar; si no, agregar dependencia
   justificada en la spec.
3. **Tabla `identidades_externas`**: (usuario_id, proveedor, subject, email, creado_en), PK compuesta
   (proveedor, subject) única. Tabla `tokens_verificacion`: (id, usuario_id, token_hash, expira_en,
   usado, creado_en).
4. **Vinculación/desvinculación**: endpoints para usuarios autenticados; desvincular la última
   identidad (si no hay contraseña) → 409 `ultima_identidad`.
5. **Rol por defecto en SSO**: OPERACION. Promoción a GESTION por admin (002).
6. **Anti-enumeración**: registro responde 202 con mensaje genérico tanto si el correo es nuevo
   como si ya existe.
7. **Dependencia nueva**: `oauth2` crate (o `openidconnect`). Justificación: implementar OAuth/OIDC
   manualmente es propenso a errores de seguridad; la crate es estándar. Si no está en Cargo.toml,
   agregar.

## Superficie de API

| Método | Ruta | Permiso | Respuesta |
|---|---|---|---|
| POST | `/api/auth/registro` | público | 202 registro pendiente |
| GET | `/api/auth/verificar-correo?token=` | público | 200 cuenta activada |
| GET | `/api/auth/sso/{proveedor}/iniciar` | público | 302 redirección |
| GET | `/api/auth/sso/{proveedor}/callback` | público | 200 sesión creada (cookie) |
| GET | `/api/auth/identidades` | sesión | 200 lista |
| POST | `/api/auth/identidades/{proveedor}/vincular` | sesión | 200 vinculada |
| DELETE | `/api/auth/identidades/{proveedor}` | sesión | 204 desvinculada |

## Archivos

| Archivo | Cambio |
|---|---|
| `migrations/0012_registro_sso.sql` | tablas `identidades_externas`, `tokens_verificacion`; columna `usuarios.email_verificado` |
| `backend/src/models/identidad.rs` | reglas puras |
| `backend/src/services/registro.rs` | registro por correo, verificación |
| `backend/src/services/sso.rs` | flujo OAuth/OIDC |
| `backend/src/services/identidades.rs` | vincular/desvincular |
| `backend/src/handlers/auth.rs` | nuevos endpoints |
| `backend/src/error.rs` | nuevos errores |
| `frontend/src/lib/schemas/auth.ts` | schemas registro/SSO |
| `frontend/src/lib/api/auth.ts` | clientes |
| `frontend/src/routes/registro/+page.svelte` | pantalla de registro |
| `frontend/src/routes/login/+page.svelte` | botones SSO |
| `docs/api.md` | tabla de rutas |

## Estrategia de pruebas

1. Unitarias: validación de registro (correo, contraseña), proveedor válido.
2. Integración: registro 202, verificación activa cuenta, SSO callback mock (si posible), identidades.
3. Frontend: tests de schemas y cliente.

## Orden de ejecución

1. Migración y aplicación.
2. Modelos, errores, servicios.
3. Handlers y router.
4. Tests backend.
5. Frontend.
6. Quality gates.

## Riesgos

- **SSO requiere credenciales externas**: los tests de integración de SSO real no son ejecutables sin client_id/secret. Se implementa el flujo y se testea con mocks o se omite la verificación real en CI.
- **Dependencia nueva**: `oauth2` o `reqwest` para token exchange. Verificar Cargo.toml.