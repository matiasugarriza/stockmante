# Progreso de implementación — Depósitos y ubicaciones (005)

**Spec**: [spec.md](./spec.md) | **Plan**: [plan.md](./plan.md) | **Actualizado**: 2026-10-26 |
**Estado**: implementada y verificada; incluida en el commit único de cierre del Módulo 1 (2026-10-26).

## Estado por fase

| Fase | Tareas | Estado | Evidencia |
|---|---|---|---|
| 1. Modelos y validadores | T001–T002 | Completa | `models/deposito.rs`; `test_deposito_validacion` 5/5 |
| 2. Servicio y esquema | T003–T007 | Completa | `services/depositos.rs`; sin migración nueva; 5 códigos de error |
| 3. HTTP y contrato | T008–T012 | Completa | 15 rutas; `test_openapi` 6/6; `test_depositos_http` 6/6 |
| 4. Frontend | T013–T016 | Completa | schemas/cliente + 20 tests nuevos; `/admin/depositos`; enlace en nav |
| 5. Cierre | T017–T019 | Completa | gates abajo; SC-005 verificado; docs actualizadas |

## Evidencia observada (2026-10-26)

### Gates

| Comando | Resultado |
|---|---|
| `cd backend && cargo test` | **111 tests, 0 fallos** (exit 0) |
| `cd backend && cargo sqlx prepare --check` | exit 0; aviso preexistente de `.sqlx` |
| `cd frontend && npm test` | **13 archivos, 136 tests, 0 fallos** |
| `cd frontend && npm run check` | 0 errores, 0 warnings |
| `cd frontend && npm run build` | OK |

### SC-005 — ubicaciones huérfanas

```sql
SELECT count(*) AS huerfanas
FROM ubicaciones u
LEFT JOIN depositos d ON d.id = u.deposito_id
WHERE d.id IS NULL;
```

Resultado: **0**. `ubicaciones.deposito_id` es `NOT NULL` con FK `ON DELETE RESTRICT` desde
`0003`; el servicio además exige depósito existente y activo al crear.

### Comportamiento cubierto por `test_depositos_http`

- CRUD de depósito con `trim`, `PUT` parcial, limpieza de dirección, `activo` por `PUT` → 422.
- Unicidad de nombre e identificador → 409 con códigos distintos.
- Baja lógica, reactivación y borrado físico solo sin dependencias.
- Raíz/hija, tipo normalizado, duplicada en el mismo nivel, padre de otro depósito y padre
  inexistente.
- Ciclo al mover una raíz bajo su hija → 422 con mensaje accionable.
- Mover a la raíz con `padre_id: null`; cambiar de depósito solo sin hijos.
- Asignación: reemplazo, deduplicación, conjunto vacío, ubicación inactiva/inexistente y producto
  inexistente.
- OPERACION → 403 `sin_permisos` en todas las rutas; anónimo → 401.

### OpenAPI

`/api-docs/openapi.json` publica 15 operaciones nuevas bajo los tags `depositos` y `ubicaciones`,
más `GET/PUT /api/productos/{id}/ubicaciones`. `test_openapi` valida rutas y schemas `Deposito`,
`Ubicacion`, `ListaUbicaciones`.

### Decisiones registradas

- Sin migración nueva: `0003` cubre el modelo; la propiedad del esquema queda documentada en esta
  spec.
- El borrado físico se bloquea contando dependencias y mapeando `23503`; la baja lógica conserva
  la historia.
- El árbol se aplana en el cliente con guard de ciclos; no hay componente recursivo.

## Próximo paso

Spec 006 (RBAC transversal y auditoría). El commit queda para el cierre del Módulo 1.
