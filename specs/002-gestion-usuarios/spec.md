# Especificación de Funcionalidad: Gestión de usuarios

**Spec**: `002-gestion-usuarios` | **Caso de uso**: US2 | **Prioridad**: P1

**Creado**: 2026-10-25 (retro-documentada) | **Estado**: Implementado; ajustes de generación y visualización de credenciales (2026-11) planificados en Phase 2 de [tasks.md](./tasks.md)

**Origen**: dividida de la spec original del Módulo 1 (`001-user-auth-catalog`, 2026-10-12). Los textos de escenarios, requisitos y criterios se conservan adaptados a la nueva operativa de alta.

**Entrada**: US2 de la especificación original: "Como Administrador, quiero crear, editar, desactivar y reactivar usuarios con rol y estado, para mantener el personal registrado sin bajas físicas que borren historial."

**Depende de**: [001-autenticacion-sesion](../001-autenticacion-sesion/spec.md) — identidad, sesiones y modelo RBAC (`FR-003`, `FR-008a`).

## Clarifications

### Session 2026-10-12 / Actualización 2026-11

- Q: ¿Quién define la contraseña inicial al crear un usuario y cómo se entrega? → A: El Administrador puede elegir entre generar una contraseña manualmente o de forma automática por el sistema (14 caracteres, con mayúscula y dígito). Al finalizar la creación, el sistema muestra obligatoriamente en pantalla un mensaje con el correo y la contraseña temporal para su entrega inmediata, además de enviarla por correo electrónico si el transporte está configurado. El usuario debe cambiarla obligatoriamente en su primer ingreso (`password_temporal = true`).

> Las clarificaciones de login genérico, sesión única y familias de rol pertenecen a 001.

## Escenarios de Usuario y Pruebas *(obligatorio)*

### User Story 2 - Gestión de usuarios por el Administrador (Prioridad: P1)

Como **Administrador**, quiero crear, editar, desactivar y reactivar usuarios con rol y estado, para mantener el personal registrado sin bajas físicas que borren historial.

**Por qué esta prioridad**: El alta de personal es prerrequisito operativo; la baja lógica preserva trazabilidad de movimientos de stock realizados por ese usuario.

**Prueba independiente**: Se prueba creando un usuario, asignándole rol Operario, desactivándolo y confirmando que no puede iniciar sesión pero sus movimientos pasados siguen siendo consultables. *(La consulta del historial de movimientos se verifica en 004/005; 002 cubre el alta, la baja lógica y el bloqueo de login.)*

**Escenarios de aceptación**:

1. **Dado** un Administrador autenticado en el panel de creación de usuarios, **Cuando** ingresa el correo, nombre, rol y selecciona el método de contraseña (manual ingresando los caracteres o automática generada por el sistema), **Entonces** se crea el usuario, el sistema muestra en pantalla un mensaje emergente con el correo y la contraseña temporal para su entrega inmediata, y el usuario queda activo para iniciar sesión con cambio obligatorio pendiente.
2. **Dado** un usuario que se autentica por primera vez con contraseña temporal, **Cuando** completa el cambio obligatorio de contraseña, **Entonces** la nueva contraseña queda activa y la temporal deja de ser válida.
3. **Dado** dos intentos de creación con el mismo correo, **Cuando** se envía el segundo, **Entonces** se rechaza por unicidad de correo con mensaje claro.
4. **Dado** un usuario activo con sesión abierta, **Cuando** el Administrador lo desactiva, **Entonces** su sesión actual queda invalidada (o su siguiente request es rechazada) y no puede volver a iniciar sesión.
5. **Dado** un usuario desactivado, **Cuando** el Administrador lo reactiva, **Entonces** puede volver a iniciar sesión; si olvidó su contraseña, el Administrador puede generar una nueva temporal (manual o automática) visualizándola en pantalla.
6. **Dado** un Operario autenticado, **Cuando** intenta acceder a la gestión de usuarios, **Entonces** se deniega el acceso.

### Casos Borde

