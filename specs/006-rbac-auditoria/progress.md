# Progreso de implementación — RBAC transversal y auditoría (006)

**Spec**: [spec.md](./spec.md) | **Plan**: [plan.md](./plan.md) | **Actualizado**: 2026-11-07 |
**Estado**: implementada y verificada (2026-10-26); **ampliación 2026-11-07 implementada y
verificada** (T001a, T012–T027).

## Estado por fase

| Fase | Tareas | Estado | Evidencia |
|---|---|---|---|
| 1. Auditoría | T001–T005 | Completa | migración `0008` aplicada en ambas bases; `services/auditoria.rs`; servicios instrumentados |
| 2. Verificación | T006–T008 | Completa | `test_rbac_matriz` 4/4; helper de actor; suite completa 115/115 |
| 3. Frontend | T009–T009a | Completa | `admin/+layout.svelte`; botones `Desactivar` en productos/depósitos; `svelte-check` 0/0 |
| 4. Cierre | T010–T011 | Completa | gates abajo; artefactos e índice actualizados |
| 5. Ampliación de auditoría | T012–T018 | Completa | categorías, depósitos, ubicaciones y asignaciones auditadas en la misma transacción |
| 6. Cierre de hallazgos | T019–T027 | Completa | matriz con lote, lecturas 200, degradación, SC-007, guard único y gates |

## Evidencia observada (2026-10-26)

### Gates

| Comando | Resultado |
|---|---|
| `cd backend && cargo test` | **115 tests, 0 fallos** (exit 0) |
| `cd backend && cargo sqlx prepare --check` | **OK** (sin diferencias; verificado 2026-11-07 sobre HEAD) |
| `cd backend && cargo test --test test_rbac_matriz` | **4/4** |
| `cd frontend && npm test` | **13 archivos, 136 tests, 0 fallos** |
| `cd frontend && npm run check` | 0 errores, 0 warnings |
| `cd frontend && npm run build` | OK |

### Migración

```
Applied 8/migrate auditoria (12.258ms)   # stockmante
Applied 8/migrate auditoria (13.3186ms)  # stockmante_test
```

### Auditoría verificada por SQL

`test_rbac_matriz` consulta la tabla real y comprueba:

- Productos: `CREAR`, `ACTUALIZAR`, `DESACTIVAR`, `REACTIVAR` con `usuario_id` del gestor y
  `cuando` no nulo.
- Usuarios: `CREAR`, `ACTUALIZAR`, `DESACTIVAR`, `REACTIVAR`, `PASSWORD_TEMPORAL`.
- Un 403 de OPERACION no incrementa el conteo de filas de ese autor.

### Matriz RBAC cubierta

31 combinaciones de 001–005 en el alcance original (44 en el test actual, que además cubre 012);
T019 suma `GET /api/productos/ubicaciones`. OPERACION → 403 `sin_permisos`; GESTION → nunca
401/403.

### Casos borde

- Sesión degradada: el `AuthUsuario` relee la fila en cada request; el test de matriz usa
  sesiones reales, no roles cacheados.
- 403 no audita: verificado en `un_403_no_registra_auditoria`.
- Reversión: la auditoría se inserta en la misma transacción; los tests de alta fallida
  (`si_el_correo_falla_no_queda_usuario`) siguen en verde.

## Ampliación 2026-11-07

- **Clarificación**: la auditoría se extiende a todas las escrituras de gestión de 001–005:
  categorías, depósitos, ubicaciones y asignación producto–ubicación, además de usuarios y
  productos (FR-019, SC-030).
- **Sin migración**: `auditoria` ya es genérica; se agregan `ELIMINAR` y `ASIGNAR_UBICACIONES` como
  acciones y `categorias`, `depositos`, `ubicaciones` como `tabla_ref`.
- **Pendiente**: T012–T018 (instrumentación y tests) y T019–T027 (hallazgos del análisis). El código
  actual solo instrumenta usuarios y productos.
- **Evidencia pendiente**: T001a (verificación de esquema), T024 (RED/gate SQLx) y T027 (gates).

## Ampliación 2026-11-07 — evidencia

### Verificación de esquema (T001a)

`psql` sobre `stockmante` y `stockmante_test`: `auditoria(id bigint NOT NULL, tabla_ref varchar NOT
NULL, registro_id integer NOT NULL, accion varchar NOT NULL, usuario_id integer NULL, cuando
timestamptz NOT NULL)`; `usuarios.id integer`; FK `usuario_id → usuarios(id) ON DELETE SET NULL`.
Coincide con `data-model.md`.

### RED (T024)

Con el código sin instrumentar (`git stash` de `src/`), `test_rbac_matriz` dio **8 passed, 3
failed**: `audita_altas_cambios_y_bajas_de_categorias`, `audita_depositos_y_ubicaciones` y
`audita_la_asignacion_de_ubicaciones_incluso_vacia`. Con la instrumentación: 11/11. Los tests de
lecturas permitidas (T020), degradación (T021), SC-007 (T022) y 403 sin auditoría verifican
comportamiento ya existente y pasaron en la primera corrida.

### Alcance de `cargo sqlx prepare --check` (T024)

`auditoria::registrar` y `auditoria::listar` usan `sqlx::query`/`query_as` en runtime (sin macros),
así que no generan entradas en `.sqlx/`: el gate verifica las macros `query!` existentes y la
corrección de estas consultas la prueban los tests de integración contra `stockmante_test`.

### Nulabilidad (T026)

`AuditoriaFila`/`AuditoriaItem`: `id: i64` (la columna es `BIGINT`; con `i32` la consulta fallaba en
runtime), `usuario_id: Option<i32>`, `usuario_nombre: Option<String>` y `LEFT JOIN usuarios`. La UI
de `/admin/auditoria` muestra «(cuenta eliminada)» cuando el autor es `null`.

### Frontend (T023)

`admin/+layout.svelte` es el único guard (OPERACION → `/caja`); se quitaron los `goto('/')` de
`admin/{productos,categorias,depositos,usuarios}/+page.svelte`.

## Gates del cierre (2026-11-07)

Corrida única para 006–017 (con `CARGO_TARGET_DIR` aparte porque el backend estaba corriendo):

| Comando | Resultado |
|---|---|
| `cd backend && cargo test --no-fail-fast` | **290 passed, 0 failed, 3 ignored** (55 binarios; ignorados: `sc018`, `sc024`, `enviar_por_http`, corridos a mano y en verde) |
| `cd backend && cargo sqlx prepare --check` | **OK** (exit 0, sin diferencias) |
| `cd frontend && npm test` | **40 archivos, 327 tests, 0 fallos** |
| `cd frontend && npm run check` | **0 errores, 0 warnings** (477 archivos) |
| `cd frontend && npm run build` | **OK** (adapter-static) |
| `cd asistente && npm test` / `npm run check` | **53/53** / **OK** |

Sin `unwrap()`/`expect()`/`panic!` en rutas de producción del backend (las únicas apariciones
están en módulos `#[cfg(test)]` o en comentarios).

**Migraciones**: `0015_alertas_stock.sql` aplicada en ambas bases; historial regularizado (ver
`specs/011-alertas-stock-minimo/progress.md`).

## Próximo paso

Recorridos manuales de quickstart pendientes (necesitan el backend reiniciado con este código):
009 T031, 010 T021, 011 T026, 016 T018. Luego el cierre transversal C2–C5 del Módulo 1.
