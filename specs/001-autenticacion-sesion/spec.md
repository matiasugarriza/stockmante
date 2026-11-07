# Especificación de Funcionalidad: Autenticación y sesión segura

**Spec**: `001-autenticacion-sesion` | **Caso de uso**: US1 | **Prioridad**: P1 (MVP)

**Creado**: 2026-10-12 | **Dividido de la spec original**: 2026-10-25 | **Estado**: En ajuste y corrección (incorporación de formulario de registro, visibilidad condicional de SSO y generación/visualización de credenciales temporales)

**Rama**: `main` (el repositorio trabajó sin rama de feature)

**Entrada** (pedido original, verbatim): "Crear la especificación funcional para el Módulo 1: Gestión de Usuarios, Autenticación y Catálogo Base (Productos, Depósitos y Ubicaciones) de Stockmante. Alcance: CRUD de usuarios con roles Administrador/Gerente y Operario/Cajero; autenticación login/logout/sesión con RBAC; unicidad de correo; baja lógica de usuarios; CRUD de productos con SKU único y código de barras opcional/único; búsqueda rápida por SKU/código de barras/nombre/categoría para caja y almacén; baja lógica de productos; CRUD de depósitos y ubicaciones jerárquicas; asignación inicial de ubicaciones por producto; casos de uso para Administrador y Operario; edge cases (SKU duplicado, desactivar usuario en sesión, ubicaciones huérfanas)."

**Alcance de esta spec**: solo autenticación, sesión y el modelo de identidad/RBAC. Los demás casos de uso se dividieron en `002`–`006` (ver [specs/README.md](../README.md)).

---

## Clarifications

### Session 2026-10-12

- Q: Cuando el fallo de login es por cuenta inactiva, ¿el mensaje debe ser genérico o específico? → A: Mensaje único genérico para todos los fallos de login ("Credenciales inválidas"), sin distinguir cuenta inexistente, contraseña incorrecta o cuenta inactiva; el estado inactivo se gestiona/consulta en la pantalla de administración de usuarios.
- Q: ¿Un mismo usuario puede tener varias sesiones activas al mismo tiempo (varios navegadores/puestos) o al iniciar sesión se cierra la anterior? → A: Una sola sesión vigente por usuario; un nuevo inicio de sesión invalida la sesión anterior; al desactivar, la sesión vigente se invalida en el siguiente request.
- Q: ¿Gerente y Administrador tienen los mismos permisos o el Gerente tiene un alcance más acotado? → A: Mismos permisos: una sola familia de gestión (Administrador/Gerente); diferenciar roles queda para un módulo posterior si el negocio lo pide.

> La clarificación sobre la contraseña inicial pertenece a `002/FR-008b`; la del formato de SKU a `003`.

---

## Escenarios de Usuario y Pruebas _(obligatorio)_

### User Story 1 - Autenticación y sesión segura (Prioridad: P1)

Como **Operario/Cajero**, quiero iniciar sesión con mi correo y contraseña para acceder solo a las funciones que mi rol me permite, y cerrar sesión cuando termino mi turno para que nadie más use mi cuenta desde el puesto.

**Por qué esta prioridad**: Sin autenticación no hay control de acceso; es la puerta de entrada al resto del sistema y protege la integridad del inventario.

**Prueba independiente**: Se puede probar completamente iniciando sesión con credenciales válidas, verificando que el operario ve solo su alcance de pantallas/acciones, y cerrando sesión; entrega valor (acceso controlado) sin necesidad del catálogo.

**Escenarios de aceptación**:

1. **Dado** un usuario activo con rol Operario, **Cuando** envía correo y contraseña correctos, **Entonces** se crea una sesión y se redirige al panel operativo sin opciones de administración de usuarios.
2. **Dado** un usuario inexistente, contraseña incorrecta o cuenta inactiva, **Cuando** intenta iniciar sesión, **Entonces** se rechaza con el mismo mensaje genérico "Credenciales inválidas" (sin revelar cuál fue el motivo ni si el correo existe).
3. **Dado** un usuario con sesión activa en un puesto, **Cuando** inicia sesión en otro puesto (u otro navegador), **Entonces** la sesión anterior queda invalidada de inmediato y cualquier request con el token viejo es rechazado.
4. **Dado** un usuario con sesión activa, **Cuando** elige "Cerrar sesión", **Entonces** la sesión queda invalidada y cualquier request posterior exige autenticación.
5. **Dado** un visitante sin cuenta, **Cuando** accede al **formulario de registro** e ingresa correo y contraseña válidos, **Entonces** se crea una cuenta pendiente de verificación, se envía un correo de confirmación y no se inicia sesión hasta verificar el correo.
6. **Dado** un usuario con cuenta de Google/Microsoft, **Cuando** el proveedor está configurado y activo, **Y** elige "Iniciar sesión con Google/Microsoft", **Entonces** se redirige al proveedor, se valida el token recibido y se crea/vincula la cuenta local y la sesión.
7. **Dado** que las credenciales o API keys de un proveedor SSO (ej. Google) no están configuradas o el servicio falla, **Cuando** se renderiza la interfaz de login o registro, **Entonces** las opciones/botones de SSO se ocultan automáticamente en la UI y el backend rechaza las peticiones devolviendo un estado controlado.
8. **Dado** un Administrador creando un nuevo usuario, **Cuando** selecciona generación de contraseña manual (ingresándola) o automática, **Entonces** el sistema la procesa, crea el usuario y **muestra un mensaje emergente / en pantalla** con el correo y la contraseña temporal generada para su entrega inmediata.

