# Contratos de API: Alertas de stock mínimo (011)

Base: `/api`. Todas las rutas exigen cookie de sesión (`stockmante_sesion`) y, además, familia
`GESTION`; `OPERACION` recibe **403** `sin_permisos` (escenario 11.7). Con contraseña temporal
pendiente el gate global de `auth::gate` responde 403 `cambio_password_requerido`.

Formato de error (existente, `ErrorRespuesta`): `{ "error": "<codigo>" }` con texto en español de
negocio.

## GET `/api/alertas`

Listado paginado de productos bajo mínimo.

**Query params**

| Param | Tipo | Default | Regla |
|---|---|---|---|
| `pagina` | int ≥ 1 | 1 | fuera de rango → 422 `validacion` |
| `por_pagina` | int 1..100 | 50 | fuera de rango → 422 `validacion` |

**200**

```json
{
  "items": [
    {
      "producto_id": 12,
      "nombre": "Cemento Loma Negra 50 kg",
      "sku": "CEM-50",
      "stock_actual": 4,
      "stock_minimo": 10,
      "faltante": 6
    }
  ],
  "total": 1,
  "pagina": 1,
  "por_pagina": 50
}
```

**Reglas de selección**: `productos.activo = TRUE AND productos.stock_minimo > 0 AND
productos.stock_actual < productos.stock_minimo`. Orden: `nombre`, `id`. No incluye productos con
mínimo 0 (11.4) ni repuestos (11.2, quedan fuera del predicado).

**Errores**: 401 `no_autenticado` / `sesion_expirada`; 403 `sin_permisos`.

## POST `/api/alertas/enviar`

Dispara el ciclo de alertas en el momento (11.5). No recibe cuerpo.

**200**

```json
{
  "fecha": "2026-11-07",
  "productos": 3,
  "destinatarios": 2,
  "enviados": 6,
  "fallidos": 0
}
```

| Campo | Significado |
|---|---|
| `fecha` | día local (UTC−03:00) usado como clave de deduplicación |
| `productos` | productos distintos bajo mínimo detectados |
| `destinatarios` | usuarios `GESTION` activos evaluados |
| `enviados` | filas `notificaciones_stock` nuevas marcadas `enviada` |
| `fallidos` | filas marcadas `fallida` en esta corrida |

**Semántica**

- Idempotente dentro del día: una segunda llamada el mismo día devuelve
  `enviados: 0` (salvo filas `fallida` reintentables) y no duplica correos.
- Un fallo de transporte responde **200** con `fallidos > 0`; no rompe la API (11.6). La fila queda
  `fallida` y el ciclo siguiente la reintenta.
- Sin usuarios `GESTION` activos: `{ "fecha": ..., "productos": n, "destinatarios": 0,
  "enviados": 0, "fallidos": 0 }`, sin error.
- Sin productos bajo mínimo: `productos: 0`, `enviados: 0`, sin error.

**Errores**: 401; 403 `sin_permisos`; 500 `interno` solo ante fallo de persistencia.

## Correo enviado

- **Uno por usuario y ciclo** (11.3), con todos los productos que ese usuario tiene pendientes
  (`estado <> 'enviada'`). Asunto: `Stockmante — productos bajo el stock mínimo`.
- Cuerpo en texto plano: saludo con el nombre, y una línea por producto con nombre, SKU, stock
  actual, mínimo y faltante (Principio III: resultado concreto, no genérico).
- El cuerpo lo arma `services::alertas` (sin I/O); el transporte solo lo envía.

## Contrato de transporte (`EnvioCorreo`)

```rust
async fn enviar_alerta_stock(
    &self,
    destino: &str,
    nombre: &str,
    cuerpo: &str,
) -> Result<(), String>;
```

Implementan: `mail::envio::CorreoDev` (registra en `bandeja()`), `mail::smtp::CorreoSmtp` (lettre) y
el fake `CorreoQueFalla` de `tests/unit/test_usuarios_alta.rs`.

## Fuera de contrato

- `GET /api/stock?bajo_minimo=true` **no cambia**: sigue abierto a cualquier sesión y sin
  `faltante` (010 queda intacto).
- No hay websockets, push ni streaming de alertas.
