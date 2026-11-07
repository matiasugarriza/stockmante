# Implementation Plan: RBAC transversal y auditoría (006)

**Spec**: [spec.md](./spec.md) | **Caso de uso**: US6 | **Creado**: 2026-10-26

## Summary

Cerrar el Módulo 1 verificando la matriz RBAC completa sobre los endpoints de 001–005 y
agregando auditoría básica de **todas las escrituras de gestión** de 001–005: usuarios, productos,
categorías, depósitos, ubicaciones y asignación producto–ubicación. No se agregan endpoints
públicos: se suma una tabla `auditoria`, un servicio interno y un test transversal. **Ampliación
2026-11-07**: se extiende la auditoría a categorías, depósitos, ubicaciones y asignaciones, y se
cierran los hallazgos del análisis (matriz del endpoint masivo, lecturas permitidas, guard único,
degradación de rol, SC-007 y evidencia de gates).

## Technical Context

- Backend: Rust + Axum 0.8 + SQLx 0.8 + PostgreSQL 15+.
- Frontend: SvelteKit 5 (SPA) + Tailwind; el backend sigue siendo la autoridad.
- Migración: `0008_auditoria.sql` (la `0006` fue tomada por el catálogo de 003 y la `0007` por 004).
- **Ampliación 2026-11-07**: no requiere migración; `auditoria` ya es genérica (`tabla_ref`,
  `accion`) y `accion VARCHAR(24)` admite `ASIGNAR_UBICACIONES`.
- Pruebas: matriz HTTP de toda la superficie restringida (incluido `GET /productos/ubicaciones`),
  lecturas permitidas a OPERACION, verificación SQL de auditoría por entidad y caso de degradación
  de rol.

## Constitution Check

| Principio | Cumplimiento |
|---|---|
| I. Simplicidad y anti-sobreingeniería | una tabla, un servicio y cuatro servicios instrumentados (usuarios, productos, categorías, depósitos/ubicaciones); sin endpoints nuevos ni abstracciones |
| II. Mantenibilidad | `auditoria.rs` centraliza el registro; actor desde `RequiereGestion`; un único guard de frontend en `admin/+layout.svelte` |
| III. Claridad en mensajes | se conservan los 403 `sin_permisos` y los mensajes existentes |
| IV. Testabilidad primero | matriz HTTP (incluido el endpoint masivo), auditoría por SQL por entidad, lecturas permitidas, degradación de rol y guard estructural |
| V. Integridad de datos | auditoría dentro de la misma transacción; un fallo de negocio no deja fila; verificación de esquema previa registrada en T001a |

> **Evidencia (2026-11-07)**: T001a (esquema real verificado con `psql` en `stockmante` y
> `stockmante_test`) y T024 (RED observado: 3 tests de auditoría fallan sin instrumentación) están en
> `progress.md`. **Alcance de `cargo sqlx prepare --check`**: el `INSERT` de `auditoria::registrar`
> y la consulta de `auditoria::listar` usan `sqlx::query`/`query_as` en runtime (no macros), por lo
> que no generan entradas en `.sqlx/`; el gate solo cubre las macros `query!` existentes, y la
> corrección de esas consultas la prueban los tests de integración contra `stockmante_test`.

## Project Structure

```text
migrations/0008_auditoria.sql
backend/src/services/auditoria.rs
backend/src/services/usuarios.rs        # instrumentación
backend/src/services/productos.rs       # instrumentación
backend/src/services/categorias.rs      # instrumentación (ampliación 2026-11-07)
backend/src/services/depositos.rs       # depósitos, ubicaciones y asignación (ampliación)
backend/src/handlers/usuarios.rs        # actor desde RequiereGestion
backend/src/handlers/productos.rs       # actor desde RequiereGestion
backend/src/handlers/categorias.rs      # actor (ampliación)
backend/src/handlers/depositos.rs       # actor (ampliación)
backend/src/handlers/ubicaciones.rs     # actor (ampliación)
backend/tests/integration/test_rbac_matriz.rs
backend/tests/unit/common/mod.rs        # helper `actor_de_test`
frontend/src/routes/admin/+layout.svelte
```

## Diseño

### Auditoría

| Columna | Tipo | Regla |
|---|---|---|
| `id` | `BIGINT IDENTITY` | PK |
| `tabla_ref` | `VARCHAR(32)` | `usuarios`, `productos`, `categorias`, `depositos` o `ubicaciones` |
| `registro_id` | `INT` | id del registro afectado (sin FK: sobrevive a limpiezas) |
| `accion` | `VARCHAR(24)` | `CREAR`, `ACTUALIZAR`, `DESACTIVAR`, `REACTIVAR`, `ELIMINAR`, `PASSWORD_TEMPORAL`, `ASIGNAR_UBICACIONES` |
| `usuario_id` | `INT NULL` | autor; FK a `usuarios` con `ON DELETE SET NULL` |
| `cuando` | `TIMESTAMPTZ` | `now()` |

La escritura ocurre **dentro de la misma transacción** del cambio auditado. Si el cambio se
revierte (validación, correo, unicidad), no queda auditoría. Un 403 nunca llega al servicio, por
lo que no registra nada.

### Acciones auditadas

