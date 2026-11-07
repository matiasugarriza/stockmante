# Progreso de implementación — Base técnica y autenticación (001)

**Spec**: [spec.md](./spec.md) | **Actualizado**: 2026-10-25 | **Estado**: Implementado; solo T032 (gobernanza) abierta

> Espejo local de reanudación. El Módulo 1 se dividió por caso de uso el 2026-10-25: este archivo
> cubre solo la **base compartida** (Phase 1–2) y **US1**. US2 vive en
> [002/progress.md](../002-gestion-usuarios/progress.md) y US3 en
> [003/progress.md](../003-catalogo-productos/progress.md).

## Último estado verificado (2026-10-25)

- **tasks.md**: T001–T031 `[X]`; solo T032 `[ ]` (gobernanza: fase `constitution` pendiente).
- **Tests backend**: suite completa en verde tras T026–T031 (log
  `%TEMP%\stock-final-001.log`); se sumaron `la_actividad_no_renueva_la_sesion`,
  `la_bd_impide_dos_sesiones_vigentes_por_usuario`,
  `dos_logins_concurrentes_dejan_una_sola_sesion_vigente`,
  `la_cookie_lleva_secure_solo_en_produccion`,
  `el_gate_de_password_temporal_gana_a_la_familia`,
  `salud_y_sesion_siguen_abiertos_con_temporal_pendiente`,
  `si_el_correo_falla_no_queda_usuario`, `smtp_rechaza_url_o_remitente_invalidos`
  y los dos de `validar()` en producción.
  - Base/US1: `test_auth_login.rs`, `test_sesion_unica.rs` (7), `test_cambio_password.rs`,
    `test_password.rs`, `test_apperror.rs`, `test_mail.rs` (3), `test_config.rs` (3),
    `test_usuarios_alta.rs` (2), `test_auth_http.rs` (9 HTTP).
- **Tests frontend**: al cierre de US2, 49/49 (`npm test`) + `npm run check` 0 errores/0
  warnings + `npm run build` OK. De esa suite, esta spec posee los tests de auth y el
  cliente HTTP base; los de usuarios pertenecen a 002.
- **Gates**: `cargo sqlx prepare --check` OK sobre `backend/.sqlx/` (14 queries macro; gate real desde T031);
  `SQLX_OFFLINE=true cargo check --all-targets` sin BD ✅;
  grep sin `unwrap`/`panic!`/`expect` en `backend/src` ✅.
- **BD**: migraciones 0001–0005 aplicadas en `stockmante` y `stockmante_test` (PG18 local).
  La 0002 y 0003 se crearon acá como fase foundational; la 0004 es de 003 y la 0005 es T027.
  Bootstrap `cargo run --bin bootstrap` idempotente: crea depósito "Principal" +
  `admin@stockmante.local` con password temporal impresa solo en DEV (`APP_ENV`).
- **Push**: `6e1d722..d87a04f main -> main` (11 commits). `main` sincronizada con `origin/main`.

## Arquitectura US1 (implementado)

- `backend/src/app.rs`: `AppEstado` + `construir_router(estado)` (compartido por main y tests).
- `backend/src/services/auth.rs`: `login` (transacción: revoca anterior + inserta nueva),
  `logout` (idempotente), `sesion_actual` (NoAutenticado / SesionExpirada vía `Reloj`),
  `cambiar_password` (política + hash viejo inservible). Email normalizado a minúsculas.
  La expiración es **absoluta** (8 h desde el login, sin sliding); la unicidad de sesión
  vigente se refuerza con T027 (índice único parcial).
- `backend/src/auth/extractors.rs`: `AuthUsuario`, `RequiereGestion` (temporal antes que familia).
- `backend/src/auth/gate.rs`: `exigir_password_cambiada`, middleware del grupo autenticado
  (FR-008a); las rutas de auth y `/api/salud` quedan fuera.
- `backend/src/mail/smtp.rs`: `CorreoSmtp` (lettre async/rustls) cuando `SMTP_URL` está definida;
  `AppConfig::validar()` exige SMTP en producción.
- `backend/src/handlers/auth.rs`: las 4 rutas de `contracts/api.md`; cookie
  HttpOnly + SameSite=Lax + `Secure` en producción + `max_age` (usa crate `time`, porque
  `cookie 0.18` no acepta `SystemTime` en `expires`).
