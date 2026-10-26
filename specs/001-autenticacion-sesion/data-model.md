# Data Model — Autenticación, sesión e identidad

**Spec**: [spec.md](./spec.md) | **Plan**: [plan.md](./plan.md) | **Creado**: 2026-10-12 | **Dividido**: 2026-10-25

> Tipos PostgreSQL; nombres en snake_case; fechas en `TIMESTAMPTZ`. Toda migración vive en
> `/migrations` (raíz). ANTES de cada migración/SQL: verificar esquema real (constitución —
> Verificación de esquema).

Esta spec posee la **identidad** (`usuarios`, `sesiones`) y el modelo de roles. Las tablas de
catálogo y de depósitos se documentan en [003](../003-catalogo-productos/data-model.md) y
[005](../005-depositos-ubicaciones/spec.md); acá solo se referencia la relación.

## Diagrama de relaciones (texto)

```text
usuarios 1─* sesiones
categorias 1─* productos                 (003)
depositos 1─* ubicaciones                (005)
ubicaciones 1─* ubicaciones (padre/hijo) (005)
productos *─* ubicaciones                (005, producto_ubicacion_permitida)
```

## usuarios

| Campo | Tipo | Restricciones |
|-------|------|----------------|
| id | SERIAL / INT GENERATED ALWAYS AS IDENTITY | PK |
| nombre | VARCHAR(120) | NOT NULL, TRIM |
| email | VARCHAR(255) | NOT NULL, UNIQUE, formato validado en app (RFC básico) |
| password_hash | VARCHAR(255) | NOT NULL (Argon2id) |
| password_temporal | BOOLEAN | NOT NULL DEFAULT TRUE — obliga cambio en primer ingreso |
| rol | VARCHAR(20) | NOT NULL, CHECK IN ('ADMIN','GERENTE','OPERARIO','CAJERO') |
| familia_rol | VARCHAR(10) | NOT NULL, CHECK IN ('GESTION','OPERACION'); mapeada en app al crear/editar el usuario |
| activo | BOOLEAN | NOT NULL DEFAULT TRUE |
| creado_en | TIMESTAMPTZ | NOT NULL DEFAULT now() |
| actualizado_en | TIMESTAMPTZ | NOT NULL DEFAULT now() |

**Reglas de negocio**

- Unicidad de email (índice único).
- Baja lógica: `activo = FALSE`; no se borra la fila (trazabilidad de movimientos futuros).
- El último usuario con `familia_rol = 'GESTION' AND activo = TRUE` no puede ser desactivado
  (validación en servicio; requiere conteo, no alcanza un CHECK SQL).
- Credenciales: hash Argon2id; temporal enviada por correo; `password_temporal` pasa a FALSE
  al cambiar la contraseña.

**Transiciones de estado**

```text
[activo=true] --desactivar--> [activo=false]  (sesiones revocadas)
[activo=false] --reactivar--> [activo=true]
cualquier login --cambio password ok--> password_temporal=false
```

> La gestión de usuarios (FR-004…FR-008) es responsabilidad de
> [002](../002-gestion-usuarios/spec.md); acá se define la entidad y sus invariantes.

## sesiones

| Campo | Tipo | Restricciones |
|-------|------|----------------|
| id | UUID | PK |
| usuario_id | INT | FK → usuarios(id) ON DELETE CASCADE, NOT NULL |
| creada_en | TIMESTAMPTZ | NOT NULL DEFAULT now() |
| expira_en | TIMESTAMPTZ | NOT NULL |
| ultima_actividad | TIMESTAMPTZ | NOT NULL DEFAULT now() (timestamp de creación; **no se actualiza**; no participa de la expiración) |
| revocada_en | TIMESTAMPTZ NULL | NULL = vigente |

**Reglas**

- Máximo UNA fila vigente por usuario: al crear sesión nueva,
  `UPDATE sesiones SET revocada_en = now() WHERE usuario_id = $1 AND revocada_en IS NULL`.
  La invariante se garantiza además con un **índice único parcial**
  `UNIQUE (usuario_id) WHERE revocada_en IS NULL` (T027); el servicio debe manejar `23505`.
- Logout / desactivar usuario: marcar `revocada_en`.
- Expiración **absoluta**: `expira_en` = ahora + `SESSION_EXPIRY_HOURS` (default 8 horas de
  reloj); sin sliding ni cálculo por inactividad. `ultima_actividad` no se usa para expirar.
- Cookie: valor = `sesiones.id` (UUID), flags HttpOnly, SameSite=Lax, path=/, `Secure` cuando
  `APP_ENV=production` (T030).

## Índices

- `usuarios(email)` UNIQUE
- `sesiones(usuario_id)` **UNIQUE** WHERE `revocada_en IS NULL` (una sola vigente; T027)

## Validaciones de integridad (app + BD)

| Regla | Nivel |
|-------|-------|
| Email único y formato | BD UNIQUE + validación manual en DTOs (el crate `validator` no se usa; T031) |
| Último admin no se desactiva | Servicio (`SELECT COUNT … FOR UPDATE`) (002) |
| Familia rol coherente con rol | CHECK + servicio (001) |
| Sesión única vigente por usuario | Servicio + índice único parcial (001; T027) |
