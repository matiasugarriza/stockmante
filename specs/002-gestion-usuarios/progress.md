# Progreso de implementación — Gestión de usuarios (002)

**Spec**: [spec.md](./spec.md) | **Actualizado**: 2026-10-25 | **Estado**: Implementado y cerrado

> Retro-spec del caso de uso US2, ya implementado y pusheado. Este archivo conserva la
> arquitectura y la evidencia de verificación que antes vivían en `001/progress.md`.

## Estado verificado

- **tasks.md**: T001–T006 `[X]`.
- **Tests backend**: `test_usuarios_email.rs` (3), `test_usuarios_alta.rs` (1),
  `test_ultimo_admin.rs` (1), `test_familia_rol.rs` (3), `test_usuarios_http.rs` (10 HTTP).
- **Tests frontend**: `schemas/usuarios.test.ts` (9), `api/usuarios.test.ts` (9),
  `utils/paginacion.test.ts` (12).
- **Gates**: `cargo sqlx prepare --check` OK; `npm run check` 0/0; `npm run build` OK;
  sin `unwrap`/`panic!`/`expect` en rutas de producción.
- **Push**: `6e1d722..d87a04f main -> main`.

## Arquitectura

- `backend/src/services/usuarios.rs`: `familia_de_rol` (única fuente del mapa
  ADMIN/GERENTE→GESTION, OPERARIO/CAJERO→OPERACION), `listar` (paginado,
  `activo: Option<bool>`), `obtener`, `crear` (temporal + `EnvioCorreo` dentro de la
  transacción: si el envío falla, no queda cuenta huérfana), `actualizar` (rol
  recalcula familia), `desactivar` (baja lógica + revoca sesiones), `reactivar`,
  `regenerar_password_temporal` (nueva temporal + revoca sesiones).
- Validación de email (`validar_email`, FR-008): exactamente una `@`, partes no
  vacías, sin whitespace y sin puntos consecutivos ni en los bordes de usuario o
  dominio. Rechaza `a@b@c.com`, `a@@b.com`, `a@b..com`, `.a@b.com`, etc. con 422.
- Guarda del último gestor: `bloquear_gestores_activos` toma `FOR UPDATE` sobre
  `familia_rol='GESTION' AND activo ORDER BY id` **antes** de bloquear la fila
  objetivo (mismo orden en `desactivar` y `actualizar` → sin deadlock `40P01`);
  `asegurar_queda_gestor` falla con `UltimoAdministrador` (409) si es el único.
  Aplica a desactivar **y** a degradar el rol (misma invariante, otra puerta).
- Paginación: `pagina >= 1`, `1 <= por_pagina <= 100`, y offset calculado con
  `checked_mul` (extremos → 422, nunca overflow).
- Unicidad de email: sin pre-chequeo; se mapea la violación `23505` de
  `usuarios_email_key` a `EmailDuplicado`.
- `backend/src/handlers/usuarios.rs`: router anidado en `/api/usuarios`; todas las
  rutas con `RequiereGestion`. GET lista devuelve `{ items, total, pagina, por_pagina }`.
  El ítem es `models::usuario::Usuario` (sin `password_hash`).
- Frontend: `lib/schemas/usuarios.ts` (Zod), `lib/api/usuarios.ts`,
  `lib/components/usuarios/UsuarioForm.svelte` + `UsuarioTabla.svelte`,
  `routes/admin/usuarios/+page.svelte`; link "Usuarios" solo para GESTION.
  Los errores de guardado del modal se muestran **dentro** del modal.
- Decisión: no existe `DELETE` físico de usuarios (solo `activo = FALSE`). El alta
  devuelve 201 con el usuario público; la temporal nunca viaja en la respuesta.

## Verificación independiente (2026-10-25)

- Probes del verificador en verde: 38 backend (antes del cierre), 37 frontend,
  `cargo sqlx prepare --check`, `npm run check` 0/0.
- **Blocker corregido**: `validar_email` aceptaba múltiples `@` (`split_once('@')`).
  Ahora rechaza 422 y quedó pinneado por
  `test_usuarios_http.rs::email_con_multiples_arrobas_es_422_en_alta_y_put`.
- Advisories corregidos: deadlock de bajas concurrentes (orden de locks), error del
  modal invisible, huecos de test (401 sin sesión, 403 en POST/PUT de OPERACION,
  `activo=all`, límites de paginación y overflow del offset).
- Advisories derivados a 001 y cerrados ahí: paginador, FR-008a, mailer dev,
  extractores.

## Pendientes técnicos cerrados (2026-10-25)

1. **Paginador admin** (`8fa8390`): `frontend/src/lib/utils/paginacion.ts`
   (`totalPaginas`, `paginaValida`); la ruta pagina de 20 en 20, reinicia a la
   página 1 al cambiar de filtro y recorta a la última página válida.
2. **FR-008a en backend** (`977bd90`): `AppError::DebeCambiarPassword` (403
   `cambio_password_requerido`); `RequiereGestion` lo devuelve mientras
   `password_temporal` siga vigente. `/auth/sesion` y `/auth/cambiar-password`
   usan `AuthUsuario`. El test cubre las **7 rutas** de gestión y verifica que
   ninguna acción rechazada persista cambios.
3. **Mailer dev** (`5a0b971`): `AppConfig` incorpora `APP_ENV` y `es_produccion()`;
   `CorreoDev::aviso_credenciales()` es la superficie testeable; el log de
   `tracing` nunca incluye la credencial.
4. **Sobre de error de extractores** (`977bd90`): `JsonValido<T>`, `PathValido<T>` y
   `QueryValida<T>`; los rechazos de Axum ahora son 422 `validacion` con el sobre
   estándar. Aplicado en `handlers/auth.rs` y `handlers/usuarios.rs`.

## Advisories aceptados

- **A1**: el gating de `APP_ENV` es fail-open (ausente ⇒ development); documentado en
  `.env.example` y en el bootstrap.
- **A2**: sin `RUST_LOG`, `tracing_subscriber` filtra a ERROR y oculta el WARN de credenciales
  en producción.
- **A3**: no hay SMTP real; en producción el stub es silencioso (limitación conocida).
- **A4**: `bandeja()` retiene la temporal en memoria; no hay ruta que la exponga.
- **A5**: `cambiarFiltro` no está guardado por `cargando` (race de clics rápidos, idéntico al
  baseline).

## Siguiente paso

Ninguno para 002. El Módulo 1 continúa en
[003-catalogo-productos](../003-catalogo-productos/progress.md).