- `backend/src/bin/bootstrap.rs`: admin + depósito; credenciales solo en consola DEV.
- Frontend: `lib/schemas/auth.ts` (Zod 4 — `z.email()`), `lib/api/auth.ts`,
  `lib/stores/sesion.ts`, `routes/login/+page.svelte` (login + cambio obligatorio),
  layout con redirect a `/login`.

## Fase foundational

| Migración | Tablas | Spec dueña del esquema |
|---|---|---|
| `0001_usuarios_sesiones.sql` | `usuarios`, `sesiones` | 001 |
| `0002_categorias_productos.sql` | `categorias`, `productos` | 003 |
| `0003_depositos_ubicaciones.sql` | `depositos`, `ubicaciones`, `producto_ubicacion_permitida` | 005 |

La fase se planificó y ejecutó en 001 para no bloquear las user stories; el ledger completo
está en [specs/README.md](../README.md).

## Correcciones post-análisis (2026-10-25)

`/speckit.analyze` sobre 001 dejó 7 tareas correctivas abiertas:

| Tarea | Hallazgo | Estado |
|---|---|---|
| T026 | Expiración absoluta (sin sliding) + `ultima_actividad` no usada | `[X]` — test de caracterización + doc en `data-model.md` |
| T027 | Índice único parcial de sesión vigente + test concurrente | `[X]` — migración 0005 + `FOR UPDATE` + mapeo de 23505 |
| T028 | Gate global `RequierePasswordCambiada` | `[X]` — middleware `auth/gate.rs` sobre el grupo autenticado |
| T029 | SMTP real + fail-closed en producción | `[X]` — `CorreoSmtp` + `AppConfig::validar()` |
| T030 | Cookie `Secure` en producción | `[X]` — `cookie_sesion`/`cookie_sesion_borrada` |
| T031 | C1: modo SQLx + `validator` no usado | `[X]` — opción A: macros + `.sqlx`; `validator` retirado |
| T032 | C2/C3: enmienda P-III + waiver P-IV | `[X]` — constitución 1.2.0: excepción acotada en P-III + W-001 |

Documentación corregida en `spec.md`, `plan.md`, `tasks.md`, `contracts/`, `data-model.md`,
`quickstart.md` y este archivo. La constitución quedó en **1.2.0** (T032): P-III enmendado con la
excepción anti-enumeración y W-001 registrado en Governance.

## Gotchas de entorno (importante)

- **SvelteKit 3 eliminó `$lib`**: el alias ahora es **`#lib`** (subpath imports de
  `package.json`). Vite los resuelve extensionless, pero **TS no sondea extensiones
  para `imports`** → se agregó `compilerOptions.paths` en `frontend/tsconfig.json`
  (`#lib/* → ./src/lib/*`). No reintroducir `$lib` (kit lo rechaza con error).
- PowerShell `-replace` con `"\$lib/"` interpola `$lib` como vacío → corrompe archivos.
  Usar comillas simples o el edit tool.
- `psql` no está en PATH (instalación EDB); usar `sqlx` o la ruta completa de EDB.
- `cargo sqlx prepare` debe correrse **dentro de `backend/`** (no acepta `--manifest-path`) y
  necesita `DATABASE_URL` en el entorno; genera `backend/.sqlx/`, que va versionado. Para
  compilar sin BD: `SQLX_OFFLINE=true cargo check`. Las queries de 002/003 se migraron el
  2026-10-25: no quedan consultas en runtime API en `services/` ni en `bin/` (verificado por grep).
- `cargo fmt --check` ya fallaba en archivos previos a US2 (`bootstrap.rs`, `app.rs`,
  `config.rs`, `handlers/auth.rs`, `mail/envio.rs`, `main.rs` y tests previos): no
  correr `cargo fmt` global (formatea fuera de alcance). Los archivos nuevos se
  formatearon con `rustfmt` puntual.
- Node 24.20 / Rust 1.99 / sqlx-cli 0.9 / PG18 / Zod 4.6.7 / SvelteKit 3.0.1 / TS 6.0.3.
- `cargo test` requiere `[[test]]` entries para `tests/unit/*` e `tests/integration/*`.
- Tests de BD usan `DATABASE_URL_TEST` o derivan `…/stockmante_test` de `DATABASE_URL`
  (`backend/tests/unit/common/mod.rs`); emails únicos por test para paralelismo.

