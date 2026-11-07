# Especificación de Funcionalidad: Control de acceso por roles y auditoría

**Spec**: `006-rbac-auditoria` | **Caso de uso**: US6 | **Prioridad**: P2

**Creado**: 2026-10-25 | **Estado**: Implementado y verificado (2026-10-26); **ampliado 2026-11-07**:
auditoría de categorías, depósitos, ubicaciones y asignaciones producto–ubicación (FR-019).

**Origen**: dividida de la spec original del Módulo 1 (`001-user-auth-catalog`, 2026-10-12).
Los textos de escenarios y criterios se conservan, con la ampliación de alcance documentada en
Clarifications (2026-11-07). La auditoría se formaliza como **FR-019 derivado** (el spec original la
menciona en US6 y en Supuestos, pero no le asignó FR).

**Depende de**: 001–005 (verifica la matriz sobre endpoints ya existentes).

## Clarifications

### Session 2026-11-07

- Q: ¿La auditoría de 006 debe limitarse a usuarios y productos, o extenderse a categorías, depósitos y ubicaciones? → A: **Extenderla a todas las escrituras de gestión de 001–005**: usuarios, productos, categorías, depósitos, ubicaciones y asignación producto–ubicación. Quedan fuera las lecturas y los eventos de autenticación (login/logout/cambio de contraseña voluntario).

## Escenarios de Usuario y Pruebas *(obligatorio)*

### User Story 6 - Control de acceso por roles en catálogo y usuarios (Prioridad: P2)

Como **Gerente/Administrador**, quiero que las acciones sensibles (CRUD de usuarios, precios de venta, bajas de productos) queden reservadas a mi rol, para que el Operario no altere la configuración del negocio.

**Por qué esta prioridad**: El RBAC evita fraudes y errores operativos; se prueba junto a cada área funcional pero se declara como criterio transversal.

**Prueba independiente**: Se prueba denegando cada acción restringida a un Operario autenticado y confirmándola para el Administrador.

**Escenarios de aceptación**:

1. **Dado** un Operario autenticado, **Cuando** intenta crear o editar un producto, **Entonces** se deniega.
2. **Dado** un Operario autenticado, **Cuando** consulta el catálogo para caja (`GET /api/productos/buscar` o `GET /api/productos/{id}`), **Entonces** la lectura está permitida si su rol lo habilita.
3. **Dado** un Administrador, **Cuando** realiza una escritura de gestión sobre usuarios, productos, categorías, depósitos, ubicaciones o asignaciones producto–ubicación, **Entonces** se ejecuta y queda registrada la auditoría básica (quién, qué, cuándo) con la acción correspondiente.

### Casos Borde

- **Sesión con rol degradado**: un cambio de rol aplica en el siguiente request (la familia se
  recalcula y se valida contra la fila actual), sin sesiones "fantasma" con permisos viejos.
- **Auditoría**: ningún rechazo (401/403/409/422) registra una acción de negocio; solo se auditan
  las acciones que se completan.

## Requisitos *(obligatorio)*

### Requisitos Funcionales

- **FR-003 (verificación)**: El modelo RBAC de dos familias es propiedad de
  [001](../001-autenticacion-sesion/spec.md). Esta spec **verifica** que la matriz se cumpla en
  todos los endpoints de escritura y en los listados restringidos.
- **FR-019** (derivado): El sistema DEBE registrar al menos autor, acción, tabla/registro y timestamp
  de **toda escritura de gestión de 001–005**: usuarios (alta, edición, baja lógica, reactivación y
  contraseña temporal), productos (alta, edición, baja lógica y reactivación), categorías (alta,
  edición, baja lógica y reactivación), depósitos y ubicaciones (alta, edición, borrado físico
  permitido, baja lógica y reactivación) y asignación producto–ubicación. NO DEBE auditar lecturas,
  login/logout ni cambio de contraseña voluntario.

> FR-019 no aparece numerado en el spec original; se deriva de US6 escenario 3 y del Supuesto de
> Auditoría. La ampliación 2026-11-07 extiende el alcance a todas las escrituras de gestión de
> 001–005, sin agregar un log rico ni consultas de auditoría.

