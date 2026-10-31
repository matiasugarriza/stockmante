# Especificación de Funcionalidad: Autenticación y sesión segura

**Spec**: `001-autenticacion-sesion` | **Caso de uso**: US1 | **Prioridad**: P1 (MVP)

**Creado**: 2026-10-12 | **Dividido de la spec original**: 2026-10-25 | **Estado**: Implementado con correcciones abiertas (T026–T032); enmendado 2026-10-28 para agregar registro y SSO (Google/Microsoft)

**Rama**: `main` (el repositorio trabajó sin rama de feature)

**Entrada** (pedido original, verbatim): "Crear la especificación funcional para el Módulo 1: Gestión de Usuarios, Autenticación y Catálogo Base (Productos, Depósitos y Ubicaciones) de Stockmante. Alcance: CRUD de usuarios con roles Administrador/Gerente y Operario/Cajero; autenticación login/logout/sesión con RBAC; unicidad de correo; baja lógica de usuarios; CRUD de productos con SKU único y código de barras opcional/único; búsqueda rápida por SKU/código de barras/nombre/categoría para caja y almacén; baja lógica de productos; CRUD de depósitos y ubicaciones jerárquicas; asignación inicial de ubicaciones por producto; casos de uso para Administrador y Operario; edge cases (SKU duplicado, desactivar usuario en sesión, ubicaciones huérfanas)."

**Alcance de esta spec**: solo autenticación, sesión y el modelo de identidad/RBAC. Los demás
casos de uso se dividieron en `002`–`006` (ver [specs/README.md](../README.md)).

## Clarifications

### Session 2026-10-12

- Q: Cuando el fallo de login es por cuenta inactiva, ¿el mensaje debe ser genérico o específico? → A: Mensaje único genérico para todos los fallos de login ("Credenciales inválidas"), sin distinguir cuenta inexistente, contraseña incorrecta o cuenta inactiva; el estado inactivo se gestiona/consulta en la pantalla de administración de usuarios.
- Q: ¿Un mismo usuario puede tener varias sesiones activas al mismo tiempo (varios navegadores/puestos) o al iniciar sesión se cierra la anterior? → A: Una sola sesión vigente por usuario; un nuevo inicio de sesión invalida la sesión anterior; al desactivar, la sesión vigente se invalida en el siguiente request.
- Q: ¿Gerente y Administrador tienen los mismos permisos o el Gerente tiene un alcance más acotado? → A: Mismos permisos: una sola familia de gestión (Administrador/Gerente); diferenciar roles queda para un módulo posterior si el negocio lo pide.

> La clarificación sobre la contraseña inicial pertenece a `002/FR-008a`; la del formato de SKU a `003`.

## Escenarios de Usuario y Pruebas *(obligatorio)*

### User Story 1 - Autenticación y sesión segura (Prioridad: P1)

Como **Operario/Cajero**, quiero iniciar sesión con mi correo y contraseña para acceder solo a las funciones que mi rol me permite, y cerrar sesión cuando termino mi turno para que nadie más use mi cuenta desde el puesto.

**Por qué esta prioridad**: Sin autenticación no hay control de acceso; es la puerta de entrada al resto del sistema y protege la integridad del inventario.

**Prueba independiente**: Se puede probar completamente iniciando sesión con credenciales válidas, verificando que el operario ve solo su alcance de pantallas/acciones, y cerrando sesión; entrega valor (acceso controlado) sin necesidad del catálogo.

**Escenarios de aceptación**:

1. **Dado** un usuario activo con rol Operario, **Cuando** envía correo y contraseña correctos, **Entonces** se crea una sesión y se redirige al panel operativo sin opciones de administración de usuarios.
2. **Dado** un usuario inexistente, contraseña incorrecta o cuenta inactiva, **Cuando** intenta iniciar sesión, **Entonces** se rechaza con el mismo mensaje genérico "Credenciales inválidas" (sin revelar cuál fue el motivo ni si el correo existe).
3. **Dado** un usuario con sesión activa en un puesto, **Cuando** inicia sesión en otro puesto (u otro navegador), **Entonces** la sesión anterior queda invalidada de inmediato y cualquier request con el token viejo es rechazado.
4. **Dado** un usuario con sesión activa, **Cuando** elige "Cerrar sesión", **Entonces** la sesión queda invalidada y cualquier request posterior exige autenticación.
5. **Dado** un visitante sin cuenta, **Cuando** se registra con correo y contraseña válidos, **Entonces** se crea una cuenta pendiente de verificación, se envía un correo de confirmación y no se inicia sesión hasta verificar el correo.
6. **Dado** un usuario con cuenta de Google/Microsoft, **Cuando** elige "Iniciar sesión con Google/Microsoft", **Entonces** se redirige al proveedor, se valida el token recibido y se crea/vincula la cuenta local y la sesión.
7. **Dado** un usuario autenticado, **Cuando** vincula o desvincula una identidad SSO, **Entonces** la operación se registra y no se permite desvincular la última identidad disponible.

### Casos Borde

- **Credenciales en claro**: nunca se muestran ni se guardan en logs; la contraseña temporal solo se transmite por correo al usuario destinatario; el restablecimiento es solo por el Administrador (sin auto-servicio).
- **SSO**: si el proveedor no está disponible, se responde 503 sin filtrar detalles; si el correo del proveedor ya está vinculado a otra cuenta local, se rechaza con 409 `identidad_ya_vinculada`; no se permite desvincular la última identidad disponible.

> El caso borde de desactivación con sesión se verifica en 001 (SC-004); el del último administrador pertenece a `002`; los de SKU, código de barras y producto con referencias a `003`; los de búsqueda a `004`; los de ubicaciones huérfanas a `005`.

## Requisitos *(obligatorio)*

### Requisitos Funcionales

- **FR-001**: El sistema DEBE permitir iniciar sesión con correo electrónico y contraseña; cada usuario DEBE tener como máximo una sesión vigente: un nuevo inicio de sesión invalida la sesión anterior. La sesión se mantiene hasta cierre explícito, inicio de sesión en otro puesto o **expiración absoluta por tiempo** (8 h desde el login por defecto; sin renovación por actividad — ver Supuestos).
- **FR-002**: El sistema DEBE permitir cerrar sesión, invalidando la sesión del cliente.
- **FR-002a**: La cookie de sesión DEBE emitirse con `HttpOnly`, `SameSite=Lax`, `path=/` y, cuando `APP_ENV=production`, `Secure`; nunca DEBE contener datos del usuario más allá del identificador opaco de sesión.
- **FR-003**: El sistema DEBE aplicar control de acceso por rol (RBAC) con dos familias: gestión (Administrador/Gerente, mismos permisos en este módulo) y operación (Operario/Cajero: caja y consulta de catálogo, sin gestión de usuarios ni precios).
- **FR-008a**: Al crear un usuario, el sistema DEBE generar una contraseña temporal (14 caracteres, con al menos una mayúscula y un dígito), enviarla al correo del usuario mediante el transporte configurado (`SMTP_URL`; en desarrollo se usa `CorreoDev`) e impedir el uso del sistema más allá del cambio obligatorio de contraseña hasta completarlo. Mientras `password_temporal` sea verdadero, la sesión solo habilita `GET /auth/sesion` y `POST /auth/cambiar-password`; cualquier otro endpoint autenticado responde 403 `cambio_password_requerido`, con independencia de la familia de rol. El reinicio de contraseña olvidada DEBE poder solicitarlo el Administrador generando una nueva contraseña temporal (no hay auto-servicio de recuperación en este módulo). En producción, si el correo no se puede enviar, el alta DEBE fallar y no dejar la cuenta creada.
- **FR-009 Registro por correo**: El sistema DEBE permitir el auto-registro de usuarios con correo electrónico y contraseña, creando una cuenta en estado pendiente de verificación y enviando un correo de confirmación; la cuenta solo se activa tras verificar el correo. El registro DEBE validar unicidad de correo y la política de contraseña (longitud mínima 8, al menos una mayúscula y un dígito). Un correo ya registrado DEBE responder el mismo mensaje genérico que un registro exitoso (anti-enumeración).
- **FR-010 Inicio de sesión SSO**: El sistema DEBE permitir iniciar sesión mediante SSO con cuentas de Google y Microsoft (OAuth 2.0 / OIDC). El flujo DEBE redirigir al proveedor, validar el token/ID token recibido, vincular o crear la cuenta local según el correo verificado por el proveedor y crear la sesión de la misma forma que el login por contraseña. Si el correo del proveedor ya existe vinculado a otra identidad, DEBE rechazarse con 409 `identidad_ya_vinculada`.
- **FR-011 Vinculación de identidades**: Un usuario autenticado DEBE poder vincular o desvincular una cuenta de Google/Microsoft a su cuenta local; la desvinculación DEBE fallar si es la única identidad disponible (409 `ultima_identidad`). Las credenciales de contraseña son opcionales cuando existe al menos una identidad SSO vinculada.
- **FR-012 Provisión de rol en SSO**: Las cuentas creadas por SSO DEBEN recibir la familia `OPERACION` por defecto; la promoción a `GESTION` queda reservada al Administrador (igual que en 002).

