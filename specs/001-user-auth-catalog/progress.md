# Progreso de implementación — Módulo 1 (espejo local de reanudación)

> Engram disponible de nuevo: la arquitectura US2 quedó en la observación 20 y el
> resumen de sesión en la 21. Este archivo sigue siendo el espejo local para retomar.

## Último estado verificado (2026-10-25)

- **tasks.md**: T001–T031 `[X]` (Phase 1 + Phase 2 + **Phase 3 US1** + **Phase 4 US2 completas**).
- **Tests backend**: 45/45 (`cargo test`) — 16 integration HTTP + 29 unit.
  - US1: `test_auth_login.rs`, `test_sesion_unica.rs`, `test_cambio_password.rs`,
    `test_password.rs`, `test_apperror.rs`, `test_mail.rs` (2), `test_config.rs` (1),
    `test_auth_http.rs` (6 HTTP).
  - US2: `test_usuarios_email.rs` (3), `test_usuarios_alta.rs` (1),
    `test_ultimo_admin.rs` (1), `test_familia_rol.rs` (3),
    `test_usuarios_http.rs` (10 HTTP).
- **Tests frontend**: 49/49 (`npm test`) + `npm run check` 0 errores/0 warnings + `npm run build` OK.
  - US2: `schemas/usuarios.test.ts` (9), `api/usuarios.test.ts` (9).
  - Paginación: `utils/paginacion.test.ts` (12).
- **Gates**: `cargo sqlx prepare --check` OK (sin macros `query!`; queries en runtime API);
  grep sin `unwrap`/`panic!`/`expect` en `backend/src` ✅.
- **BD**: migraciones 0001–0003 en `stockmante` y `stockmante_test` (PG18 local, es-AR).
  Bootstrap `cargo run --bin bootstrap` idempotente: crea depósito "Principal" +
  `admin@stockmante.local` con password temporal impresa solo en DEV (`APP_ENV`).
- **Commits**: `e2f2705` (phase1), `4adbab7` (phase2), `59bb4b1` (frontend client),
  `962ce0d` (US1), `f8358fa` (`default-run`), `f1fc3e8` (US2),
  `979315a` (`.gitignore`), `977bd90` (temporal + extractores),
  `5a0b971` (mailer dev), `8fa8390` (paginador).

## Arquitectura US1 (implementado)

- `backend/src/app.rs`: `AppEstado` + `construir_router(estado)` (compartido por main y tests).
- `backend/src/services/auth.rs`: `login` (transacción: revoca anterior + inserta nueva),
  `logout` (idempotente), `sesion_actual` (NoAutenticado / SesionExpirada vía `Reloj`),
  `cambiar_password` (política + hash viejo inservible). Email normalizado a minúsculas.
- `backend/src/auth/extractors.rs`: `AuthUsuario`, `RequiereGestion` (listo para US2).
- `backend/src/handlers/auth.rs`: las 4 rutas de `contracts/api.md`; cookie
  HttpOnly + SameSite=Lax + `max_age` (usa crate `time`, porque `cookie 0.18` no acepta
  `SystemTime` en `expires`).
- `backend/src/bin/bootstrap.rs`: admin + depósito; credenciales solo en consola DEV.
- Frontend: `lib/schemas/auth.ts` (Zod 4 — `z.email()`), `lib/api/auth.ts`,
  `lib/stores/sesion.ts`, `routes/login/+page.svelte` (login + cambio obligatorio),
  layout con redirect a `/login`.

## Arquitectura US2 (implementado)

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
  `usuarios_email_key` a `EmailDuplicado`. El email se normaliza (trim + minúsculas)
  en `models::usuario::normalizar_email` (reutilizado por `services::auth`).
- `backend/src/handlers/usuarios.rs`: router anidado en `/api/usuarios`; todas las
  rutas con `RequiereGestion`. GET lista devuelve `{ items, total, pagina, por_pagina }`
  (default 20, máx 100). El ítem es `models::usuario::Usuario` (sin `password_hash`).
- Frontend: `lib/schemas/usuarios.ts` (Zod), `lib/api/usuarios.ts`,
  `lib/components/usuarios/UsuarioForm.svelte` + `UsuarioTabla.svelte`,
  `routes/admin/usuarios/+page.svelte`; link "Usuarios" en el layout solo para GESTION.
  Los errores de guardado del modal se muestran **dentro** del modal (el overlay tapa
  el banner de la página).
