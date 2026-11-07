# Progreso de implementación — RBAC transversal y auditoría (006)

**Spec**: [spec.md](./spec.md) | **Plan**: [plan.md](./plan.md) | **Actualizado**: 2026-11-07 |
**Estado**: implementada y verificada (2026-10-26); **ampliación 2026-11-07** en curso:
auditoría de categorías, depósitos, ubicaciones y asignaciones (T012–T027).

## Estado por fase

| Fase | Tareas | Estado | Evidencia |
|---|---|---|---|
| 1. Auditoría | T001–T005 | Completa | migración `0008` aplicada en ambas bases; `services/auditoria.rs`; servicios instrumentados |
| 2. Verificación | T006–T008 | Completa | `test_rbac_matriz` 4/4; helper de actor; suite completa 115/115 |
| 3. Frontend | T009–T009a | Completa | `admin/+layout.svelte`; botones `Desactivar` en productos/depósitos; `svelte-check` 0/0 |
| 4. Cierre | T010–T011 | Completa | gates abajo; artefactos e índice actualizados |
| 5. Ampliación de auditoría | T012–T018 | Pendiente | categorías, depósitos, ubicaciones y asignaciones |
| 6. Cierre de hallazgos | T019–T027 | Pendiente | matriz, lecturas, degradación, SC-007 y evidencia |

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

## Próximo paso

Implementar T012–T027 (ampliación de auditoría y cierre de hallazgos) y después el cierre
transversal C2–C5 del Módulo 1.