- **Desactivación de usuario activo en sesión**: al desactivar, la sesión vigente queda inválida en el siguiente request (o de inmediato); no debe quedar una sesión "fantasma" que siga moviendo stock.
- **Último gestor activo (familia GESTION)**: el sistema no permite desactivar (ni eliminar lógicamente) ni degradar de rol al único gestor activo —Administrador o Gerente—, para no dejar el sistema sin administración.
- **Correo inválido o ya registrado**: validación de formato y unicidad en el momento del guardado, con mensaje claro.
- **Contraseña manual fuera de política**: si el Administrador elige la opción manual y la contraseña no cumple la política (mínimo 8, una mayúscula y un dígito), el alta se rechaza con mensaje claro y no se crea el usuario.

> Los casos borde de credenciales en claro y sesión única pertenecen a 001.

## Requisitos *(obligatorio)*

### Requisitos Funcionales

- **FR-004**: El sistema DEBE garantizar unicidad del correo electrónico de usuario.
- **FR-005**: El sistema DEBE soportar estado Activo/Inactivo mediante baja lógica; los usuarios inactivos no pueden iniciar sesión y sus sesiones activas se invalidan al desactivarse.
- **FR-006**: El sistema DEBE impedir que se desactive o se degrade de rol al último gestor activo de la familia GESTION (Administrador o Gerente). El código de error del contrato sigue siendo `ultimo_administrador` por compatibilidad.
- **FR-007**: El CRUD de usuarios DEBE ser exclusivo del rol Administrador/Gerente.
- **FR-008**: El sistema DEBE validar el formato del correo electrónico y rechazar guardados inválidos con mensaje claro.
- **FR-008b** (Gestión de credenciales temporales): Al crear un usuario, el Administrador DEBE poder optar entre generar una contraseña de forma manual o automática (14 caracteres, con al menos una mayúscula y un dígito). El sistema DEBE mostrar en pantalla de forma clara (mediante modal o alerta visual) el correo y la contraseña temporal generada para que el Administrador pueda copiarla y entregarla de inmediato. Adicionalmente, se enviará por el transporte de correo configurado. El usuario quedará con el flag `password_temporal = true` exigiendo cambio obligatorio en el primer ingreso.

> **Nota de identificación**: la elección manual/automática y la visualización en pantalla se
> especifican acá como parte de la operativa de **alta y reseteo propios de 002**; el ciclo de
> vida de la contraseña temporal (entrega por correo, cambio obligatorio, reseteo en 001) es la
> `FR-008a` de [001](../001-autenticacion-sesion/spec.md), que esta spec consume.

> La estructura de `usuarios`/`sesiones` y el modelo RBAC son propiedad de [001/data-model.md](../001-autenticacion-sesion/data-model.md); acá se define su ciclo de vida.

### Entidades Clave

- **Usuario**: identidad de persona en el sistema; atributos clave: nombre, correo (único), credenciales (no expuestas), rol (Administrador/Gerente u Operario/Cajero), estado (Activo/Inactivo).
- **Rol**: definición de permisos; en este módulo, dos familias operativas: gestión (admin/gerente) y operación (operario/cajero).

## Criterios de Éxito

- **SC-001**: Un Administrador puede crear un usuario operativo (definiendo contraseña manual o automática) y visualizar inmediatamente las credenciales en pantalla, pudiendo el usuario iniciar sesión en menos de 2 minutos desde el alta.

## Supuestos

- **Contraseñas**: almacenamiento con hash irreversible; política básica (longitud mínima 8, al menos una mayúscula y un dígito). La contraseña inicial puede ser ingresada manualmente por el admin o generada de forma automática por el sistema y visualizada en pantalla de manera obligatoria al crear el registro. El usuario debe cambiarla en el primer ingreso.
- **Roles**: se modelan dos familias (Administrador/Gerente y Operario/Cajero). Administrador y Gerente tienen los mismos permisos en este módulo.
- **Idioma de UI**: español (es-AR por defecto), alineado con el resto del proyecto.

**Fuera de alcance global**: ver [specs/README.md](../README.md).

## Trazabilidad

- Tareas originales: T026–T031 → `T001–T006` en [tasks.md](./tasks.md); ajustes 2026-11 → Phase 2 (`T008` en adelante).
- Commits: `f1fc3e8` (implementación), `977bd90` (`DebeCambiarPassword` y extractores), `8fa8390` (paginación).