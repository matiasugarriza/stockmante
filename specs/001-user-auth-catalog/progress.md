# Progreso de implementación — Módulo 1 (espejo local de reanudación)

> Engram quedó temporalmente no disponible al cerrar la sesión anterior
> (`could not resolve an authoritative OpenCode runtime session`).
> Este archivo es el espejo local para retomar; sincronizar con Engram al iniciar la próxima sesión.

## Último estado verificado (2026-10-25)

- **tasks.md**: T001–T031 `[X]` (Phase 1 + Phase 2 + **Phase 3 US1** + **Phase 4 US2 completas**).
- **Tests backend**: 40/40 (`cargo test`) — 13 integration HTTP + 27 unit.
  - US1: `test_auth_login.rs`, `test_sesion_unica.rs`, `test_cambio_password.rs`,
    `test_password.rs`, `test_apperror.rs`, `test_mail.rs`,
    `test_auth_http.rs` (5 HTTP).
  - US2: `test_usuarios_email.rs` (3), `test_usuarios_alta.rs` (1),
    `test_ultimo_admin.rs` (1), `test_familia_rol.rs` (3),
    `test_usuarios_http.rs` (8 HTTP).
- **Tests frontend**: 37/37 (`npm test`) + `npm run check` 0 errores/0 warnings + `npm run build` OK.
  - US2: `schemas/usuarios.test.ts` (9), `api/usuarios.test.ts` (9).
- **Gates**: `cargo sqlx prepare --check` OK (sin macros `query!`; queries en runtime API);
  grep sin `unwrap`/`panic!`/`expect` en `backend/src` ✅.
- **BD**: migraciones 0001–0003 en `stockmante` y `stockmante_test` (PG18 local, es-AR).
  Bootstrap `cargo run --bin bootstrap` idempotente: crea depósito "Principal" +
  `admin@stockmante.local` con password temporal impresa solo en DEV (`APP_ENV`).
- **Commits previos**: `e2f2705` (phase1), `4adbab7` (phase2), `59bb4b1` (frontend client),
  `962ce0d` (US1), `f8358fa` (`default-run`).

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
- Advisories pendientes (fuera de alcance US2): la UI lista hasta 100 usuarios sin
  paginador; `RequiereGestion` no bloquea usuarios con `password_temporal=true`
  (gap preexistente de US1); `CorreoDev` no imprime la temporal en consola DEV;
  rechazos de extractores Axum (JSON/Path/Query mal formados) no usan el sobre
  `{error:{codigo,mensaje}}` (preexistente).

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

- `.atl/` sigue untracked (lo genera el plugin `skill-registry`): decidir `.gitignore`
  vs commitear el registro.
- Los 2 commits locales de US1 (`962ce0d`, `f8358fa`) siguen sin push a `origin/main`.

## Notas de entorno

- Node 24.20 / Rust 1.99 / sqlx-cli 0.9 / PG18 / Zod 4.6.7 / SvelteKit 3.0.1 / TS 6.0.3.
- `cargo test` requiere `[[test]]` entries para `tests/unit/*` e `tests/integration/*`.
- Tests de BD usan `DATABASE_URL_TEST` o derivan `…/stockmante_test` de `DATABASE_URL`
  (`backend/tests/unit/common/mod.rs`); emails únicos por test para paralelismo.
- Passwords de BD: solo las conoce el usuario (`.env` local, gitignored).