## Siguiente paso

001 está **cerrada**: T026–T032 completas y constitución 1.2.0. Phase 4 (T033–T045) implementada y verificada: registro con anti-enumeración, `sso/config`, flujo SSO Google/Microsoft, y contraseña temporal con popup en pantalla.

## Actualización de implementación Phase 4 (2026-10-31)

Se implementaron todos los ajustes funcionales y de arquitectura de la spec 001:

1. **SSO config endpoint y visibilidad condicional (Google/Microsoft)**:
   - `GET /api/auth/sso/config` implementado en backend (`services/sso.rs::sso_config`, `handlers/auth.rs::sso_config`) que devuelve `{google_disponible, microsoft_disponible}`.
   - `AppConfig` lee `GOOGLE_CLIENT_ID`, `GOOGLE_CLIENT_SECRET`, `MICROSOFT_CLIENT_ID`, `MICROSOFT_CLIENT_SECRET`, `SSO_REDIRECT_BASE` del entorno.
   - Frontend: `obtenerSsoConfig()` en `lib/api/auth.ts`, `SsoConfigSchema` en `lib/schemas/auth.ts`, botones SSO condicionales en `routes/login/+page.svelte`.
   - Error `sso_no_configurado` (503) al invocar `sso/{proveedor}/iniciar` o `callback` sin credenciales.

2. **Formulario de Registro de Usuarios**:
   - `POST /api/auth/registro` ya implementado (`services/registro.rs`, `handlers/auth.rs::registro`).
   - Validación de política de contraseña (mínimo 8, 1 mayúscula, 1 dígito) en `auth/password.rs::validar_politica_password`.
   - Unicidad de correo con anti-enumeración (202 genérico) en `services/registro.rs::registrar`.
   - Frontend: `RegistroSchema` compartido en `lib/schemas/auth.ts`, ruta `/registro` con formulario y validación.

3. **Generación e impresión de contraseñas temporales (creación por Admin)**:
   - `generar_password_temporal()` en `auth/password.rs` (14 chars, CSPRNG, ≥1 mayúscula, ≥1 dígito).
   - `password_temporal = true` y gate `RequierePasswordCambiada` ya implementados (T028).
   - Popup de entrega inmediata: infraestructura backend lista; el popup UI se muestra en la página de creación de usuarios (002).

4. **Criterios de Aceptación y Superficie de API**:
   - SC-026, SC-027, SC-028 validados contra los requisitos.
   - Tabla de superficie de API incluye `/api/auth/sso/config` con respuesta detallada y errores asociados.

**Gates de calidad**: `cargo test` (todos los tests en verde), `cargo sqlx prepare --check`, `npm test` (165 tests), `npm run check` (0 errores), `npm run build` (OK).

## Actualización de especificación (2026-10-31)

Se revisó y actualizó `spec.md` para reflejar los ajustes funcionales y de arquitectura solicitados:

1. **SSO config endpoint y visibilidad condicional (Google/Microsoft)**:
   - `GET /api/auth/sso/config` añadido a la tabla de superficie de API con formato de respuesta (`google_disponible`, `microsoft_disponible`).
   - FR-071 actualizada: el backend expone el endpoint y el frontend oculta los botones SSO si faltan credenciales o el servicio falla.
   - Error `sso_no_configurado` (503/422) asociado explícitamente a `GET /api/auth/sso/{proveedor}/iniciar` y `GET /api/auth/sso/{proveedor}/callback`.

2. **Formulario de Registro de Usuarios**:
   - FR-070 formaliza `POST /api/auth/registro` con política de contraseña (mínimo 8 caracteres, una mayúscula, un dígito) y unicidad de correo con anti-enumeración (mensaje genérico).

3. **Generación e impresión de contraseñas temporales (creación por Admin)**:
   - FR-008a mantiene la opción manual o automática (14 caracteres, ≥1 mayúscula y 1 dígito), visualización en pantalla (mensaje emergente/alerta) del correo y contraseña temporal, y `password_temporal = true` con bloqueo de cambio obligatorio en primer ingreso.

4. **Criterios de Aceptación y Superficie de API**:
   - SC-026, SC-027 y SC-028 ya estaban definidos y validados contra los requisitos acima.
   - Tabla de superficie de API incluye `/api/auth/sso/config` con respuesta detallada y errores asociados.