### Casos Borde

- **Credenciales en claro**: nunca se guardan en logs; la contraseña temporal se muestra en pantalla al creador (admin) y opcionalmente se transmite por correo; el restablecimiento es solo por el Administrador.
- **SSO**: si el proveedor no está configurado o no está disponible, se oculta en frontend y se responde 503/422 controlado en backend sin filtrar detalles; si el correo del proveedor ya está vinculado a otra cuenta local, se rechaza con 409 `identidad_ya_vinculada`.

---

## Requisitos _(obligatorio)_

### Requisitos Funcionales

- **FR-001**: El sistema DEBE permitir iniciar sesión con correo electrónico y contraseña; cada usuario DEBE tener como máximo una sesión vigente: un nuevo inicio de sesión invalida la sesión anterior. La sesión se mantiene hasta cierre explícito, inicio de sesión en otro puesto o expiración absoluta por tiempo (8 h desde el login por defecto).
- **FR-002**: El sistema DEBE permitir cerrar sesión, invalidando la sesión del cliente.
- **FR-002a**: La cookie de sesión DEBE emitirse con `HttpOnly`, `SameSite=Lax`, `path=/` y, cuando `APP_ENV=production`, `Secure`.
- **FR-003**: El sistema DEBE aplicar control de acceso por rol (RBAC) con dos familias: gestión (Administrador/Gerente) y operación (Operario/Cajero).
- **FR-008a**: Al crear un usuario, el Administrador DEBE poder optar entre **generar una contraseña de forma manual** o **automática** (14 caracteres, con al menos una mayúscula y un dígito). El sistema DEBE **imprimir o mostrar en pantalla un mensaje claro con el correo y la contraseña temporal** creada. Asimismo, enviarla al correo mediante el transporte configurado (`SMTP_URL`; en desarrollo `CorreoDev`). El usuario deberá cambiarla obligatoriamente en su primer ingreso (`password_temporal = true`).
  > **Visualización**: el modal copiable de correo + clave se implementa y verifica en [002/FR-008b](../002-gestion-usuarios/spec.md); esta FR rige en 001 la generación, el envío por correo y el cambio obligatorio (dueños: `specs/README.md` §FR-008a/FR-008b — evita implementación doble).
- **FR-070 Formulario de Registro**: El sistema DEBE implementar un **Formulario de Registro** accesible públicamente que permita el auto-registro con correo y contraseña, validando unicidad de correo y política de contraseña (mínimo 8 caracteres, una mayúscula y un dígito). Crea cuenta pendiente de verificación y aplica anti-enumeración respondiendo un mensaje genérico si el correo ya existe.
- **FR-071 Inicio de sesión SSO (Google y Microsoft)**: El sistema DEBE permitir iniciar sesión mediante SSO. El backend DEBE exponer `GET /api/auth/sso/config` (público) que devuelva el estado de disponibilidad de cada proveedor (`google_disponible`, `microsoft_disponible`) para que el frontend decida qué botones renderizar. El frontend DEBE **ocultar dinámicamente los botones y opciones de SSO** si las API keys/credenciales del proveedor no están configuradas en el entorno o si el servicio de salud del proveedor falla. Si está activo, redirige, valida el token/ID token, y vincula/crea la cuenta local. Si el correo ya pertenece a otra identidad distinta, responde 409 `identidad_ya_vinculada`.
- **FR-072 Vinculación de identidades**: Un usuario autenticado DEBE poder vincular o desvincular una cuenta de Google/Microsoft; la desvinculación falla si es la única identidad y no posee contraseña local (409 `ultima_identidad`).
- **FR-073 Provisión de rol en SSO**: Las cuentas creadas por SSO DEBEN recibir la familia `OPERACION` por defecto.

### Superficie de API

