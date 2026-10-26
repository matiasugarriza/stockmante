# Specs — índice por caso de uso

**Regla de estructura**: una spec por **caso de uso** (user story), no una spec por módulo.
Cada spec tiene su propio alcance, requisitos, contratos, plan y tareas; la base técnica
compartida vive en `001-autenticacion-sesion`.

**Origen**: hasta el 2026-10-25 existía una sola spec (`specs/001-user-auth-catalog/`) que
agrupaba los 6 casos de uso del Módulo 1. Se dividió a pedido del usuario; el contenido
original queda en el historial de git. Los textos de requisitos, escenarios y supuestos se
conservan **verbatim**; solo se distribuyeron por caso de uso.

## Mapa de specs

| Spec | Caso de uso | Requisitos | Criterios | Estado | Depende de |
|---|---|---|---|---|---|
| [001-autenticacion-sesion](./001-autenticacion-sesion/spec.md) | Login, sesión única y cambio obligatorio de contraseña | FR-001, FR-002, FR-003, FR-008a | SC-003, SC-004 | Implementado y cerrado | — |
| [002-gestion-usuarios](./002-gestion-usuarios/spec.md) | Alta, edición, baja lógica y reactivación de usuarios | FR-004, FR-005, FR-006, FR-007, FR-008 | SC-001 | Implementado (retro-documentado) | 001 |
| [003-catalogo-productos](./003-catalogo-productos/spec.md) | CRUD de productos y categorías, baja lógica | FR-009, FR-010, FR-012, FR-014 | SC-006 | **A replantear** (WIP de backend deshecho; respaldo en `wip/003-catalogo`) | 001 |
| [004-busqueda-caja](./004-busqueda-caja/spec.md) | Búsqueda rápida por SKU, código, nombre y categoría | FR-011, FR-013 | SC-002 | Especificado (sin plan aún) | 001, 003 |
| [005-depositos-ubicaciones](./005-depositos-ubicaciones/spec.md) | Depósitos, ubicaciones jerárquicas y asignación a producto | FR-015, FR-016, FR-017, FR-018 | SC-005 | Especificado (sin plan aún) | 001 (003 para asignación) |
| [006-rbac-auditoria](./006-rbac-auditoria/spec.md) | Verificación transversal de RBAC y auditoría básica | FR-003 (verificación), FR-019 (derivado) | SC-007, SC-008 | Especificado (sin plan aún) | 001–005 |

## Partición de requisitos funcionales

Cada FR tiene **un solo dueño**. Los demás specs lo referencian, no lo repiten.

| FR | Dueño | Tema |
|---|---|---|
| FR-001 | 001 | Login y sesión única |
| FR-002 | 001 | Logout |
| FR-003 | 001 | Modelo RBAC de dos familias (002–006 aportan sus filas de permisos) |
| FR-004 | 002 | Unicidad de correo |
| FR-005 | 002 | Baja lógica de usuarios |
| FR-006 | 002 | Guarda del último gestor GESTION activo |
| FR-007 | 002 | CRUD de usuarios solo gestión |
| FR-008 | 002 | Validación de formato de correo |
| FR-008a | 001 | Contraseña temporal: generación, entrega, cambio obligatorio y reseteo |
| FR-009 | 003 | CRUD de productos y formato de SKU/código |
| FR-010 | 003 | Categorías planas |
| FR-011 | 004 | Búsqueda y filtrado de productos |
| FR-012 | 003 | Borrado físico bloqueado / baja lógica de productos |
| FR-013 | 004 | Inactivos fuera de búsquedas de venta |
| FR-014 | 003 | Precio y baja de productos solo gestión |
| FR-015 | 005 | CRUD de depósitos |
| FR-016 | 005 | Ubicaciones jerárquicas sin huérfanas |
| FR-017 | 005 | Asignación de ubicaciones permitidas por producto |
| FR-018 | 005 | Borrado bloqueado de depósitos con dependencias |
| FR-019 | 006 | Auditoría básica (derivado de US6 escenario 3 y Supuestos; el spec original no le asignó FR) |

## Partición de criterios de éxito