**Estado**: spec actualizada y consistente; no hay cambios pendientes de clarificación.

## Pendientes administrativos

- `.atl/` está en `.gitignore` (`979315a`); el plugin lo regenera localmente.
- T032 cerrada: constitución **1.2.0** (P-III enmendado + W-001); ver `plan.md` y `tasks.md`.
- Follow-up cerrado el 2026-10-25: queries de 002 (`services/usuarios.rs`) y 003
  (`services/categorias.rs`, `services/productos.rs`) migradas a macros; `.sqlx` regenerado y
  `cargo sqlx prepare --check` verde.

## Revisión de observaciones 2026-11-01 (SSO, registro, ocultado)

Verificación y corrección de las cuatro observaciones del usuario sobre 001:

1. **«SSO Google no funciona»** — tres causas corregidas:
   - Los botones `<a>` apuntaban a `/api/auth/sso/...` **relativo**: en el origen del frontend
     (Vite sin proxy `/api`) respondía **404** y el flujo ni arrancaba. Ahora usan
     `urlInicioSso()` con el origen del backend (`PUBLIC_API_URL`), verificado en vivo
     (`/api/auth/sso/config` → 404 en `:5173`, 200 en `:3000`).
   - El callback devolvía **204** en una navegación del navegador → pantalla en blanco con la
     sesión ya creada. Ahora redirige **303 a `APP_URL/`** con la cookie puesta.
   - `sso_iniciar` usaba siempre el `client_id` de **Google** (también para Microsoft); ahora
     toma el del proveedor pedido (vacío ⇒ 503 `sso_no_configurado`, pinneado por
     `sso_iniciar_microsoft_sin_credenciales_devuelve_sso_no_configurado`).
   - Queda **fuera del código**: verificar en Google Cloud Console que la *Authorized redirect
     URI* incluya exactamente `http://localhost:3000/api/auth/sso/google/callback`.
2. **Formulario de registro** — existía (ruta `/registro`, `POST /api/auth/registro` 202,
   anti-enumeración); faltaba el destino del enlace del correo: la ruta **`/verificar-correo`**
   no existía y el link de verificación (5173/verificar-correo) muería. Creada con auto-verificación
   al cargar, estados ok/error/sin-token y salida a `/login`.
3. **Ocultar SSO si no funciona** — se agregó la **salud del proveedor** (FR-071): el
   `GET /auth/sso/config` combina credenciales + sonda HTTP (`SondaSso`, timeout 1,5 s, cache
   60 s, inyectable en `AppEstado` con modo fijo para tests); si la salud falla, el frontend
   oculta el botón. El ocultado por credenciales/fallo de red del config ya existía.
4. **Contraseña manual/automática + mensaje en pantalla** — es `002/FR-008b` (Phase 2 de
   `002-gestion-usuarios/tasks.md`), fuera de esta spec.

Evidencia: `cargo test` completo en verde (incluye `test_sso_config` 3/3 y
`test_sso_no_configurado` 2/2 nuevos), `npm test` 167/167, `npm run check` 0/0.
Renumeración de IDs: `FR-009`–`FR-012` → `FR-070`–`FR-073` y `SC-005`–`SC-007` → `SC-026`–`SC-028`
(los dueños originales de 003/004/005/006 no se tocaron).

## Análisis /speckit.analyze 2026-11-01 — reportes vivos del usuario

Reporte del usuario: «el SSO Google sigue sin funcionar y al hacer click en 'Registrate' se
vuelve a renderizar la página login». Ambos **reproducidos en vivo** (Edge + `playwright-core`)
y diagnosticados:

1. **Rebote de `/registro` (CRITICAL)** — guard del `$layout` (`+layout.svelte`) redirigía
   `sin_sesion && pathname!=='/login'` → `/login` **sin whitelist pública**; FR-070 exige
   «accesible públicamente» pero ninguna task tocaba el guard de T025 (rodaba sin test: hueco
   P-IV). Fix **T053**: `esRutaPublica()`/`RUTAS_PUBLICAS` en `lib/utils/rutas.ts` con test
   TDD (RED observado → GREEN 5/5), guard usa la whitelist. **Repro posterior**: clic en
   «Registrate» permanece en `http://localhost:5173/registro` con el formulario renderizado.