> FR-003 es el modelo de permisos; cada caso de uso declara sus propias filas de la matriz y
> `006` las verifica de forma transversal.

### Superficie de API (amendment SSO/registro)

| Método | Ruta | Permiso | Respuesta |
|---|---|---|---|
| POST | `/api/auth/registro` | público | 202 registro pendiente de verificación |
| GET | `/api/auth/verificar-correo?token=` | público | 200 cuenta activada |
| GET | `/api/auth/sso/{proveedor}/iniciar` | público | 302 redirección al proveedor (google/microsoft) |
| GET | `/api/auth/sso/{proveedor}/callback` | público | 200 sesión creada (cookie) |
| GET | `/api/auth/identidades` | sesión | 200 lista de identidades vinculadas |
| POST | `/api/auth/identidades/{proveedor}/vincular` | sesión | 200 identidad vinculada |
| DELETE | `/api/auth/identidades/{proveedor}` | sesión | 204 identidad desvinculada |

**Errores nuevos**:

| Código | Status | Cuándo |
|---|---|---|
| `correo_ya_registrado` | 202 (mensaje genérico) | El correo ya existe (anti-enumeración). |
| `token_verificacion_invalido` | 422 | Token de verificación inválido o expirado. |
| `proveedor_no_disponible` | 503 | El proveedor SSO no está disponible. |
| `identidad_ya_vinculada` | 409 | El correo del proveedor ya está vinculado a otra cuenta. |
| `ultima_identidad` | 409 | No se puede desvincular la última identidad disponible. |
| `proveedor_no_soportado` | 422 | Proveedor distinto de google/microsoft. |

### Entidades Clave

- **Usuario**: identidad de persona en el sistema; atributos clave: nombre, correo (único), credenciales (no expuestas, opcionales si hay SSO), rol (Administrador/Gerente u Operario/Cajero), estado (Activo/Inactivo), correo verificado.
- **Rol**: definición de permisos; en este módulo, dos familias operativas: gestión (admin/gerente) y operación (operario/cajero).
- **Sesión**: estado de autenticación de un usuario en un momento dado; vincula usuario + vigencia + invalidación.
- **Identidad externa**: vinculación entre una cuenta local y un proveedor SSO (Google/Microsoft); atributos: proveedor, subject (id del proveedor), correo verificado, fecha de vinculación. Un usuario puede tener varias identidades externas, pero cada (proveedor, subject) es único.
- **Token de verificación de correo**: token de un solo uso con expiración para confirmar el correo en el auto-registro.

