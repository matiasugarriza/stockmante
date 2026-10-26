# API Contracts — Base, autenticación y RBAC

**Base**: `/api` | **Formato**: JSON (UTF-8) | **Auth**: cookie de sesión `HttpOnly` salvo login | **Creado**: 2026-10-12 | **Dividido**: 2026-10-25

Errores de dominio → 409/422 (variante `AppError` específica); no autorizado → 401/403; no encontrado → 404; infraestructura → 500 `{ "error": { "codigo": "interno", "mensaje": "Error interno. Contacte al administrador." } }` sin detalles de BD (Principio V).

Shape de error estándar (sin adornos — Principio III):

```json
{ "error": { "codigo": "validacion", "mensaje": "El nombre es obligatorio." } }
```

> Este archivo es el **contrato base** que consumen todas las specs: sobre de error, cookie de
> sesión y guards. Los endpoints de cada caso de uso viven en su propio `contracts/api.md`:
> [002](../../002-gestion-usuarios/contracts/api.md),
> [003](../../003-catalogo-productos/contracts/api.md).

## Auth

| Method | Path | Auth | Descripción |
|--------|------|------|-------------|
| POST | `/auth/login` | pública | Body: `{ "email": "…", "password": "…" }` → 204 + Set-Cookie sesión (HttpOnly; SameSite=Lax; `Secure` en producción; `path=/`); revoca sesión anterior. 401 `{codigo:"credenciales_invalidas"}` genérico para email inexistente, password mal o inactivo. |
| POST | `/auth/logout` | sesión | Revoca sesión; 204. |
| GET | `/auth/sesion` | sesión | → `{ "usuario": { id, nombre, email, rol, familia_rol, debe_cambiar_password } }` o 401. |
| POST | `/auth/cambiar-password` | sesión | Body: `{ "password_actual", "password_nueva" }` (la actual puede ser la temporal). Valida política. 204; `password_temporal=false`. 422 si no cumple política. |

## Guards

| Extractor | Exige | Rechazo |
|---|---|---|
| `AuthUsuario` | Cookie de sesión vigente | 401 `no_autenticado` / `sesion_expirada` |
| `RequierePasswordCambiada` | `password_temporal = false` (salvo `GET /auth/sesion` y `POST /auth/cambiar-password`) | 403 `cambio_password_requerido` |
| `RequiereGestion` | Sesión vigente + `familia_rol = GESTION` + `password_temporal = false` | 403 `sin_permisos` / `cambio_password_requerido` |

El gate de contraseña temporal se evalúa **antes** que la familia y aplica a todo endpoint autenticado, no solo a los de gestión (FR-008a; T028).

## Sesión

- Cookie `HttpOnly`, `SameSite=Lax`, `path=/`, `Secure` cuando `APP_ENV=production`.
- Expiración **absoluta** de `SESSION_EXPIRY_HOURS` (default 8) desde el login; sin sliding.
- Una sola sesión vigente por usuario; el login revoca la anterior (índice único parcial; T027).

## Salud

| Method | Path | Auth | Descripción |
|--------|------|------|-------------|
| GET | `/salud` | pública | Healthcheck operativo → 200 `{ "ok": true }`. Sin datos de negocio ni de BD. |

## RBAC — matriz (resumen, fuente de verdad)

| Acción | Familia operación | Familia gestión |
|--------|-------------------|-----------------|
| login/logout/cambiar-password (propia) | ✅ | ✅ |
| buscar producto, ver catálogo | ✅ | ✅ |
| CRUD usuarios, productos, categorías, depósitos, ubicaciones, precios, bajas | ❌ 403 | ✅ |

La verificación transversal de esta matriz (todos los endpoints de escritura) es la tarea
principal de [006](../../006-rbac-auditoria/spec.md).

## Endpoints por spec

| Spec | Endpoints |
|---|---|
| 001 | `/auth/*` |
| 002 | `/usuarios*` |
| 003 | `/categorias*`, `/productos*` (CRUD) |
| 004 | `/productos/buscar` |
| 005 | `/depositos*`, `/ubicaciones*`, `PUT /productos/{id}/ubicaciones` |
| 006 | Sin endpoints nuevos (auditoría interna + verificación) |
