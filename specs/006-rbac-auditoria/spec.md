# Especificación de Funcionalidad: Control de acceso por roles y auditoría

**Spec**: `006-rbac-auditoria` | **Caso de uso**: US6 | **Prioridad**: P2

**Creado**: 2026-10-25 | **Estado**: Implementado y verificado (2026-10-26)

**Origen**: dividida de la spec original del Módulo 1 (`001-user-auth-catalog`, 2026-10-12).
Los textos de escenarios y criterios se conservan **verbatim**. La auditoría se formaliza como
**FR-019 derivado** (el spec original la menciona en US6 y en Supuestos, pero no le asignó FR).

**Depende de**: 001–005 (verifica la matriz sobre endpoints ya existentes).

## Escenarios de Usuario y Pruebas *(obligatorio)*

### User Story 6 - Control de acceso por roles en catálogo y usuarios (Prioridad: P2)

Como **Gerente/Administrador**, quiero que las acciones sensibles (CRUD de usuarios, precios de venta, bajas de productos) queden reservadas a mi rol, para que el Operario no altere la configuración del negocio.

**Por qué esta prioridad**: El RBAC evita fraudes y errores operativos; se prueba junto a cada área funcional pero se declara como criterio transversal.

**Prueba independiente**: Se prueba denegando cada acción restringida a un Operario autenticado y confirmándola para el Administrador.

**Escenarios de aceptación**:

1. **Dado** un Operario autenticado, **Cuando** intenta crear o editar un producto, **Entonces** se deniega.
2. **Dado** un Operario autenticado, **Cuando** consulta el catálogo para caja, **Entonces** la lectura está permitida si su rol lo habilita.
3. **Dado** un Administrador, **Cuando** realiza cualquiera de las acciones de gestión, **Entonces** se ejecuta y queda registrada la auditoría básica (quién, qué, cuándo) en la medida definida en Supuestos.

### Casos Borde

- **Sesión con rol degradado**: un cambio de rol aplica en el siguiente request (la familia se
  recalcula y se valida contra la fila actual), sin sesiones "fantasma" con permisos viejos.
- **Auditoría**: un rechazo 403 no debe registrar una acción de negocio exitosa; solo se auditan
  las acciones que se completan.

## Requisitos *(obligatorio)*

### Requisitos Funcionales

- **FR-003 (verificación)**: El modelo RBAC de dos familias es propiedad de
  [001](../001-autenticacion-sesion/spec.md). Esta spec **verifica** que la matriz se cumpla en
  todos los endpoints de escritura y en los listados restringidos.
- **FR-019** (derivado): El sistema DEBE registrar al menos autor y timestamp de altas, bajas y
  cambios sensibles de usuarios y productos.

> FR-019 no aparece numerado en el spec original; se deriva de US6 escenario 3 y del Supuesto de
> Auditoría. No amplía alcance: no incluye un log rico ni consultas de auditoría.

### Entidades Clave

- **Auditoría**: fila con tabla de referencia, id del registro, acción, usuario y timestamp.
- **Rol**: familia de permisos (propiedad de 001).

## Criterios de Éxito

- **SC-007**: El 100% de los intentos de borrado físico de productos o depósitos con dependencias son bloqueados con alternativa de baja lógica ofrecida al Administrador.
- **SC-008**: Ningún usuario con rol Operario puede completar (no solo intentar) una operación de gestión de usuarios, edición de precios o baja de productos.

## Supuestos

- **Auditoría**: se registra al menos autor y timestamp de altas/bajas/cambios sensibles de
  usuarios y productos; un log de auditoría rico es un candidato a módulo posterior.
- **Alcance de la verificación**: todos los endpoints de escritura de 001–005 y los listados
  restringidos (`GET /usuarios`, `GET /productos`).
- **Migración**: `0008` (la `0006` quedó ocupada por el catálogo de 003 y la `0007` por la
  búsqueda de 004; la numeración global no se reutiliza).

## Contrato preliminar

Esta spec no agrega endpoints públicos de negocio. Agrega:

- Servicio interno `backend/src/services/auditoria.rs`.
- Migración `migrations/0008_auditoria.sql` con una tabla simple
  (`tabla_ref`, `registro_id`, `accion`, `usuario_id`, `cuando`).
- Invocaciones en usuarios (desactivar/reactivar/update) y productos (update de precio,
  desactivar).
- Test de integración `backend/tests/integration/test_rbac_matriz.rs`: cada endpoint de escritura
  con sesión OPERACION → 403; con GESTION → no 403.
- Ajuste de layouts frontend (`admin` vs `caja`) para ocultar rutas de gestión a operación (UX; el
  backend ya deniega).

El contrato definitivo, el plan y las tareas se generan al arrancar la spec.

## Fuera de alcance

- Log de auditoría rico, consultas y reportes de auditoría.
- Diferenciación de permisos entre Administrador y Gerente.
- Nuevos módulos funcionales.
