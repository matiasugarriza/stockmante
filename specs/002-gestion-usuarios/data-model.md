# Data Model — Gestión de usuarios

**Spec**: [spec.md](./spec.md) | **Plan**: [plan.md](./plan.md) | **Creado**: 2026-10-25

> Las tablas `usuarios` y `sesiones` son **propiedad de
> [001](../001-autenticacion-sesion/data-model.md)**. Este archivo documenta solo el ciclo de
> vida que gestiona US2 y las invariantes que su servicio debe mantener.

## usuarios (referencia)

| Campo | Tipo | Restricciones |
|-------|------|----------------|
| id | INT GENERATED ALWAYS AS IDENTITY | PK |
| nombre | VARCHAR(120) | NOT NULL, TRIM |
| email | VARCHAR(255) | NOT NULL, UNIQUE, normalizado a minúsculas en app |
| password_hash | VARCHAR(255) | NOT NULL (Argon2id) |
| password_temporal | BOOLEAN | NOT NULL DEFAULT TRUE |
| rol | VARCHAR(20) | NOT NULL, CHECK IN ('ADMIN','GERENTE','OPERARIO','CAJERO') |
| familia_rol | VARCHAR(10) | NOT NULL, CHECK IN ('GESTION','OPERACION') |
| activo | BOOLEAN | NOT NULL DEFAULT TRUE |
| creado_en / actualizado_en | TIMESTAMPTZ | NOT NULL DEFAULT now() |

## Invariantes de US2

| Regla | Dónde se garantiza |
|---|---|
| Email único | Índice `usuarios_email_key`; violación `23505` → `EmailDuplicado` (409) |
| Formato de email | `validar_email` en servicio (422) |
| Baja lógica, nunca `DELETE` | `desactivar`/`reactivar`; no existe ruta de borrado físico |
| Al desactivar se revocan sesiones | `UPDATE sesiones SET revocada_en = now()` en la misma transacción |
| Último gestor activo no se desactiva ni se degrada | `bloquear_gestores_activos` (`FOR UPDATE`, orden por `id`) + `asegurar_queda_gestor` → `UltimoAdministrador` (409) |
| Familia coherente con rol | `familia_de_rol` al crear y actualizar |
| Contraseña temporal en el alta y el reseteo | `generar_password_temporal` + `EnvioCorreo` dentro de la transacción |
| `password_hash` nunca sale en respuestas | Modelo público `Usuario` |

## Transiciones de estado

```text
[activo=true] --desactivar--> [activo=false]   (sesiones revocadas)
[activo=false] --reactivar--> [activo=true]
[activo=true]  --degradar el último GESTION a OPERACION--> 409 UltimoAdministrador
cualquier usuario --password-temporal--> nueva temporal + sesiones revocadas
```

## Paginación

- `pagina >= 1`; `1 <= por_pagina <= 100` (default 20).
- `offset = (pagina - 1) * por_pagina` con `checked_mul`: valores extremos → 422, nunca overflow.
- Respuesta: `{ items, total, pagina, por_pagina }`.

## Validaciones de integridad (app + BD)

| Regla | Nivel |
|-------|-------|
| Email único y formato | BD UNIQUE + app |
| Último gestor activo | Servicio (`SELECT … FOR UPDATE`) |
| Familia rol coherente | CHECK + servicio |
| Sesiones revocadas al desactivar | Servicio, misma transacción |
