# Especificación de Funcionalidad: Alertas de stock mínimo

**Spec**: `011-alertas-stock-minimo` | **Caso de uso**: US11 | **Prioridad**: P2

**Creado**: 2026-10-26 | **Estado**: Especificado (sin `plan.md` ni `tasks.md`; se generan al
implementar) | **Módulo**: 2 — Operación de stock

**Origen**: "notificaciones de stock mínimo" figura como fuera de alcance del Módulo 1 en el
README y en `specs/README.md`. La columna `productos.stock_minimo` ya existe desde 0002.

**Depende de**: 007 (stock), 010 (consulta de stock), 002 (correos de usuarios GESTION).

## Escenarios de Usuario y Pruebas *(obligatorio)*

### User Story 11 - Enterarme cuando un producto está por debajo del mínimo (Prioridad: P2)

Como **Gerente**, quiero recibir un aviso cuando un producto baja del stock mínimo, para reponer a
tiempo sin revisar el listado todos los días.

**Por qué esta prioridad**: la consulta manual funciona, pero una alerta convierte el control en
algo proactivo; es el complemento natural de 010 y del mínimo que ya carga el catálogo.

**Escenario 11.1 — Alerta visible**

- **Dado** un producto con `stock_minimo = 10` y `stock_actual = 4`,
- **Cuando** el gerente consulta `GET /api/alertas`,
- **Entonces** el producto aparece con mínimo, stock actual y faltante (6).

**Escenario 11.2 — Producto repuesto**

- **Dado** el mismo producto con stock 12,
- **Cuando** se consulta la lista,
- **Entonces** ya no aparece como alerta activa.

**Escenario 11.3 — Correo diario sin duplicados**

- **Dado** tres productos bajo mínimo,
- **Cuando** corre el ciclo de alertas,
- **Entonces** se envía **un** correo resumen a cada usuario `GESTION` activo con los tres
  productos, y un segundo ciclo el mismo día no repite el envío.

**Escenario 11.4 — Mínimo cero**

- **Dado** un producto con `stock_minimo = 0`,
- **Cuando** su stock llega a cero,
- **Entonces** no genera alerta: el mínimo cero significa "sin control".

**Escenario 11.5 — Envío manual**

- **Dado** un gerente,
- **Cuando** ejecuta `POST /api/alertas/enviar`,
- **Entonces** se dispara el ciclo en el momento y responde con cuántos productos y destinatarios
  procesó.

**Escenario 11.6 — Correo caído**

- **Dado** un fallo del transporte de correo,
- **Cuando** corre el ciclo,
- **Entonces** la ejecución no rompe la API, la alerta queda pendiente y el siguiente ciclo la
  reintenta sin duplicar las ya enviadas.

**Escenario 11.7 — Operario**

- **Dado** una sesión `OPERACION`,
- **Cuando** consulta o dispara alertas,
- **Entonces** responde **403** `sin_permisos`.

## Requisitos Funcionales

- **FR-041 Detección.** El sistema DEBE considerar en alerta a todo producto activo con
  `stock_minimo > 0` y `stock_actual < stock_minimo`, e informar mínimo, stock actual y faltante.
- **FR-042 Digest con deduplicación.** El sistema DEBE enviar un correo resumen por usuario
  `GESTION` activo como máximo una vez por producto y por día, registrando el envío en
  `notificaciones_stock (producto_id, fecha, enviada_en)` con restricción única
  `(producto_id, fecha)`.
- **FR-043 Ejecución y errores.** El ciclo DEBE correr cada `ALERTAS_INTERVALO_MINUTOS` (default
  60, `0` lo desactiva) y poder dispararse a demanda con `POST /api/alertas/enviar` (solo
  `GESTION`). Un fallo de correo DEBE registrar el error, no duplicar envíos y reintentar en el
  ciclo siguiente.

## Criterios de Éxito Medibles

- **SC-019** Cero correos duplicados por producto y día, y toda alerta activa es visible en
  `GET /api/alertas` dentro de un ciclo (≤ 60 minutos con la configuración por defecto).

## Supuestos

- El aviso va a los correos de los usuarios `GESTION` activos; no hay suscripciones por usuario ni
  por producto.
- El umbral es `productos.stock_minimo`, ya administrado por el catálogo (003).
- El envío es por correo (SMTP existente) y por la lista in-app; no hay push ni websockets.
- Si no hay usuarios `GESTION` activos, el ciclo no envía nada y no falla.
- La alerta se evalúa sobre el stock global (`productos.stock_actual`), no por depósito.

## Fuera de Alcance

- Notificaciones al operario o a un tercero.
- Suscripciones, silenciamiento por producto o umbrales por depósito.
- Avisos por WhatsApp, SMS o push.
- Sugerencia automática de cantidad a comprar (eso es 012 + 014).

## Dependencias y orden de implementación

1. 007 y 010 implementadas.
2. Migración reservada: `0012_alertas_stock.sql` (`notificaciones_stock`).
3. Se apoya en el transporte de correo existente (`EnvioCorreo`).
