# Implementation Plan: Gestión de usuarios (002)

**Spec**: [spec.md](./spec.md) | **Creado**: 2026-10-25 (retro-documentado) | **Actualizado**: 2026-11-01

**Estado**: Fase 1 (CRUD, baja lógica, último gestor, paginación) **implementada**; Fase 2 —
`FR-008b`: elección manual/automática de contraseña y visualización de credenciales en pantalla —
**planificada** en Phase 2 de [tasks.md](./tasks.md).

**Input**: Feature specification from `/specs/002-gestion-usuarios/spec.md`

## Summary

CRUD de usuarios para la familia GESTION con baja lógica, guarda del último gestor activo y
paginación, implementado en Fase 1 (`f1fc3e8` y follow-ups).

Fase 2 agrega la operativa de credenciales que el ajuste 2026-11 trajo a la spec: el
Administrador elige en el alta (y en el regenerado de contraseña) entre una clave **manual** o una
**automática**; el sistema devuelve en claro las credenciales para que la interfaz las muestre de
inmediato en pantalla (modal con botón «copiar»), además de enviarlas por correo. `Migraciones:
ninguna — el cambio es de contrato y de UI, no de esquema.`

## Technical Context

**Language/Version**: Rust 1.99 (backend), TypeScript 6 / SvelteKit 3 (frontend).

**Primary Dependencies**: Axum 0.7+, SQLx 0.8 (macros `query!`/`query_as!`/`query_scalar!`,
compile-time checked), tokio, serde, argon2, tower-http, Zod 4, Tailwind CSS.

**Storage**: PostgreSQL 18; tablas `usuarios` y `sesiones` (propiedad de 001). **Sin migración nueva.**

**Acceso a datos**: consultas SQLx con macros compile-time checked (Principio IV y «Stack fijo»
de la constitución) y `backend/.sqlx` versionado para builds offline (`SQLX_OFFLINE=true`).
Fase 2 no toca SQL ⇒ `cargo sqlx prepare --check` se mantiene sin diferencias.

**Testing**: `cargo test` (unit + integration con `stockmante_test`), vitest, `cargo sqlx
prepare --check`, `npm run check`, `npm run build`.

**Constraints**: sin `unwrap`/`panic`/`expect` en rutas de producción; `AppError` con 409/422/403/404;
RBAC por `RequiereGestion`; sin `DELETE` físico de usuarios; `password_hash` nunca sale del backend.

**Scale/Scope**: 10–50 usuarios; el texto plano de la clave temporal solo existe en la respuesta
201/200 de alta/regeneración frente a una sesión GESTION.

## Constitution Check

| Gate (Principio) | Estado | Evidencia |
|---|---|---|
| I — Simplicidad / YAGNI | ✅ PASS | Sin tablas nuevas ni migración; un campo opcional `password` en el DTO; sin enum de método: la presencia del campo comunica la elección D1. |
| II — Mantenibilidad | ✅ PASS | Selección/validación de clave compartida en `services/usuarios.rs` (alta y regenerado llaman la misma función); modal de credenciales como componente único reutilizado por ambos flujos. |
| III — Claridad en Mensajes | ✅ PASS | 422 en español para clave manual fuera de política (mensaje de `validar_politica_password`); textos de la modal en español. |
| IV — Testabilidad Primero | ✅ PASS | RED observado antes de la implementación en T008–T010 (unit + HTTP); vitest para Zod/cliente. |
| V — Integridad de Datos / Errores | ✅ PASS (con nota) | **Clave en claro en respuestas**: exigido por FR-008b; existe solo en las envolturas 201/200 de las rutas GESTION, nunca en listados/detalles, nunca en logs ni persistida. Email del alta sigue fall-closed dentro de la transacción (decisión D4 conservada de Fase 1). |

**Resultado post-diseño**: PASS — sin violaciones nuevas.

## Project Structure

```text
backend/
├── src/services/usuarios.rs        # crear y regenerar ahora aceptan password: Option<&str>
│                                   # y devuelven la clave en claro generada (para la modal)
├── src/handlers/usuarios.rs        # POST /usuarios 201 { usuario, password_temporal };
│                                   # POST /{id}/password-temporal 200 + body opcional
├── src/models/usuario.rs           # UsuarioCreado (envoltura de credenciales), sin cambios
│                                   # en el modelo item
└── tests/
    ├── unit/{...test_usuarios_alta, test_usuarios_password}.rs
    └── integration/test_usuarios_http.rs

frontend/src/
├── lib/schemas/usuarios.ts         # + password opcional con política, UsuarioCreadoSchema
├── lib/api/usuarios.ts             # crearUsuario y regenerarPasswordTemporal devuelven credenciales
├── lib/components/usuarios/
│   ├── UsuarioForm.svelte          # + selector manual/automática en el alta
│   ├── PasswordTemporalModal.svelte  # (nuevo) diálogo de regeneración con mismo selector
│   └── CredencialesModal.svelte      # (nuevo) correo + clave con botón «copiar»
└── routes/admin/usuarios/+page.svelte  # abre CredencialesModal tras alta y tras regenerar
```

## Fase 1 — implementada (referencia)

Complete el CRUD original; ver el historial de git para el detalle fino. Núcleo:

- **`familia_de_rol`**: única fuente del mapa `ADMIN|GERENTE → GESTION`, `OPERARIO|CAJERO → OPERACION`.
- **`crear`**: temporal + `EnvioCorreo` dentro de la transacción (fail-closed: si el envío falla,
  no queda cuenta huérfana). Devolvía 201 con el usuario público.