| Método | Ruta                                         | Permiso | Descripción / Respuesta                                                |
| ------ | -------------------------------------------- | ------- | ---------------------------------------------------------------------- |
| POST   | `/api/auth/registro`                         | público | Procesa el **formulario de registro**, 202 pendiente                   |
| GET    | `/api/auth/verificar-correo?token=`          | público | 200 cuenta activada                                                    |
| GET    | `/api/auth/sso/config`                       | público | 200 `{ "google_disponible": bool, "microsoft_disponible": bool }` — credenciales **y salud** del proveedor (FR-071; el frontend oculta los botones si falta alguna) |
| GET    | `/api/auth/sso/{proveedor}/iniciar`          | público | 307 redirección al proveedor (google/microsoft) |
| GET    | `/api/auth/sso/{proveedor}/callback`         | público | 303 redirige al panel con sesión creada (cookie) |
| GET    | `/api/auth/identidades`                      | sesión  | 200 lista de identidades vinculadas                                    |
| POST   | `/api/auth/identidades/{proveedor}/vincular` | sesión  | 200 identidad vinculada                                                |
| DELETE | `/api/auth/identidades/{proveedor}`          | sesión  | 204 identidad desvinculada                                             |

**Errores nuevos / específicos**:

- `correo_ya_registrado`: 202 (mensaje genérico anti-enumeración).
- `sso_no_configurado`: 503 / 422 si se intenta invocar `GET /api/auth/sso/{proveedor}/iniciar` o `GET /api/auth/sso/{proveedor}/callback` con un proveedor sin credenciales configuradas.
- `identidad_ya_vinculada`: 409.
- `ultima_identidad`: 409.

---

## Entidades Clave

- **Usuario**: Identidad de persona en el sistema; atributos clave: nombre, correo (único), credenciales (no expuestas, opcionales si hay SSO), rol (Administrador/Gerente u Operario/Cajero), estado (Activo/Inactivo), correo verificado.
- **Rol**: Definición de permisos; dos familias operativas: gestión (admin/gerente) y operación (operario/cajero).
- **Sesión**: Estado de autenticación de un usuario en un momento dado; vincula usuario + vigencia + invalidación.
- **Identidad externa**: Vinculación entre una cuenta local y un proveedor SSO (Google/Microsoft); atributos: proveedor, subject (id del proveedor), correo verificado, fecha de vinculación.
- **Token de verificación de correo**: Token de un solo uso con expiración para confirmar el correo en el auto-registro.

---

## Criterios de Éxito

- **SC-003**: Intentos de login con credenciales inválidas o inactivas devuelven el mensaje genérico sin revelar datos.
- **SC-004**: Cero sesiones activas tras la invalidación o baja de usuario.
- **SC-026**: El formulario de registro valida correctamente campos, requiere verificación por correo y previene la enumeración de usuarios.
- **SC-027**: Los botones de Google/Microsoft SSO se ocultan de manera automática en la interfaz si faltan las credenciales o claves API correspondientes o la sonda del proveedor falla; y con credenciales + *Authorized redirect URI* registrada en la consola del proveedor + proveedor respondiendo a la sonda, el clic en "Continuar con Google" termina con la sesión creada en el panel (sin `redirect_uri_mismatch` ni pantalla en blanco), verificación manual en quickstart V12.
- **SC-028**: La creación manual o automática de usuarios por parte de un administrador expone de forma inmediata y clara en la pantalla el correo junto con la contraseña temporal generada.

---

## Supuestos

- **Configuración de entorno**: Las variables de entorno para Google y Microsoft OAuth (`GOOGLE_CLIENT_ID`, `GOOGLE_CLIENT_SECRET`, etc.) determinan si el backend expone el servicio y si el frontend renderiza los botones correspondientes.
- **Configuración externa (Google Cloud Console)**: el Client ID de `GOOGLE_CLIENT_ID` DEBE tener como *Authorized redirect URI* exactamente `http://localhost:3000/api/auth/sso/google/callback` (es decir `${SSO_REDIRECT_BASE}/api/auth/sso/google/callback`). Sin esa entrada exacta, Google responde `Error 400: redirect_uri_mismatch` y el flujo no arranca aunque el backend esté correcto (SC-027). Con la app en modo "Testing", la cuenta de prueba DEBE figurar como *test user*. **Esto es una configuración externa al repositorio y es prerrequisito de verificación de FR-071/SC-027 (tasks T054/T055).**
- **Autenticación**: Se usa autenticación por correo y contraseña con sesión del servidor, más registro por correo y SSO con Google y Microsoft (OAuth 2.0 / OIDC).
- **Sesión**: Expiración **absoluta** por defecto de 8 horas de reloj desde el inicio de sesión (`SESSION_EXPIRY_HOURS`), sin renovación por actividad (sin sliding); una sola sesión vigente por usuario.
- **Idioma de UI**: Español (es-AR por defecto).