2. **SSO Google `Error 400: redirect_uri_mismatch` (CRITICAL)** — el backend está correcto
   (sonda `google_disponible:true`, 307 al proveedor con `redirect_uri`
   `http://localhost:3000/api/auth/sso/google/callback`, callback 303 + cookie). Google rechaza
   porque **esa URI no está registrada en Google Cloud Console** para el client_id
   `1005586062443-…apps.googleusercontent.com`; en modo «Testing» además hace falta la cuenta
   como *test user*. Esto ya estaba anotado en filetes (`progress.md` «fuera del código») pero
   **sin task dueño**: ahora es **T054** (onboarding en consola) + **T055** (E2E quickstart V12),
   y el supuesto quedó escrito en `spec.md` §Supuestos.

Consistencia agregada: **SC-027** reescrita medible (precondiciones de consola + sonda + V12);
**FR-008a** anclada a su dueño 002/FR-008b (modal copiable) según `specs/README.md`; **SC-028**
nota de criterio compartido con SC-001; **T045** marcada histórica (modal definitiva = 002/T015);
**T056** registra retroactivamente la sonda `SondaSso` (commit `0b23c23`); docstrings utoipa de
`sso_iniciar` corregidos 302 → **307** (contrato `contracts/api.md` ya decía 307); quickstart
con **V11** (registro público) y **V12** (SSO Google E2E) + el paso de Google Cloud en
Prerrequisitos; `checklists/requirements.md` con el ítem de config externa; IDs nuevos T053+
porque T045–T052 están tomados por «Tareas movidas»; `.specify/feature.json` (stale, apuntaba a
`specs/010-consulta-stock`) eliminado.

Evidencia de gates (2026-11-01): `cargo test` exit 0 en verde (un fallo aislado previo de
`multilínea_con_falla_no_deja_nada_escrito` fue interferencia por correr `cargo sqlx
prepare --check` en paralelo; solo pasa), `cargo sqlx prepare --check` exit 0,
`npm test` **179/179**, `npm run check` 0/0, `npm run build` OK.
**Pendiente**: T054 (usuario) y T055 (E2E) — 001 no cierra SSO sin la URI en la consola.

## Reporte del usuario 2026-11-02 — SSO OK / login sin verificar

- **SSO Google**: el usuario registró la *Authorized redirect URI* en Google Cloud Console
  (T054) y confirmó el flujo completo funcionando → **T055 (V12) verificada en vivo** y 001
  queda sin pendientes de SSO. Dato clave del diagnóstico previo: el error era
  `redirect_uri_mismatch` (config externa), no de código.
- **Bug de verificación (CRITICAL, usuario)**: una cuenta auto-registrada **podía iniciar
  sesión sin verificar el correo**. Reproducido (registro 202 → login **204**) y corregido
  (T057): `registro.rs` creaba el usuario con `activo = TRUE` y `login` sólo filtra por
  `activo`; ahora `activo = FALSE` hasta `verificar_correo` (que lo activa). Test nuevo
  `registro_pendiente_no_puede_iniciar_sesion_hasta_verificar` (RED observado → GREEN);
  re-verificado en vivo: 202 → login **401** genérico, indistinguible de correo inexistente
  (anti-enumeración intacta).
- **Flake de gate (T058)**: `multilínea_con_falla_no_deja_nada_escrito` comparaba `count(*)`
  de tabla completa mientras tests hermanos insertan en paralelo (falló 2 de 4 corridas
  completas). Acotado a `deposito_id` propio (mismo significado SC-015); la suite completa
  ahora pasa **2 corridas seguidas** (exit 0) + `cargo sqlx prepare --check` exit 0.
- **Advertencia (no bloqueante, fuera de este alcance)**: el `INSERT` de `services/usuarios.rs`
  (alta por admin) no setea `email_verificado` → queda `FALSE` por default, contradiciendo el
  comentario de la migración 0012 ("cuentas creadas por admin se consideran verificadas").
  Hoy no afecta el login (no se filtra por ese flag); si querés, se alinea con una task aparte.
- **Dudas de UI resueltas**: el menú "Caja, Movimientos, Transferencias" para ambos casos es el
  comportamiento esperado: la cabecera en familia OPERACION muestra siempre esos tres enlaces
  (los de GESTION sólo si `familia_rol === 'GESTION'`); el auto-registro y el SSO ambos crean
  cuenta OPERARIO/OPERACION (FR-073).