| SC | Dueño | Verificación |
|---|---|---|
| SC-001 | 002 | Alta de usuario + primer login < 2 min |
| SC-002 | 004 | 95% de búsquedas por SKU/código ≤ 1 s |
| SC-003 | 001 | Login genérico sin enumeración |
| SC-004 | 001 | Cero acciones con sesión de usuario desactivado |
| SC-005 | 005 | Cero ubicaciones sin depósito |
| SC-006 | 003 | Cero SKU y códigos de barras duplicados |
| SC-007 | 006 | Borrados con dependencias bloqueados (productos y depósitos) |
| SC-008 | 006 | Operario no completa operaciones de gestión |

## Ledger de migraciones

| Versión | Archivo | Propósito | Spec dueña | Creada en |
|---|---|---|---|---|
| 0001 | `0001_usuarios_sesiones.sql` | `usuarios`, `sesiones` | 001 | 001 / T007 |
| 0002 | `0002_categorias_productos.sql` | `categorias`, `productos` | 003 | 001 / T008 (fase foundational) |
| 0003 | `0003_depositos_ubicaciones.sql` | `depositos`, `ubicaciones`, `producto_ubicacion_permitida` | 005 | 001 / T009 (fase foundational) |
| 0004 | `0004_productos_codigo_barras_null_distinct.sql` (eliminada) | Corrige `UNIQUE NULLS NOT DISTINCT` → varios productos sin código | 003 | WIP 003 deshecho; el cambio sigue aplicado a mano en las BD locales |
| 0005 | pendiente | Extensión `pg_trgm` + índice de búsqueda | 004 | 004 |
| 0006 | pendiente | Tabla de auditoría | 006 | 006 |

> La numeración es global y **no se reutiliza**. La fase foundational de 001 creó las tablas
> de 003 y 005 para no bloquear las user stories; la propiedad del esquema queda en el spec
> que lo usa.
>
> La 0004 se retiró junto con el WIP de 003 (nunca se publicó); el replanteo define si vuelve con
> ese número o con uno nuevo.

## Convenciones de tareas

- Las specs creadas después del split reinician en `T001` y llevan una tabla de trazabilidad
  con el ID anterior cuando aplica.
- 001 conserva los IDs originales `T001–T025`; 002 renumera `T026–T031` → `T001–T006` con
  trazabilidad; 003 renumera `T032–T036` → `T001–T005` con trazabilidad.
- Cada spec cierra con sus gates: `cargo test`, `cargo sqlx prepare --check`, `npm test`,
  `npm run check`, `npm run build` cuando corresponda.

## Cierre transversal del Módulo 1 (ex T048–T052)

Estas tareas no pertenecen a un caso de uso; se ejecutan cuando 001–006 estén cerradas.

- [ ] **C1** Actualizar `README.md` de la raíz (setup local, env, comandos, mapa de specs).
- [ ] **C2** Correr los gates completos del monorepo y el grep de `unwrap`/`panic!`/`expect`.
- [ ] **C3** Ejecutar los escenarios V1–V8 de los quickstarts de 001–005 y registrar resultados.
- [ ] **C4** Revisar mensajes de error accionables en español (Principio III).
- [ ] **C5** Commit final de cierre del Módulo 1 y actualización de estado de las specs.

## Fuera de alcance del Módulo 1

Movimientos de stock, transferencias entre depósitos, facturación, reportes financieros,
notificaciones de stock mínimo, integración con balanzas/impresoras de etiquetas, importación
masiva de productos (CSV/Excel) e IA/chatbot de inventario.

## Decisiones de la partición

- **`bad` no es un SKU inválido**: `bad` normaliza a `BAD`, que cumple el regex del contrato.
  El caso inválido de T032 se corrigió a `bad!`; es un error del texto original de la tarea,
  no un cambio de requisito.
- **FR-008a** queda entera en 001 porque describe el ciclo de vida de la contraseña temporal;
  002 la referencia para el alta.
- **Auditoría**: el spec original solo la menciona en US6 y en Supuestos; 006 la formaliza
  como FR-019 (derivado), sin ampliar alcance.
