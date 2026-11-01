# Progreso de implementación — Gestión de usuarios (002)

**Spec**: [spec.md](./spec.md) | **Actualizado**: 2026-11-01 | **Estado**: Fase 1 y Phase 2 (`FR-008b`: credenciales manual/automáticas + modal en pantalla) **implementadas y verificadas**

> Retro-spec del caso de uso US2, ya implementado y pusheado. Este archivo conserva la
> arquitectura y la evidencia de verificación que antes vivían en `001/progress.md`.

## Estado verificado

- **tasks.md**: T001–T007 `[X]` (T007 retro-asignada en el análisis del 2026-10-25).
- **Tests backend**: `test_usuarios_email.rs` (3), `test_usuarios_alta.rs` (2),
  `test_ultimo_admin.rs` (1), `test_familia_rol.rs` (3), `test_usuarios_http.rs` (10 HTTP).
- **Tests frontend**: `schemas/usuarios.test.ts` (9), `api/usuarios.test.ts` (9),
  `utils/paginacion.test.ts` (12).
- **Gates**: `cargo sqlx prepare --check` OK; `npm run check` 0/0; `npm run build` OK;
  sin `unwrap`/`panic!`/`expect` en rutas de producción.
- **Cierre de gobernanza (2026-10-25)**: `services/usuarios.rs` migrado a macros
  (`query!`/`query_as!`/`query_scalar!`), `backend/.sqlx` regenerado con `cargo sqlx prepare`
  y verificado con `cargo sqlx prepare --check` + `SQLX_OFFLINE=true cargo check --all-targets`.
  La enmienda constitucional y el waiver viven en 001 (T032, constitución 1.2.0).
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

## Ajuste de especificación 2026-11-01 (Phase 2 pendiente)

El usuario actualizó la spec con la nueva operativa de credenciales al dar de alta:
elección **manual/automática** de la contraseña y **modal obligatoria** con correo + clave en
claro para entrega inmediata, además del correo por el transporte configurado.

**Correcciones aplicadas sobre la spec y los docs de 002 (2026-11-01)**:

1. Se restauró la estructura markdown del `spec.md` (el edit del usuario strippeó headings,
   listas y links): `## Clarifications`, escenarios numerados, `Casos Borde`, `Requisitos`,
   `Entidades Clave`, `Criterios de Éxito`, `Supuestos` y `Trazabilidad`, conservando
   **verbatim** todos los textos nuevos del usuario.
2. **Colisión de identificador resuelta**: el requisito nuevo estaba rotulado `FR-008a`, que ya
   es de 001 en el registro global (`specs/README.md` § Partición de FRs, cual dueño único).
   Se renumeró a **`FR-008b`** (dueño: 002, siguiente subíndice libre del par 008) con nota que
   lo ata a la `FR-008a` de 001. Se actualizó el puntero `001/spec.md` (`002/FR-008a` →
   `002/FR-008b`) y las filas del mapa/registro/decisiones de `specs/README.md`.
3. **Estado corregido**: la línea de estado decía «Implementado con ajustes…» pero los ajustes
   aún no existen en el código (`crear` solo genera automática y la clave solo va por correo);
   ahora apunta a Phase 2 como pendiente.
4. **Contratos alineados** en `contracts/api.md`: `POST /usuarios` con `password?` y respuesta
   envuelta `{ usuario, password_temporal }`; `POST /{id}/password-temporal` 204→**200** con
   `{ password_temporal }` y body opcional. En `contracts/validation.md` se corrigió el error de
   «12 caracteres» de la automática (**`TEMP_LEN` es 14**) y se documentó la política manual.
5. `quickstart.md`: V3 con elección de método + modal obligatoria; fila de regenerado 204→200;
   fila nueva de clave manual fuera de política → 422.

**Plan y tasks regenerados**: `plan.md` ahora documenta Fase 1 (condensada) + **Fase 2** con
decisiones D1–D8 (campo opcional `password`, envoltura de credenciales, 204→200, correo
fail-closed conservado, política compartida con 001). `tasks.md` agrega **Phase 2, T008–T016**
(PRIMERO tests RED → backend → frontend (schemas, selector, modales) → docs → gates).

**Hallazgo resuelto**: `001/spec.md` definió `FR-009`–`FR-012` que colisionaban con 003/004 en
  el registro global; se renumeraron a `FR-070`–`FR-073` (y sus `SC-005`–`SC-007` a
  `SC-026`–`SC-028`) el 2026-11-01 — ver `specs/README.md` § Decisiones de la partición.

## Phase 2 — FR-008b implementada (2026-11-01)

TDD con RED observado en ambos lados (backend E0061/E0308 contra la firma nueva; frontend
7 tests en rojo con la envoltura/política sin implementar) y GREEN completo:

- **Backend**: `elegir_password_temporal` compartido entre `crear` (ahora
  `(Usuario, String)`) y `regenerar_password_temporal` (devuelve `String`); política manual
  reutiliza `validar_politica_password` de 001 (D5); vacío/en blanco ⇒ automática (D1).
  Contratos: 201 `{ usuario, password_temporal }` (`UsuarioCreado`) y 200
  `{ password_temporal }` (`ClaveTemporalGenerada`) — **204 → 200 en regenerar (D3)**; extractor
  nuevo `JsonValidoOpcional` para el body opcional con el sobre de error estándar. El correo
  sigue dentro de la transacción con fail-closed (D4) y la clave en claro no se loguea (D7).
- **Frontend**: `CrearUsuarioSchema.password` con política Zod + `ActualizarUsuarioSchema` sin
  password (D8); `UsuarioCreadoSchema`/`ClaveTemporalGeneradaSchema`; selector manual/automática
  en `UsuarioForm`; `PasswordTemporalModal` para el regenerado (escenario 5) y
  `CredencialesModal` obligatoria con botón «Copiar contraseña» tras alta y regenerado.
- **Puntos tocados por el cambio**: `test_usuarios_http` (alta a la envoltura, regenerar a 200),
  `test_rbac_matriz` (204→200 de password-temporal + id bajo `usuario`), `test_usuarios_email`
  y `test_ultimo_admin` (firma de `crear`).
- **Gates**: `cargo test` 158/158; `cargo sqlx prepare --check` exit 0 (sin queries nuevas);
  `npm test` 174/174; `npm run check` 0/0; `npm run build` OK.

## Siguiente paso

002 queda **cerrada** (Fase 1 + Phase 2). El resto del Módulo 1 continúa en
[003-catalogo-productos](../003-catalogo-productos/progress.md).
