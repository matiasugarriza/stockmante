# API Contracts — Módulo 1

**Base**: `/api` | **Formato**: JSON (UTF-8) | **Auth**: cookie de sesión `HttpOnly` salvo login | **Date**: 2026-10-12

Errores de dominio → 409/422 (variante `AppError` específica); no autorizado → 401/403; no encontrado → 404; infraestructura → 500 `{ "error": { "codigo": "interno", "mensaje": "Error interno. Contacte al administrador." } }` sin detalles de BD (Principio V).

Shape de error estándar (sin adornos — Principio III):

```json
{ "error": { "codigo": "sku_duplicado", "mensaje": "Ya existe un producto con SKU SKU-001." } }
```

## Auth

| Method | Path | Auth | Descripción |
|--------|------|------|-------------|
| POST | `/auth/login` | pública | Body: `{ "email": "…", "password": "…" }` → 204 + Set-Cookie sesión; revoca sesión anterior. 401 `{codigo:"credenciales_invalidas"}` genérico para email inexistente, password mal o inactivo. |
| POST | `/auth/logout` | sesión | Revoca sesión; 204. |
| GET | `/auth/sesion` | sesión | → `{ "usuario": { id, nombre, email, rol, familia_rol, debe_cambiar_password } }` o 401. |
| POST | `/auth/cambiar-password` | sesión | Body: `{ "password_actual", "password_nueva" }` (la actual puede ser la temporal). Valida política. 204; `password_temporal=false`. 422 si no cumple política. |

## Usuarios (familia GESTION)

| Method | Path | Body / Query | Éxito | Errores clave |
|--------|------|--------------|-------|----------------|
| GET | `/usuarios` | `?activo=true\|false\|all` paginado | 200 lista | 403 NoAutorizado |
| GET | `/usuarios/{id}` | — | 200 usuario | 404 |
| POST | `/usuarios` | `{ nombre, email, rol }` — genera contraseña temporal y la envía por correo | 201 + `{ id, email, rol, password_temporal: true }` (nunca el hash ni la contraseña en respuesta de prod) | 409 email_duplicado; 422 validacion |
| PUT | `/usuarios/{id}` | `{ nombre?, email?, rol? }` | 200 | 409; 422 |
| POST | `/usuarios/{id}/desactivar` | — | 204; revoca sesiones | 409 ultimo_administrador; 404 |
| POST | `/usuarios/{id}/reactivar` | — | 204 | 404 |
| POST | `/usuarios/{id}/password-temporal` | — | 204; nueva temporal enviada | 404; 403 si no gestión |

**GET /usuarios item**: `{ id, nombre, email, rol, familia_rol, activo, password_temporal, creado_en }` — sin `password_hash`.

## Categorías (GESTION para escritura; lectura autenticada)

| Method | Path | Body | Éxito |
|--------|------|------|-------|
| GET | `/categorias` | — | 200 lista |
| POST | `/categorias` | `{ nombre }` | 201 |
| PUT | `/categorias/{id}` | `{ nombre, activo? }` | 200 |
| DELETE | `/categorias/{id}` | — | 204 si sin productos; 409 con productos asociados |

## Productos

| Method | Path | Auth | Body / Query | Éxito | Errores clave |
|--------|------|------|--------------|-------|----------------|
| GET | `/productos/buscar` | sesión | `?q=&campo=sku\|codigo\|nombre\|categoria` — `q` min 2 chars si campo=nombre; exacto para SKU/código | 200 `{ items: [...] }` orden estable | 422 si q < 2 en nombre |
| GET | `/productos` | gestión | paginación + filtros | 200 | 403 |
| GET | `/productos/{id}` | sesión | — | 200 | 404 |
| POST | `/productos` | gestión | `{ nombre, sku, codigo_barras?, categoria_id, unidad_medida, costo, precio_venta, stock_minimo }` | 201 | 409 sku_duplicado / codigo_barras_duplicado; 422 |
| PUT | `/productos/{id}` | gestión | mismos campos editables | 200 | 409; 402/422 |
| DELETE | `/productos/{id}` | gestión | — | 204 si sin referencias; 409 producto_con_referencias (mensaje sugiere baja lógica) | |
| POST | `/productos/{id}/desactivar` | gestión | — | 204 | 404 |

**Item producto**: `{ id, nombre, sku, codigo_barras, categoria: {id,nombre}, unidad_medida, costo, precio_venta, stock_minimo, activo }`.

## Depósitos y ubicaciones (GESTION)

| Method | Path | Body | Éxito | Errores |
|--------|------|------|-------|---------|
| GET/POST | `/depositos` | POST: `{ nombre, identificador, direccion? }` | 200/201 | 409 nombre/identificador únicos |
| PUT/DELETE | `/depositos/{id}` | PUT body; DELETE solo sin dependencias | 200/204 | 409 deposito_con_dependencias |
| GET | `/depositos/{id}/ubicaciones` | — | 200 árbol o lista plana con `padre_id` | |
| POST | `/ubicaciones` | `{ deposito_id, padre_id?, nombre, tipo }` | 201 | 422 ubicacion_huerfana si deposito_id inválido o ≠ padre |
| PUT/DELETE | `/ubicaciones/{id}` | DELETE solo sin hijos/asignaciones | 200/204 | 409 |
| PUT | `/productos/{id}/ubicaciones` | `{ ubicacion_ids: number[] }` reemplaza conjunto | 204 | 404 ubicación |

## RBAC — matriz (resumen)

| Acción | Familia operación | Familia gestión |
|--------|-------------------|-----------------|
| login/logout/cambiar-password (propia) | ✅ | ✅ |
| buscar producto, ver catálogo | ✅ | ✅ |
| CRUD usuarios, productos, categorías, depósitos, ubicaciones, precios, bajas | ❌ 403 | ✅ |