- Decisión: no existe `DELETE` físico de usuarios (solo `activo = FALSE`). El alta
  devuelve 201 con el usuario público; la temporal nunca viaja en la respuesta.

## Verificación independiente US2 (2026-10-25)

- Probes del verificador en verde: 38 backend (antes del cierre), 37 frontend,
  `cargo sqlx prepare --check`, `npm run check` 0/0.
- **Blocker corregido**: `validar_email` aceptaba múltiples `@` (`split_once('@')`).
  Ahora rechaza 422 y quedó pinneado por
  `test_usuarios_http.rs::email_con_multiples_arrobas_es_422_en_alta_y_put`
  (POST y PUT, 8 variantes inválidas, verifica que nada se persista).
- Advisories corregidos: deadlock de bajas concurrentes (orden de locks), error del
  modal invisible, huecos de test (401 sin sesión, 403 en POST/PUT de OPERACION,
  `activo=all`, límites de paginación y overflow del offset).
- Advisories que quedaron fuera del alcance US2 (paginador, FR-008a, mailer dev,
  extractores): cerrados en la sección siguiente.

## Pendientes técnicos cerrados (2026-10-25)

Los cuatro advisories que quedaron fuera del alcance US2 ya están resueltos y
pinneados con tests:

1. **Paginador admin** (`8fa8390`): `frontend/src/lib/utils/paginacion.ts`
   (`totalPaginas`, `paginaValida`); `routes/admin/usuarios/+page.svelte` pagina
   de 20 en 20, reinicia a la página 1 al cambiar de filtro y recorta a la última
   página válida si una baja o un filtro vacía la actual.
2. **FR-008a en backend** (`977bd90`): `AppError::DebeCambiarPassword` (403
   `cambio_password_requerido`); `RequiereGestion` lo devuelve mientras
   `password_temporal` siga vigente. `/auth/sesion` y `/auth/cambiar-password`
   usan `AuthUsuario`, así que la sesión alcanza para completar el cambio. El
   test cubre las **7 rutas** de gestión y verifica que ninguna acción rechazada
   persista cambios (alta, fila objetivo intacta).
3. **Mailer dev** (`5a0b971`): `AppConfig` incorpora `APP_ENV` y
   `es_produccion()` (trim + case-insensitive); `main` usa
   `CorreoDev::para_desarrollo()` fuera de producción y el stub silencioso en
   producción. `CorreoDev::aviso_credenciales()` es la superficie testeable; el
   log de `tracing` nunca incluye la credencial. `bootstrap` ahora usa el mismo
   predicado (`AppConfig::es_produccion`) en lugar de comparar `APP_ENV` por su
   cuenta. Decisión: es una ayuda local equivalente a la del bootstrap, no una
   excepción a la regla de producción. `.env.example` documenta `APP_ENV` para
   que cada entorno lo defina explícitamente (si falta se asume development).
4. **Sobre de error de extractores** (`977bd90`): `backend/src/extractores.rs`
   con `JsonValido<T>`, `PathValido<T>` y `QueryValida<T>`; los rechazos de Axum
   (antes 400/415 con texto plano) ahora son 422 `validacion` con
   `{error:{codigo,mensaje}}`. Aplicado en `handlers/auth.rs` y
   `handlers/usuarios.rs`.

Evidencia: 45/45 backend, 46/46 frontend, `cargo sqlx prepare --check`,
`npm run check` 0/0 y `npm run build` OK. Tests nuevos que lo pinnean:
`gestor_con_password_temporal_no_puede_gestionar_hasta_cambiarla`,
`rechazos_de_extractores_usan_el_sobre_de_error`,
`login_con_json_invalido_usa_el_sobre_de_error`,
`correo_dev_solo_expone_la_temporal_en_modo_desarrollo`,
`solo_production_se_considera_produccion` y `utils/paginacion.test.ts`.

## Verificación independiente de pendientes (2026-10-25)

- Veredicto: **aprobado con advisories, 0 blockers** sobre `f1fc3e8..HEAD`.
- Probes: 45/45 backend, 46/46 frontend (al momento de verificar),
  `cargo sqlx prepare --check`, `npm run check` 0/0, `npm run build` OK; bootstrap
  real en producción sin `password:` y en dev con `password:`.
- FR-008a: las 7 rutas de gestión dan 403 `cambio_password_requerido` con la
  temporal; `/auth/sesion` y `/auth/cambiar-password` siguen accesibles; tras el
  cambio la gestión responde 200/201/204. OPERACION+temporal conserva 403
  `sin_permisos` (orden de checks preservado).