### Entidades Clave

- **Auditoría**: fila con tabla de referencia, id del registro, acción, usuario y timestamp.
- **Rol**: familia de permisos (propiedad de 001).

## Criterios de Éxito

- **SC-007**: Todo intento de borrado físico de depósitos/ubicaciones **con dependencias** cubierto
  por la suite (depósito con ubicaciones, ubicación con hijos o asignaciones) se bloquea con 409 y
  la baja lógica se ofrece como alternativa al Administrador. Los productos no exponen borrado
  físico (003 usa baja lógica): su alternativa es `POST /api/productos/{id}/desactivar`.
- **SC-008**: Ningún usuario con rol Operario puede completar (no solo intentar) una operación de gestión de usuarios, edición de precios o baja de productos.
- **SC-030**: Toda escritura de gestión de 001–005 que se completa deja exactamente una fila de
  auditoría con autor, acción y timestamp; ninguna escritura rechazada (401/403/409/422) deja fila.

## Supuestos

- **Auditoría**: se registra autor, acción, tabla/registro y timestamp de toda escritura de gestión
  de 001–005; no se guardan diffs ni valores. Un log rico, consultas y reportes quedan para un
  módulo posterior.
- **Acciones auditadas**: `CREAR`, `ACTUALIZAR`, `DESACTIVAR`, `REACTIVAR` y `PASSWORD_TEMPORAL`
  para usuarios; `CREAR`, `ACTUALIZAR`, `DESACTIVAR` y `REACTIVAR` para productos y categorías;
  `CREAR`, `ACTUALIZAR`, `ELIMINAR`, `DESACTIVAR` y `REACTIVAR` para depósitos y ubicaciones; y
  `ASIGNAR_UBICACIONES` sobre el producto para `PUT /api/productos/{id}/ubicaciones`.
- **Asignaciones**: se audita el producto afectado, no la tabla puente: `tabla_ref = productos`,
  `accion = ASIGNAR_UBICACIONES`, `registro_id = producto_id`. Un conjunto vacío también se audita.
- **No auditado**: lecturas, login/logout, cambio de contraseña voluntario, movimientos de stock,
  ventas, compras y acciones de bootstrap; esos módulos tienen su propia auditoría o no son
  escrituras de gestión de 001–005.
- **Alcance de la verificación**: todos los endpoints de escritura de 001–005 y los listados
  restringidos (`GET /usuarios`, `GET /productos`, `GET /productos/ubicaciones`, `GET /depositos` y
  anidados); las lecturas de caja (`GET /productos/{id}`, `/productos/buscar`, categorías) se
  verifican como permitidas a OPERACION.
- **Migración**: `0008` (la `0006` quedó ocupada por el catálogo de 003 y la `0007` por la
  búsqueda de 004; la numeración global no se reutiliza).

## Contrato preliminar

Esta spec no agrega endpoints públicos de negocio. Agrega:

- Servicio interno `backend/src/services/auditoria.rs`.
- Migración `migrations/0008_auditoria.sql` con una tabla simple
  (`tabla_ref`, `registro_id`, `accion`, `usuario_id`, `cuando`).
- Invocaciones de auditoría en usuarios, productos, categorías, depósitos, ubicaciones y
  asignaciones producto–ubicación; la lista completa de acciones está en
  [contracts/api.md](./contracts/api.md).
- Test de integración `backend/tests/integration/test_rbac_matriz.rs`: cada endpoint de escritura
  con sesión OPERACION → 403; con GESTION → no 403.
- Ajuste de layouts frontend (`admin` vs `caja`) para ocultar rutas de gestión a operación (UX; el
  backend ya deniega).

El contrato definitivo está en [contracts/api.md](./contracts/api.md); el plan y las tareas viven en esta carpeta.

## Fuera de alcance

- Log de auditoría rico, consultas y reportes de auditoría.
- Diferenciación de permisos entre Administrador y Gerente.
- Auditoría de movimientos de stock, ventas, compras y eventos de autenticación (login/logout,
  cambio voluntario de contraseña).
- Nuevos módulos funcionales.
