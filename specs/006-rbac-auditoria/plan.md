# Implementation Plan: RBAC transversal y auditoría (006)

**Spec**: [spec.md](./spec.md) | **Caso de uso**: US6 | **Creado**: 2026-10-26

## Summary

Cerrar el Módulo 1 verificando la matriz RBAC completa sobre los endpoints de 001–005 y
agregando auditoría básica de altas, cambios y bajas de usuarios y productos. No se agregan
endpoints públicos: se suma una tabla `auditoria`, un servicio interno y un test transversal.

## Technical Context

- Backend: Rust + Axum 0.8 + SQLx 0.8 + PostgreSQL 15+.
- Frontend: SvelteKit 5 (SPA) + Tailwind; el backend sigue siendo la autoridad.
- Migración: `0008_auditoria.sql` (la `0006` fue tomada por el catálogo de 003; el texto de la
  spec se corrige en esta misma spec).
- Pruebas: matriz HTTP de toda la superficie restringida + verificación SQL de auditoría.

## Constitution Check

| Principio | Cumplimiento |
|---|---|
| I. Especificación primero | spec 006 + este plan + tasks antes del código |
| II. Tipado y centralización | servicio `auditoria.rs`; actor desde `RequiereGestion`; sin lógica de permisos duplicada |
| III. Mensajes accionables en español | se conservan los 403 `sin_permisos` y los mensajes existentes |
| IV. Seguridad y RBAC | la verificación cubre el 100% de las escrituras y listados restringidos |
| V. Pruebas | matriz HTTP, auditoría por SQL y guard de layout; sin `unwrap`/`panic!`/`expect` en producción |

## Project Structure

```text
migrations/0008_auditoria.sql
backend/src/services/auditoria.rs
backend/src/services/usuarios.rs        # instrumentación
backend/src/services/productos.rs       # instrumentación
backend/src/handlers/usuarios.rs        # actor desde RequiereGestion
backend/src/handlers/productos.rs       # actor desde RequiereGestion
backend/tests/integration/test_rbac_matriz.rs
backend/tests/unit/common/mod.rs        # helper `actor_de_test`
frontend/src/routes/admin/+layout.svelte
```

## Diseño

### Auditoría

| Columna | Tipo | Regla |
|---|---|---|
| `id` | `BIGINT IDENTITY` | PK |
| `tabla_ref` | `VARCHAR(32)` | `usuarios` o `productos` |
| `registro_id` | `INT` | id del registro afectado (sin FK: sobrevive a limpiezas) |
| `accion` | `VARCHAR(24)` | `CREAR`, `ACTUALIZAR`, `DESACTIVAR`, `REACTIVAR`, `PASSWORD_TEMPORAL` |
| `usuario_id` | `INT NULL` | autor; FK a `usuarios` con `ON DELETE SET NULL` |
| `cuando` | `TIMESTAMPTZ` | `now()` |

La escritura ocurre **dentro de la misma transacción** del cambio auditado. Si el cambio se
revierte (validación, correo, unicidad), no queda auditoría. Un 403 nunca llega al servicio, por
lo que no registra nada.

### Acciones auditadas

- Usuarios: alta, edición, baja lógica, reactivación y contraseña temporal.
- Productos: alta, edición (incluye precio), baja lógica y reactivación.
- Depósitos/ubicaciones: sin auditoría en esta spec (FR-019 solo nombra usuarios y productos);
  su borrado con dependencias ya se bloquea con 409 (SC-007).

### RBAC

No se cambia el modelo: se verifica endpoint por endpoint con sesión OPERACION (403
`sin_permisos`) y GESTION (nunca 401/403). Los listados restringidos son los de gestión
(`GET /usuarios`, `GET /productos`, `GET /depositos`, `GET /ubicaciones` y anidados); la lectura
de caja (`GET /productos/{id}`, `/productos/buscar`, categorías) sigue disponible para OPERACION.

### Frontend

`/admin/+layout.svelte` centraliza el guard: si la sesión es OPERACION redirige a `/caja` y no
renderiza formularios de gestión. Los enlaces del header ya estaban ocultos; esto cubre el acceso
directo por URL.

## Estrategia de pruebas

- `test_rbac_matriz` (4 tests): matriz completa, auditoría de productos, auditoría de usuarios y
  un 403 que no audita.
- Tests de servicio existentes: se actualizan al nuevo parámetro `actor_id` con
  `actor_de_test` (usuario OPERACION único).
- Frontend: `svelte-check` y la suite existente; el guard es estructural.

## Riesgos y mitigaciones

| Riesgo | Mitigación |
|---|---|
| Auditar fuera de la transacción | `registrar` recibe `&mut Transaction`; se invoca antes del `commit` |
| Actor nulo o falsificado | el actor sale de la sesión validada (`RequiereGestion`), nunca del body |
| Crecimiento sin consultas | índices por `(tabla_ref, registro_id)` y `(usuario_id)`; consultas quedan fuera de alcance |
| Test de matriz frágil por datos | ids `999999` y cuerpos vacíos: se prueba RBAC, no reglas de negocio |

## Fases

1. Migración y servicio de auditoría.
2. Instrumentación transaccional de usuarios y productos.
3. Actores en handlers y ajuste de tests de servicio.
4. Test transversal de RBAC y auditoría.
5. Guard de frontend, gates y documentación.

## Complexity Tracking

Sin desviaciones: una tabla, un servicio, dos servicios instrumentados y un test transversal.