## Criterios de Éxito

- **SC-003**: El 100% de los intentos de inicio de sesión de usuarios inactivos, inexistentes o con credenciales inválidas son rechazados con el mismo mensaje genérico, sin diferenciar el motivo ni revelar si el correo existe.
- **SC-004**: Cero sesiones activas de usuarios desactivados pueden ejecutar acciones posteriores a la desactivación (validado en pruebas de seguridad). *(La baja lógica la ejecuta 002; la invalidación se verifica en 001 con `backend/tests/unit/test_sesion_unica.rs::desactivar_usuario_deja_sesiones_invalidas`.)*
- **SC-005**: El 100% de los registros por correo requieren verificación antes de activar la cuenta; un correo ya registrado no revela su existencia (mismo mensaje genérico).
- **SC-006**: El 100% de los inicios de sesión SSO validan el token del proveedor y crean/vinculan la cuenta local; un proveedor no disponible responde 503 sin filtrar detalles internos.

## Supuestos

- **Autenticación**: se usa autenticación por correo y contraseña con sesión del servidor, más registro por correo y SSO con Google y Microsoft (OAuth 2.0 / OIDC). MFA queda fuera de este módulo a menos que se agregue como requisito posterior. El mensaje genérico de fallo de login es una excepción deliberada de seguridad (anti-enumeración) al Principio III (Claridad en Mensajes), acotada al endpoint de autenticación. **Esta excepción requiere una enmienda constitucional explícita (propuesta en `tasks.md` T032); hasta que se apruebe se registra como desviación consciente, no como cumplimiento.**
- **Contraseñas**: almacenamiento con hash irreversible; política básica (longitud mínima 8, al menos una mayúscula y un dígito) salvo política corporativa distinta. La contraseña inicial es temporal (14 caracteres, con al menos una mayúscula y un dígito), generada por el sistema y enviada por correo mediante el transporte configurado; en desarrollo se usa el stub de consola (`CorreoDev`) y en producción el envío es obligatorio (si falla, el alta se revierte). El usuario debe cambiarla en el primer ingreso. No hay flujo de "olvidé mi contraseña" de auto-servicio en este módulo: el Administrador genera una nueva temporal. La política de expiración periódica de contraseñas (cada N días) queda fuera de este módulo.
- **Sesión**: expiración **absoluta** por defecto de 8 horas de reloj desde el inicio de sesión (ajustable con `SESSION_EXPIRY_HOURS`), sin renovación por actividad (sin sliding); una sola sesión vigente por usuario (ver FR-001; nuevo login invalida la anterior); el cierre de sesión invalida la sesión en el servidor.
- **Roles**: se modelan dos familias (Administrador/Gerente y Operario/Cajero). Administrador y Gerente tienen los mismos permisos en este módulo (familia de gestión); diferenciar sus permisos queda fuera de alcance salvo enmienda posterior.
- **Idioma de UI**: español (es-AR por defecto), alineado con el resto del proyecto.

**Fuera de alcance global**: ver [specs/README.md](../README.md).

## Correcciones post-análisis (2026-10-25)

- Expiración de sesión: **absoluta** (8 h desde el login), sin sliding (FR-001).
- Cookie de sesión: `HttpOnly`, `SameSite=Lax`, `path=/` y `Secure` en producción (FR-002a).
- Contraseña temporal: 14 caracteres; entrega por correo con transporte real en producción (FR-008a).
- Gate de contraseña temporal: global a todo endpoint autenticado salvo `/auth/sesion` y `/auth/cambiar-password`, sin depender de la familia (FR-008a).
- SC-004: la invalidación se verifica en 001; la baja lógica sigue siendo de 002.
- Correcciones de implementación abiertas: T026–T032 en [tasks.md](./tasks.md).

## Trazabilidad

- Tareas originales: T001–T025 (fases Setup, Foundational y US1).
- Dependen de esta spec: `002`, `003`, `004`, `005`, `006`.