- Usuarios: alta, edición, baja lógica, reactivación y contraseña temporal.
- Productos: alta, edición (incluye precio), baja lógica y reactivación. No exponen `DELETE` físico
  (003 usa baja lógica); SC-007 aplica a depósitos/ubicaciones y la alternativa para productos es
  `desactivar`.
- Categorías: alta, edición, baja lógica y reactivación (no hay `DELETE`).
- Depósitos y ubicaciones: alta, edición, borrado físico permitido (sin dependencias), baja lógica y
  reactivación. El borrado con dependencias responde 409 y no audita.
- Asignaciones producto–ubicación: `ASIGNAR_UBICACIONES` sobre `productos` con
  `registro_id = producto_id`, tanto al asignar como al limpiar con `[]`.
- No se auditan lecturas, login/logout, cambio voluntario de contraseña, movimientos de stock,
  ventas, compras ni bootstrap.

### RBAC

No se cambia el modelo: se verifica endpoint por endpoint con sesión OPERACION (403
`sin_permisos`) y GESTION (nunca 401/403). Los listados restringidos son los de gestión
(`GET /usuarios`, `GET /productos`, `GET /productos/ubicaciones`, `GET /depositos`,
`GET /depositos/{id}/ubicaciones` y `GET /ubicaciones/{id}`); la lectura de caja
(`GET /productos/{id}`, `/productos/buscar`, `GET /categorias` y `GET /categorias/{id}`) sigue
disponible para OPERACION y se verifica con aserciones 200.

### Frontend

`/admin/+layout.svelte` es el **único** guard: si la sesión es OPERACION redirige a `/caja` y no
renderiza formularios de gestión. Se eliminan los redirects a `/` de `admin/productos`,
`admin/categorias`, `admin/depositos` y `admin/usuarios` para que no compitan con el layout. La
alternativa de baja lógica ya existe en `admin/productos` y `admin/depositos` (botones
`Desactivar`); T009a la verifica como parte de SC-007.

## Estrategia de pruebas

- `test_rbac_matriz` (4 tests base + ampliación): matriz completa con `GET /productos/ubicaciones`
  y el resto de 001–005; auditoría de productos, usuarios, categorías, depósitos, ubicaciones y
  asignaciones; 403 que no audita.
- Casos nuevos: lecturas permitidas a OPERACION (200), degradación de rol en sesión viva (403 en el
  siguiente request) y SC-007 directo (409 con dependencias).
- Tests de servicio existentes: se actualizan al parámetro `actor_id` con `actor_de_test`.
- Frontend: `svelte-check` y la suite existente; un único guard en `admin/+layout.svelte`.

## Riesgos y mitigaciones

| Riesgo | Mitigación |
|---|---|
| Auditar fuera de la transacción | `registrar` recibe `&mut Transaction`; se invoca antes del `commit` |
| Actor nulo o falsificado | el actor sale de la sesión validada (`RequiereGestion`), nunca del body |
| Crecimiento sin consultas | índices por `(tabla_ref, registro_id)` y `(usuario_id)`; consultas quedan fuera de alcance |
| Test de matriz frágil por datos | ids `999999` y cuerpos vacíos: se prueba RBAC, no reglas de negocio |
| Más servicios instrumentados = más superficie de error | el registro se hace en la misma transacción y se cubre con tests por entidad |
| Auditar un `DELETE` físico deja `registro_id` huérfano | no hay FK; la fila de auditoría sobrevive por diseño |
| Asignación sin fila propia | se audita el producto (`ASIGNAR_UBICACIONES`), no la tabla puente |

## Fases

1. Migración y servicio de auditoría (base).
2. Instrumentación transaccional de usuarios y productos (base).
3. Actores en handlers y ajuste de tests de servicio.
4. Test transversal base de RBAC y auditoría.
5. Guard de frontend, gates y documentación.
6. **Ampliación 2026-11-07** (Fase 5 de `tasks.md`): auditoría de categorías, depósitos,
   ubicaciones y asignaciones.
7. **Cierre de hallazgos** (Fase 6 de `tasks.md`): endpoint masivo en la matriz, lecturas permitidas,
   degradación de rol, SC-007 directo y evidencia de los gates.

## Ampliación 2026-11-07 — Auditoría de todas las escrituras de gestión

- **Alcance**: categorías (alta, edición, baja, reactivación), depósitos y ubicaciones (alta,
  edición, borrado físico permitido, baja, reactivación) y asignación producto–ubicación.
- **Sin migración**: `auditoria` ya es genérica; se agregan valores de `accion`
  (`ELIMINAR`, `ASIGNAR_UBICACIONES`) y de `tabla_ref` (`categorias`, `depositos`, `ubicaciones`).
- **Asignaciones**: se registra `ASIGNAR_UBICACIONES` sobre `productos` con `registro_id =
  producto_id`; también cuando el conjunto queda vacío.
- **No auditado**: lecturas, login/logout, cambio voluntario de contraseña, stock, ventas, compras
  y bootstrap.
- **Correcciones de consistencia**: matriz con `GET /productos/ubicaciones`; lecturas permitidas a
  OPERACION; un solo guard de frontend; test de degradación de rol; SC-007 directo; verificación de
  esquema y evidencia RED/SQLx.

## Complexity Tracking

Sin desviaciones: una tabla, un servicio, cuatro servicios instrumentados y un test transversal.
La ampliación no agrega tablas, endpoints ni dependencias.
