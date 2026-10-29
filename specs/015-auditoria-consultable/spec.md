# Especificación de Funcionalidad: Auditoría consultable

**Spec**: `015-auditoria-consultable` | **Caso de uso**: US15 | **Prioridad**: P2

**Creado**: 2026-10-26 | **Estado**: Especificado (sin `plan.md` ni `tasks.md`; se generan al
implementar) | **Módulo**: 4 — Gobierno y asistencia

**Origen**: 006 implementó la auditoría básica y dejó explícitamente fuera "log rico y consultas de
auditoría". Con movimientos, compras y ventas, la tabla crece y necesita una superficie de consulta.

**Depende de**: 006 (tabla `auditoria`), 007, 012 y 013 (acciones a cubrir).

## Escenarios de Usuario y Pruebas *(obligatorio)*

### User Story 15 - Investigar quién hizo qué y cuándo (Prioridad: P2)

Como **Gerente**, quiero consultar la auditoría filtrando por tabla, registro, autor, acción y
fechas, para investigar una diferencia de stock o un cambio sensible sin pedir acceso a la base.

**Por qué esta prioridad**: una auditoría que solo se lee por SQL no sirve en la operación diaria;
es el complemento de gobierno que exige un sistema con roles y dinero de por medio.

**Escenario 15.1 — Consulta filtrada**

- **Dado** un gerente,
- **Cuando** consulta `GET /api/auditoria?tabla=productos&registro_id=12`,
- **Entonces** obtiene las filas ordenadas de la más reciente a la más antigua con acción, autor y
  fecha.

**Escenario 15.2 — Filtros combinados**

- **Dado** un rango de fechas y un autor,
- **Cuando** se combinan `usuario_id` y `desde`/`hasta`,
- **Entonces** el resultado respeta todos los filtros y está paginado.

**Escenario 15.3 — Cobertura ampliada**

- **Dado** una entrada de stock, una compra confirmada y una venta,
- **Cuando** se consulta la auditoría,
- **Entonces** aparecen las acciones `CREAR` de `movimientos`, `compras` y `ventas`, además de las
  ya cubiertas por 006.

**Escenario 15.4 — Sin detalle sensible**

- **Dado** una acción auditada,
- **Cuando** se consulta,
- **Entonces** la fila muestra tabla, id, acción, autor y fecha, pero **no** valores anteriores ni
  posteriores ni datos personales.

**Escenario 15.5 — Pantalla**

- **Dado** un gerente,
- **Cuando** abre `/admin/auditoria`,
- **Entonces** ve la tabla con filtros y paginación; una sesión `OPERACION` es redirigida por el
  guard de `/admin`.

**Escenario 15.6 — Operario**

- **Dado** una sesión `OPERACION`,
- **Cuando** llama a la API de auditoría,
- **Entonces** responde **403** `sin_permisos`.

## Requisitos Funcionales

- **FR-060 Consulta.** El sistema DEBE exponer `GET /api/auditoria` paginado con filtros por
  `tabla_ref`, `registro_id`, `usuario_id`, `accion` y rango de fechas, ordenado por fecha
  descendente.
- **FR-061 Cobertura ampliada.** DEBEN auditarse las altas y confirmaciones de movimientos,
  transferencias, conteos, proveedores, compras, clientes y ventas, además de las acciones ya
  cubiertas por 006. Cada fila se escribe en la misma transacción del cambio.
- **FR-062 Retención y permisos.** La auditoría DEBE ser de solo lectura por API (sin endpoints de
  edición ni borrado), DEBE exigir `GESTION` y DEBE conservar las filas aunque el registro
  referenciado se elimine; la política de retención queda fuera de alcance y se documenta como
  deuda operativa.

## Criterios de Éxito Medibles

- **SC-023** Toda acción sensible de las specs 007–013 deja al menos una fila consultable por API
  en menos de 2 segundos con 1.000.000 de filas (prueba con datos sembrados y `EXPLAIN ANALYZE`).

## Supuestos

- No se guardan diffs ni snapshots: la fila sigue siendo `tabla_ref`, `registro_id`, `accion`,
  `usuario_id` y `cuando`.
- La auditoría no se puede editar ni borrar desde la aplicación.
- El filtro por tabla acepta los valores canónicos (`usuarios`, `productos`, `movimientos`,
  `transferencias`, `conteos`, `proveedores`, `compras`, `clientes`, `ventas`).
- No hay exportación de auditoría en esta spec (se evalúa con los reportes si hace falta).

## Fuera de Alcance

- Diferencias campo a campo, valores anteriores y posteriores.
- Retención automática, archivado o purga.
- Alertas en tiempo real sobre acciones sospechosas.
- Exportación CSV de auditoría.

## Dependencias y orden de implementación

1. 006 implementada; 007, 012 y 013 implementadas para cubrir sus acciones.
2. Migración reservada: `0015_auditoria_consultas.sql` (índices por `cuando`, `tabla_ref` y
   `accion`).
3. Requiere extender `services/auditoria.rs` con las nuevas constantes de tabla y acción.
