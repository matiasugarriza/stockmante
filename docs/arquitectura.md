# Arquitectura

## Vista general

```text
Navegador (SPA SvelteKit)
   │  fetch tipado (cookie HttpOnly, credentials: include)
   ▼
Axum (backend/)
   ├── handlers/     HTTP, DTOs, extractores, autorización por extractor
   ├── services/     reglas de negocio y transacciones SQLx
   ├── models/       structs de dominio + validaciones puras
   ├── auth/         sesión, RBAC, contraseñas, gate de cambio obligatorio
   ├── mail/         envío de la contraseña temporal (SMTP o stub de consola)
   └── docs.rs       OpenAPI + Swagger UI
   ▼
PostgreSQL (migrations/)
```

## Backend

- **Un solo `AppEstado`** (`pool`, `config`, `reloj`, `correo`) compartido por todos los routers.
  `Reloj` y `EnvioCorreo` son traits para poder testear sin depender del reloj real ni de SMTP.
- **Errores**: todo handler devuelve `Result<_, AppError>`; `IntoResponse` traduce a
  `{ error: { codigo, mensaje } }`. Los errores de infraestructura se loguean y salen como 500
  genérico (nunca se filtra `sqlx::Error`).
- **Transacciones**: las escrituras que tocan más de una fila o dependen de una lectura previa usan
  `pool.begin()`. Regla dura para los módulos de stock futuros: todo movimiento es transaccional.
- **Unicidad**: se deja que la base imponga los índices únicos y se mapea `23505` al error de
  dominio; no hay pre-chequeos TOCTOU.
- **Autorización**: `AuthUsuario` valida la sesión contra la fila actual (un cambio de rol aplica en
  el siguiente request). `RequiereGestion` exige familia `GESTION` y contraseña ya cambiada.
- **Migraciones**: numeración global en `migrations/`; un número no se reutiliza aunque se retire
  una migración. El ledger vive en `specs/README.md`.
- **Documentación**: `utoipa` deriva el OpenAPI de las anotaciones de handlers y modelos.
- **Tests**: unitarios puros (validaciones, password, configuración) e integración HTTP con
  `tower::ServiceExt::oneshot` contra `stockmante_test`.

## Frontend

- **SPA**: `ssr = false`, `prerender = false`; el backend resuelve la sesión.
- **Acceso a la API**: solo a través de `src/lib/api/*.ts`, que usa `client.ts`
  (`credentials: 'include'`, `ApiError` con `codigo` y `mensaje`).
- **Validación**: Zod en `src/lib/schemas/*.ts` como espejo del backend; el backend sigue siendo la
  autoridad. Los formularios muestran errores por campo.
- **Sesión**: store `sesion` (`cargando | activa | sin_sesion`) y guardas de layout:
  - sin sesión → `/login`;
  - contraseña temporal → `/login` (cambio obligatorio);
  - rutas `/admin/*` → solo familia `GESTION`; `OPERACION` va a `/caja`.
- **Alias**: `#lib/*` (SvelteKit 3), no `$lib`.

## Datos

Tablas del Módulo 1:

| Tabla | Propósito | Spec dueña |
|---|---|---|
| `usuarios`, `sesiones` | cuentas, roles, sesión única | 001 |
| `categorias`, `productos` | catálogo y baja lógica | 003 |
| `depositos`, `ubicaciones`, `producto_ubicacion_permitida` | trazabilidad espacial | 005 |
| `auditoria` | autor + timestamp de acciones sensibles | 006 |

Reglas transversales:

- No hay borrado físico de usuarios, productos ni categorías: baja lógica (`activo = false`).
- Depósitos y ubicaciones sí admiten `DELETE`, pero solo sin dependencias; si las hay, `409` y se
  ofrece la baja lógica.
- Los productos inactivos se conservan para reportes históricos y no aparecen en la búsqueda de caja.

## Decisiones de documentación

- `specs/*/contracts/api.md` sigue siendo la fuente de reglas y mensajes de negocio.
- Swagger/OpenAPI es la vista navegable y el contrato legible por máquina; se genera del código para
  no desincronizarse.
- `DOCS_ENABLED` permite apagar la documentación en producción sin tocar el router.
