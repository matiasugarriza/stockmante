# Especificación de Funcionalidad: Asistente de inventario

**Spec**: `017-asistente-inventario` | **Caso de uso**: US17 | **Prioridad**: P3

**Creado**: 2026-10-26 | **Estado**: Especificado (sin `plan.md` ni `tasks.md`; se generan al
implementar) | **Módulo**: 4 — Gobierno y asistencia

**Origen**: "IA/chatbot de inventario" figura fuera del alcance del Módulo 1 y la constitución ya
fija el stack (LangChain + LLM con Tools) y las reglas de seguridad de escritura.

**Depende de**: 007 (movimientos), 010 (consulta de stock), 014 (reportes), 013 (ventas).

## Escenarios de Usuario y Pruebas *(obligatorio)*

### User Story 17 - Preguntarle al sistema en lenguaje natural (Prioridad: P3)

Como **Gerente u Operario**, quiero preguntar "¿cuánto stock queda de yerba?" o "registrá una merma
de 2 unidades de yerba en Principal" y que el asistente consulte o ejecute la acción con mis
permisos, para operar más rápido sin navegar menús.

**Por qué esta prioridad**: es una capa de conveniencia sobre una API que ya funciona; no bloquea la
operación y su valor depende de que 007–014 estén cerradas.

**Escenario 17.1 — Consulta de stock**

- **Dado** un gerente autenticado,
- **Cuando** pregunta "¿cuánto stock hay de yerba?",
- **Entonces** el asistente consulta la API con la sesión del usuario y responde con el stock por
  depósito y el mínimo, sin inventar datos.

**Escenario 17.2 — Escritura con permisos**

- **Dado** un operario autenticado,
- **Cuando** pide "registrá una merma de 2 unidades de yerba en Principal por vencimiento",
- **Entonces** el asistente llama a `POST /api/movimientos` con la sesión del operario; si el
  permiso o el stock no alcanzan, responde el error real de la API
  (`"Stock insuficiente en Principal: 3 disponibles, 5 solicitadas."`).

**Escenario 17.3 — Escritura reservada a gestión**

- **Dado** un operario,
- **Cuando** pide un ajuste,
- **Entonces** el asistente responde que la acción requiere un gerente y **no** la ejecuta.

**Escenario 17.4 — Sin configuración de LLM**

- **Dado** un entorno sin `LLM_API_KEY` o sin el servicio de asistente,
- **Cuando** se usa el endpoint,
- **Entonces** responde **503** `asistente_no_disponible` con
  `"El asistente no está configurado en este entorno."` y el resto del sistema sigue operativo.

**Escenario 17.5 — Token nunca en el chat**

- **Dado** cualquier usuario,
- **Cuando** conversa,
- **Entonces** el asistente nunca le pide pegar un token ni credenciales; la sesión viaja del host
  autenticado al servicio, no del texto del usuario.

**Escenario 17.6 — Operario sin catálogo**

- **Dado** una sesión vencida o revocada,
- **Cuando** el asistente intenta una tool,
- **Entonces** recibe 401 de la API, lo informa y no reintenta con otra credencial.

## Requisitos Funcionales

- **FR-066 Tools de lectura.** El asistente DEBE poder consultar stock, productos, depósitos,
  ventas y reportes agregados mediante tools que llaman a la API Rust con la identidad y los
  permisos del usuario.
- **FR-067 Escrituras solo por API.** Toda escritura DEBE ejecutarse llamando a `POST
  /api/movimientos` (y a los endpoints de ventas cuando corresponda) con la sesión del usuario; el
  asistente NUNCA escribe contra la base ni usa credenciales de mayor privilegio.
- **FR-068 Seguridad del token y límites.** La sesión DEBE inyectarla el backend autenticado al
  servicio de asistente; el usuario NUNCA pega tokens en el chat. El endpoint DEBE aplicar un límite
  de consultas por usuario y un tiempo máximo de respuesta, y DEBE fallar con 503
  `asistente_no_disponible` cuando no esté configurado.
- **FR-069 Permisos y trazabilidad.** Cada tool DEBE respetar el RBAC vigente y cada escritura
  ejecutada DEBE quedar auditada como cualquier otra acción, con el usuario real como autor; las
  conversaciones NO se persisten.

## Criterios de Éxito Medibles

- **SC-025** Cero escrituras del asistente fuera de la API y cero acciones ejecutadas con permisos
  distintos a los del usuario de la sesión (verificado con una matriz de tools por familia de rol).

## Supuestos

- El asistente es un servicio separado (LangChain + proveedor LLM) detrás del backend Rust; el
  frontend solo habla con `/api/asistente/consulta`.
- El proveedor de LLM se configura por entorno (`LLM_PROVIDER`, `LLM_API_KEY`, `LLM_MODEL`); sin
  configuración el endpoint no se habilita.
- El asistente no persiste conversaciones ni historial; cada consulta es independiente.
- Las tools devuelven JSON estable de la API, sin adornos, y el modelo solo redacta la respuesta.
- No hay voz, ni adjuntos, ni ejecución de SQL libre por parte del modelo.

## Fuera de Alcance

- Historial de conversaciones, memoria por usuario o entrenamiento con datos del negocio.
- Generación de reportes complejos o predicciones de demanda.
- Escrituras masivas, anulaciones de ventas o cambios de precios desde el chat.
- Canales externos (WhatsApp, Telegram, Slack).

## Dependencias y orden de implementación

1. 007, 010, 013 y 014 implementadas.
2. Sin migración propia; la trazabilidad usa la auditoría existente y la de 015.
3. Requiere una decisión de despliegue del servicio de asistente y su configuración por entorno.