- **`listar`**: paginado (`pagina >= 1`, `1 <= por_pagina <= 100`), filtro `activo`, offset con
  `checked_mul` (extremos → 422).
- **`desactivar`**: baja lógica + revocación de sesiones; `bloquear_gestores_activos` con `FOR
  UPDATE` ordenado por `id` antes de la fila objetivo (sin deadlock `40P01`); último gestor → 409.
- **Unicidad**: violación `23505` → `EmailDuplicado` (409), sin pre-chequeo TOCTOU.
- **Handlers**: todas las rutas bajo `RequiereGestion`; `DebeCambiarPassword` (403) mientras la
  temporal siga vigente.
- **Frontend**: schemas Zod, cliente tipado, tabla + modal; errores dentro del modal; paginación.

## Fase 2 — decisiones de diseño

| # | Decisión | Alternativa rechazada | Por qué |
|---|---|---|---|
| D1 | `CrearUsuarioDto.password: Option<String>`; body opcional `{ password?: string }` en regenerar. Ausente/vacío ⇒ automática (14 chars); presente ⇒ manual con política de 001. | Enum explícito `{ metodo: "manual"\|"automatica" }` | YAGNI: la presencia del campo ya codifica la elección; el selector de la UI traduce al contrato sin estado redundante que sincronizar. |
| D2 | Respuesta 201/200 **envuelta**: `{ usuario: {...}, password_temporal: "..." }` (modelo nuevo `UsuarioCreado`). | Devolver la clave dentro del item `usuario` | El item `usuario` lo consumen listado y detalle; poner la clave ahí ampliaría la superficie de exposición y rompería el «nunca en claro» de cualquier GET. |
| D3 | `POST /usuarios/{id}/password-temporal`: de 204→**200** con body `{ password_temporal }`, aceptando `{ password? }` opcional. | Mantener 204 + endpoint paralelo | El único consumidor es el frontend de 002; un endpoint extra duplicaría reglas. Es un cambio de contrato deliberado y registrado en `contracts/api.md`. |
| D4 | El correo sigue enviándose **dentro de la transacción** con fail-closed (500) ante fallo del transporte configurado. | Crear igual y confiar solo en la modal | Conserva el invariante de Fase 1 («alta sin credenciales entregadas no queda huérfana»); la spec dice «**adicionalmente**, se enviará por el transporte configurado». |
| D5 | La clave manual pasa por `validar_politica_password` de 001 (misma regla que satisface la automática). | Política propia duplicada | Una sola fuente de verdad de la política (propiedad de 001). |
| D6 | Alta y regenerado comparten la misma rutina «elegir clave (manual/automática) + validar + hashear + correo». | Dos caminos independientes | Escenario 5 y escenario 1 son el mismo flujo operado sobre distinto usuario. |
| D7 | La clave en claro no se imprime en ningún `tracing`/log; solo llega a la respuesta que la modal pide. | Registrar el alta con la clave | Principio V + supuesto de la spec (credenciales en claro no van a logs; ese caso borde vive en 001). |
| D8 | `ActualizarUsuarioSchema` del frontend deja de derivarse del schema de alta (omite `password`); `PUT` no recibe claves. | Permitir password en PUT | No cambiar la semántica existente sin que la spec lo pida. |

### Contrato resultante (resumen; detalle en `contracts/api.md`)

- `POST /api/usuarios` → **201** `{ usuario, password_temporal }`; 409 `email_duplicado`;
  422 `validacion` (datos o clave manual débil); 403 `sin_permisos`.
- `POST /api/usuarios/{id}/password-temporal` con `{ password? }` opcional → **200**
  `{ password_temporal }`; revoca sesiones; envía correo; 404 / 422.
- Los GET siguen devolviendo **solo** el item `usuario`, nunca la clave en claro.

## Estrategia de pruebas

| Capa | Enfoque |
|---|---|
| Dominio (RED→GREEN) | Clave manual válida hasheada y devuelta igual; clave manual débil → 422; automática → 14 chars con mayúscula y dígito; correo enviado en ambos modos |
| Handlers | 201 con `{ usuario, password_temporal }`; 200 en regenerar con y sin body; 422/409/403/404/401 sin cambios; **GET /usuarios no contiene la clave en claro** |
| Frontend | Zod con/sin `password` débil; cliente devuelve credenciales; modal renderiza correo+clave |
| SQLx | `cargo sqlx prepare --check` sin diferencias (sin tocar queries) |
| Regresión Fase 1 | Suites `test_usuarios_alta`, `test_ultimo_admin`, `test_familia_rol`, `test_usuarios_http` actualizadas para la firma nueva y verdes |

## Commits de referencia

| Commit | Contenido |
|---|---|
| `f1fc3e8` | US2 completa: servicio, handlers, frontend y tests (Fase 1) |
| `977bd90` | `DebeCambiarPassword`, sobre de error de extractores, validación de email endurecida |
| `8fa8390` | Paginación del listado de usuarios |
| `2878b4b` | Regresiones de alto riesgo de los pendientes |
| `67032c7` | Migración de `services/usuarios.rs` a macros SQLx y regeneración de `backend/.sqlx` |
| _(pendiente)_ | Fase 2: git se registra en `tasks.md` por unidad de trabajo |

## Complexity Tracking

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| Respuesta 201/200 con la clave en claro | Requerimiento explícito de `FR-008b` (modal de entrega inmediata) | Sin modal, el alta dependería solo del correo: incumple la spec y el escenario 1/5 |
| Cambio 204→200 en regenerar | La modal necesita leer la clave nueva | Devolverla en un header o endpoint extra es más frágil y no más simple |