- Contrato: 12 rechazos (JSON roto/faltante, sin content-type, path y query
  inválidos, offset extremo) → 422 `validacion` con el sobre exacto; fila,
  sesiones y conteo de usuarios intactos.
- Credenciales: `CorreoDev::new()` silencioso; `para_desarrollo()` imprime;
  `APP_ENV` decide; el log de `tracing` no contiene la temporal.
- Advisories aceptados:
  - **A1** (causado por el cambio): el gating es fail-open (`APP_ENV` ausente ⇒
    development) y `.env.example` no lo documentaba. Mitigado documentando
    `APP_ENV=development`; un despliegue debe definir `APP_ENV=production`.
  - **A2** (preexistente): sin `RUST_LOG`, `tracing_subscriber` filtra a ERROR y
    oculta el WARN de credenciales en producción.
  - **A3** (preexistente): no hay SMTP real; en producción el stub es silencioso
    y la temporal no se envía (limitación conocida del módulo).
  - **A4** (preexistente): `bandeja()` retiene la temporal en memoria; no hay
    ruta que la exponga.
  - **A5** (preexistente): `cambiarFiltro` no está guardado por `cargando`
    (race de clics rápidos, idéntico al baseline).
  - **A6**: `.gitignore` agrega `.atl/` (benigno, ajeno a los pendientes).
- Regresiones incorporadas del verificador: 7 rutas FR-008a con invariante de
  fila intacta, conteo de usuarios tras rechazos, mapeo directo de
  `DebeCambiarPassword`, bordes y no finitos de paginación.

## Gotchas de entorno (importante)

- **SvelteKit 3 eliminó `$lib`**: el alias ahora es **`#lib`** (subpath imports de
  `package.json`). Vite los resuelve extensionless, pero **TS no sondea extensiones
  para `imports`** → se agregó `compilerOptions.paths` en `frontend/tsconfig.json`
  (`#lib/* → ./src/lib/*`). No reintroducir `$lib` (kit lo rechaza con error).
- PowerShell `-replace` con `"\$lib/"` interpola `$lib` como vacío → corrompe archivos.
  Usar comillas simples o el edit tool.
- `psql` no está en PATH (instalación EDB); usar `sqlx` o la ruta completa de EDB.
- `cargo sqlx prepare --check` pasa sin `.sqlx/` porque no se usan macros `query!`
  (runtime API de sqlx con SQL literal).
- `cargo fmt --check` ya fallaba en archivos previos a US2 (`bootstrap.rs`, `app.rs`,
  `config.rs`, `handlers/auth.rs`, `mail/envio.rs`, `main.rs` y tests previos): no
  correr `cargo fmt` global (formatea fuera de alcance). Los archivos nuevos de US2
  se formatearon con `rustfmt` puntual.

## Siguiente paso

Phase 5 — US3 Catálogo de productos y categorías (T032–T036):

1. Tests RED: `test_sku.rs` (normalización + regex) y `test_producto_unicidad.rs`
   (SKU/código de barras duplicados → 409).
2. `services/categorias.rs` + `services/productos.rs`: CRUD, SKU normalizado,
   mapeo `23505` → `SkuDuplicado`/`CodigoBarrasDuplicado`, DELETE solo sin referencias
   (`ProductoConReferencias`), baja lógica.
3. `handlers/productos.rs` + `handlers/categorias.rs` (escritura con `RequiereGestion`;
   lectura de detalle autenticada).
4. Frontend: schemas + cliente + rutas `admin/productos` y `admin/categorias`.

Luego: US4 → US5 → US6 → Polish (README T048, gates T049, V1–V8 T050).

## Pendientes administrativos

- `.atl/` ya está en `.gitignore` (`979315a`); el plugin lo regenera localmente.
- Los 11 commits locales (`962ce0d` en adelante) siguen sin push a `origin/main`.

## Notas de entorno

- Node 24.20 / Rust 1.99 / sqlx-cli 0.9 / PG18 / Zod 4.6.7 / SvelteKit 3.0.1 / TS 6.0.3.
- `cargo test` requiere `[[test]]` entries para `tests/unit/*` e `tests/integration/*`.
- Tests de BD usan `DATABASE_URL_TEST` o derivan `…/stockmante_test` de `DATABASE_URL`
  (`backend/tests/unit/common/mod.rs`); emails únicos por test para paralelismo.
- Passwords de BD: solo las conoce el usuario (`.env` local, gitignored).
