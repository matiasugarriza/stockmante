# Especificación de Funcionalidad: Gestión de usuarios

**Spec**: `002-gestion-usuarios` | **Caso de uso**: US2 | **Prioridad**: P1

**Creado**: 2026-10-25 (retro-documentada) | **Estado**: Implementado

**Origen**: dividida de la spec original del Módulo 1 (`001-user-auth-catalog`, 2026-10-12).
Los textos de escenarios, requisitos y criterios se conservan **verbatim**; solo se separaron
por caso de uso.

**Entrada**: US2 de la especificación original: "Como Administrador, quiero crear, editar,
desactivar y reactivar usuarios con rol y estado, para mantener el personal registrado sin bajas
físicas que borren historial."

**Depende de**: [001-autenticacion-sesion](../001-autenticacion-sesion/spec.md) — identidad,
sesiones y modelo RBAC (FR-003, FR-008a).

## Clarifications

### Session 2026-10-12

- Q: ¿Quién define la contraseña inicial al crear un usuario y qué pasa si el operario la olvida? → A: El sistema genera una contraseña temporal y la envía por correo; el usuario debe cambiarla en el primer ingreso; la recuperación de contraseña olvidada queda fuera de este módulo (solicitar reinicio al Administrador).

> Las clarificaciones de login genérico, sesión única y familias de rol pertenecen a 001.

## Escenarios de Usuario y Pruebas *(obligatorio)*

### User Story 2 - Gestión de usuarios por el Administrador (Prioridad: P1)

Como **Administrador**, quiero crear, editar, desactivar y reactivar usuarios con rol y estado, para mantener el personal registrado sin bajas físicas que borren historial.

**Por qué esta prioridad**: El alta de personal es prerrequisito operativo; la baja lógica preserva trazabilidad de movimientos de stock realizados por ese usuario.

**Prueba independiente**: Se prueba creando un usuario, asignándole rol Operario, desactivándolo y confirmando que no puede iniciar sesión pero sus movimientos pasados siguen siendo consultables.

**Escenarios de aceptación**:

1. **Dado** un Administrador autenticado, **Cuando** crea un usuario con correo, nombre y rol, **Entonces** el sistema genera una contraseña temporal, la envía al correo del nuevo usuario y el usuario queda activo y puede iniciar sesión con esa contraseña.
2. **Dado** un usuario que se autentica por primera vez con contraseña temporal, **Cuando** completa el cambio obligatorio de contraseña, **Entonces** la nueva contraseña queda activa y la temporal deja de ser válida.
3. **Dado** dos intentos de creación con el mismo correo, **Cuando** se envía el segundo, **Entonces** se rechaza por unicidad de correo con mensaje claro.
4. **Dado** un usuario activo con sesión abierta, **Cuando** el Administrador lo desactiva, **Entonces** su sesión actual queda invalidada (o su siguiente request es rechazada) y no puede volver a iniciar sesión.
5. **Dado** un usuario desactivado, **Cuando** el Administrador lo reactiva, **Entonces** puede volver a iniciar sesión; si olvidó su contraseña, el Administrador puede generar una nueva temporal por el mismo medio del alta.
6. **Dado** un Operario autenticado, **Cuando** intenta acceder a la gestión de usuarios, **Entonces** se deniega el acceso.

### Casos Borde

- **Desactivación de usuario activo en sesión**: al desactivar, la sesión vigente queda inválida en el siguiente request (o de inmediato); no debe quedar una sesión "fantasma" que siga moviendo stock.
- **Último Administrador activo**: el sistema no permite desactivar (ni eliminar lógicamente) al único Administrador activo, para no dejar el sistema sin administración.
- **Correo inválido o ya registrado**: validación de formato y unicidad en el momento del guardado, con mensaje claro.

> Los casos borde de credenciales en claro y sesión única pertenecen a 001.

## Requisitos *(obligatorio)*

### Requisitos Funcionales

- **FR-004**: El sistema DEBE garantizar unicidad del correo electrónico de usuario.
- **FR-005**: El sistema DEBE soportar estado Activo/Inactivo mediante baja lógica; los usuarios inactivos no pueden iniciar sesión y sus sesiones activas se invalidan al desactivarse.
- **FR-006**: El sistema DEBE impedir que se desactive al último Administrador activo.
- **FR-007**: El CRUD de usuarios DEBE ser exclusivo del rol Administrador/Gerente.
- **FR-008**: El sistema DEBE validar el formato del correo electrónico y rechazar guardados inválidos con mensaje claro.

> FR-008a (contraseña temporal, entrega por correo, cambio obligatorio y reseteo) es
> propiedad de [001](../001-autenticacion-sesion/spec.md). Esta spec la consume en el alta y en
> el reinicio de contraseña.

### Entidades Clave

- **Usuario**: identidad de persona en el sistema; atributos clave: nombre, correo (único), credenciales (no expuestas), rol (Administrador/Gerente u Operario/Cajero), estado (Activo/Inactivo).
- **Rol**: definición de permisos; en este módulo, dos familias operativas: gestión (admin/gerente) y operación (operario/cajero).

> La estructura de `usuarios`/`sesiones` y el modelo RBAC son propiedad de
> [001/data-model.md](../001-autenticacion-sesion/data-model.md); acá se define su ciclo de vida.

## Criterios de Éxito

- **SC-001**: Un Administrador puede crear un usuario operativo y este puede iniciar sesión en menos de 2 minutos desde el alta (incluyendo la primera autenticación).

## Supuestos

- **Contraseñas**: almacenamiento con hash irreversible; política básica (longitud mínima 8, al menos una mayúscula y un dígito). La contraseña inicial es temporal, generada por el sistema y enviada por correo; el usuario debe cambiarla en el primer ingreso. No hay flujo de "olvidé mi contraseña" de auto-servicio: el Administrador genera una nueva temporal.
- **Roles**: se modelan dos familias (Administrador/Gerente y Operario/Cajero). Administrador y Gerente tienen los mismos permisos en este módulo.
- **Idioma de UI**: español (es-AR por defecto), alineado con el resto del proyecto.

**Fuera de alcance global**: ver [specs/README.md](../README.md).

## Trazabilidad

- Tareas originales: T026–T031 → `T001–T006` en [tasks.md](./tasks.md).
- Commits: `f1fc3e8` (implementación), `977bd90` (FR-008a y extractores), `8fa8390` (paginación).
