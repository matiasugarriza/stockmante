# Progreso de implementación — RBAC transversal y auditoría (006)

**Spec**: [spec.md](./spec.md) | **Plan**: [plan.md](./plan.md) | **Actualizado**: 2026-10-26 |
**Estado**: implementada y verificada; incluida en el commit único de cierre del Módulo 1 (2026-10-26).

## Estado por fase

| Fase | Tareas | Estado | Evidencia |
|---|---|---|---|
| 1. Auditoría | T001–T005 | Completa | migración `0008` aplicada en ambas bases; `services/auditoria.rs`; servicios instrumentados |
| 2. Verificación | T006–T008 | Completa | `test_rbac_matriz` 4/4; helper de actor; suite completa 115/115 |
| 3. Frontend | T009 | Completa | `admin/+layout.svelte`; `svelte-check` 0/0 |
| 4. Cierre | T010–T011 | Completa | gates abajo; artefactos e índice actualizados |

## Evidencia observada (2026-10-26)

### Gates

| Comando | Resultado |
|---|---|
| `cd backend && cargo test` | **115 tests, 0 fallos** (exit 0) |
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

33 combinaciones método/ruta: escrituras de usuarios, categorías, productos, depósitos,
ubicaciones y asignaciones; listados de gestión. OPERACION → 403 `sin_permisos`; GESTION →
nunca 401/403.

### Casos borde

- Sesión degradada: el `AuthUsuario` relee la fila en cada request; el test de matriz usa
  sesiones reales, no roles cacheados.
- 403 no audita: verificado en `un_403_no_registra_auditoria`.
- Reversión: la auditoría se inserta en la misma transacción; los tests de alta fallida
  (`si_el_correo_falla_no_queda_usuario`) siguen en verde.

## Próximo paso

Cierre transversal C2–C5 del Módulo 1 y commit único final en `feat/modulo-1-resto`.
