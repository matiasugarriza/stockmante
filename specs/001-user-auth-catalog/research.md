# Research: Módulo 1 — Usuarios, Autenticación y Catálogo Base

**Date**: 2026-10-12 | **Spec**: [spec.md](./spec.md) | **Plan**: [plan.md](./plan.md)

Todos los `NEEDS CLARIFICATION` del Technical Context quedaron resueltos. Sin pendientes.

## R1 — Mecanismo de sesión con invalidación única

- **Decision**: Sesión **respaldada en servidor** en tabla `sesiones`, entregada mediante cookie `HttpOnly` + `SameSite=Lax`. Un solo `session_id` vigente por usuario: el alta de sesión nueva revoca (borrado lógico) la anterior. El logout y la desactivación de usuario borran la fila. La expiración por inactividad se calcula en servidor (`expires_at`).
- **Rationale**: la spec exige una sola sesión vigente por usuario y invalidación en el siguiente request tras desactivar (FR-001, SC-004); verificar contra BD en cada request lo garantiza de forma simple. Coincide con Supuestos de spec ("sesión del servidor").
- **Alternatives considered**: JWT stateless (no permite revocación inmediata sin blacklist = peor); cookies con token solo firmado sin BD (mismo problema de single-session); Redis (dependencia nueva innecesaria — YAGNI, Principio I).

## R2 — Hash de contraseñas

- **Decision**: **Argon2id** (crato/argon2 crate, la recomendación actual de OWASP para password hashing). Contraseña temporal generada criptográficamente (12+ caracteres) y enviada por correo; cambio obligatorio marcado en el usuario (`debe_cambiar_password`).
- **Rationale**: Principio V y seguridad; no se guarda contraseña en claro jamás; argon2id es resistente a GPU/ASIC.
- **Alternatives considered**: bcrypt (viejo pero aceptable), PBKDF2 (más flojo), bcrypt-via-SQL (mezcla lógica de negocio con BD).

## R3 — Validación de entradas

- **Decision**: **Backend**: `serde` + `validator` (o `garde`) en structs DTO; errores de validación → `AppError::Validacion` (422). **Frontend**: **Zod** en `$lib/api/*.ts` y forms (definiciones compartidas por feature). Un único esquema por entidad en cada capa; el backend es la autoridad final.
- **Rationale**: SDD pide funciones tipadas centralizadas; Zod ya es el estándar de facto en SvelteKit/TS; en Rust el ecosistema maduro es validator/serde. Constitución: validación antes de tocar BD + verificación de esquema.
- **Alternatives considered**: Joi (JS legacy, no se alinea con TS estricto), solo backend (UX pobre: errores tardíos), shared schema via JSON Schema (overhead de tooling sin necesidad actual).

## R4 — Mapeo de errores HTTP (contrato con Principio V)

- **Decision**: enum `AppError` con variantes específicas de dominio (`EmailDuplicado`, `SkuDuplicado`, `CodigoBarrasDuplicado`, `CredencialesInvalidas`, `UsuarioInactivo`, `UltimoAdministrador`, `ProductoConReferencias`, `UbicacionHuerfana`, `DepositoConDependencias`, `Validacion(Vec<String>)`, `NoAutorizado`, `SesionExpirada`, `NoEncontrado`) → 409/422/401/403/404 según corresponda; variante genérica `AppError::Interno` → 500 con mensaje genérico (sin exponer detalles de BD).
- **Rationale**: Principio V literal de constitution v1.1.1. Errores de login SIEMPRE `CredencialesInvalidas` con el mismo mensaje (anti-enumeración, acotado en spec Supuestos).
- **Alternatives considered**: `anyhow`/`Box<dyn Error>` hacia el cliente (viola Principio V), un solo error genérico 400 (viola claridad de mensajes, Principio III).

## R5 — RBAC: guards en Axum

- **Decision**: Extractor `AuthUsuario` (lee cookie → carga sesión + usuario) y extractor/limitador `RequiereGestion` (rol familia gestión). Middleware global de sesión expirada. Endpoints de caja solo requieren `AuthUsuario`; endpoints de gestión requieren `RequiereGestion`.
- **Rationale**: axum extractors son el mecanismo idiomático; mantiene handlers delgados (Principio II) y centraliza la autorización (Principio V: permisos OPERARIO/ADMIN en API, no en chatbot).
- **Alternatives considered**: crates de RBAC completos (complejidad innecesaria — YAGNI), guards solo en frontend (inseguro; frontend es UX, el backend es la frontera).

## R6 — Estructura de repositorio (mono-repo backend + frontend)

- **Decision**: mono-repo con `backend/` (Axum, SQLx, migraciones en `backend/migrations/` o raíz `/migrations` según constitución → **`/migrations`** en raíz) y `frontend/` (SvelteKit SPA + Tailwind). Contratos HTTP documentados en `specs/001-user-auth-catalog/contracts/api.md`.
- **Rationale**: constitución fija stack; `/migrations` versionadas en raíz es regla de la constitución.
- **Alternatives considered**: monorepo con workspace Rust compartido (premature — un solo backend), `apps/`+`packages/` (mismo costo, paths alineados a constitución).

## R7 — Estrategia de seed / bootstrap

- **Decision**: migración o binario de setup crea: depósito "Principal" (Supuestos), un Administrador inicial con contraseña temporal impresa en consola solo en `dev` (nunca en logs de prod) y categorías base opcionales.
- **Rationale**: spec Supuestos lo requiere; evita sistema sin administración.
- **Alternatives considered**: fixture SQL manual (olvidable, sin camino verificado), registro público (riesgo de seguridad).

## R8 — Correo (envío de contraseña temporal)

- **Decision**: interfaz `TraitEnvioCorreo` con implementación de desarrollo (log/consola) y envío SMTP real tras `EMAIL_*` env; config por variables de entorno. Sin proveedor cloud hardcodeado.
- **Rationale**: YAGNI + testabilidad (Principio IV: inyección de I/O); sin credenciales externas en este módulo.
- **Alternatives considered**: proveedor cloud específico (dependencia externa prematura), sin correo (spec exige envío).
