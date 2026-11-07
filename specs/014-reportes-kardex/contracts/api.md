# Contratos de API: Reportes, kardex y valorización (014)

Base: `/api/reportes`. Todas las rutas exigen cookie de sesión (`stockmante_sesion`) y familia
`GESTION` (FR-059, SC-031). Solo lectura; sin paginación.

| Operación | `OPERACION` | `GESTION` |
|---|---|---|
| Cualquier reporte, JSON o CSV | ❌ 403 `sin_permisos` | ✅ |

Formato de error (`ErrorRespuesta`): `{ "error": { "codigo": "<codigo>", "mensaje": "<texto>" } }`.
Códigos de la feature: `validacion` (422), `recurso_no_encontrado` (404), `sin_permisos` (403),
`no_autenticado`/`sesion_expirada` (401). Mensajes exactos en [data-model.md](../data-model.md#errores).

**Fechas**: `desde` y `hasta` en `YYYY-MM-DD` (también RFC 3339), día local UTC-03:00, ambos
inclusivos. **CSV**: `formato=csv` → `200`, `Content-Type: text/csv; charset=utf-8`,
`Content-Disposition: attachment; filename="<reporte>_<desde>_<hasta>.csv"`, BOM UTF-8, RFC 4180,
líneas `\r\n`, decimales con punto.

## GET `/kardex`

| Parámetro | Obligatorio | Nota |
|---|---|---|
| `producto_id` | sí | entero |
| `deposito_id` | no | filtra saldo inicial y líneas |
| `desde`, `hasta` | no | sin `desde` = desde el inicio; sin `hasta` = sin límite |
| `formato` | no | `json` (default) o `csv` |

**200 JSON**

```json
{
  "producto_id": 1, "producto_nombre": "Yerba 1 kg", "deposito_id": null,
  "desde": "2026-10-01", "hasta": "2026-10-31",
  "saldo_inicial": 20,
  "lineas": [
    { "movimiento_id": 41, "creado_en": "2026-10-03T13:10:00Z", "tipo": "ENTRADA",
      "motivo": "COMPRA", "deposito_id": 1, "deposito_nombre": "Principal",
      "cantidad": 10, "saldo": 30 },
    { "movimiento_id": 57, "creado_en": "2026-10-31T23:30:00Z", "tipo": "SALIDA",
      "motivo": "VENTA", "deposito_id": 1, "deposito_nombre": "Principal",
      "cantidad": -4, "saldo": 26 }
  ],
  "saldo_final": 26
}
```

(La segunda línea es el 31/10 a las 20:30 locales: entra en `hasta=2026-10-31`.)

**CSV**: `fecha,tipo,motivo,deposito,cantidad,saldo`.

Errores: 404 producto inexistente; 422 rango/fecha/formato; 403.

## GET `/stock-valorizado`

| Parámetro | Obligatorio | Nota |
|---|---|---|
| `formato` | no | `json` o `csv` |

**200 JSON**

```json
{
  "items": [
    { "producto_id": 1, "nombre": "Yerba 1 kg", "sku": "YER-1", "cantidad": 26,
      "costo_unitario": 1500.00, "valor": 39000.00 }
  ],
  "total": 39000.00
}
```

**CSV**: `producto,sku,cantidad,costo_unitario,valor` (sin fila de total; el total es la suma).

## GET `/rotacion`

| Parámetro | Obligatorio | Nota |
|---|---|---|
| `desde`, `hasta` | **sí** | 422 si falta alguno |
| `formato` | no | `json` o `csv` |

**200 JSON**

```json
{
  "desde": "2026-10-01", "hasta": "2026-10-31",
  "items": [
    { "producto_id": 1, "nombre": "Yerba 1 kg", "unidades_salidas": 4,
      "stock_inicial": 20, "stock_final": 26, "stock_promedio": 23.0,
      "indice_rotacion": 0.1739 }
  ]
}
```

`unidades_salidas` cuenta movimientos `SALIDA` excepto motivo `TRANSFERENCIA`.

**CSV**: `producto,unidades_salidas,stock_inicial,stock_final,stock_promedio,indice_rotacion`.

## GET `/ventas`

| Parámetro | Obligatorio | Nota |
|---|---|---|
| `desde`, `hasta` | **sí** | 422 si falta alguno |
| `formato` | no | `json` o `csv` |
| `agrupar` | no | solo CSV: `dia` (default) o `producto` |

**200 JSON** (solo ventas `CONFIRMADA`)

```json
{
  "desde": "2026-10-01", "hasta": "2026-10-31",
  "cantidad_ventas": 2, "total": 9000.00,
  "por_dia": [ { "dia": "2026-10-31", "cantidad_ventas": 2, "total": 9000.00 } ],
  "por_producto": [ { "producto_id": 1, "nombre": "Yerba 1 kg", "unidades": 4, "total": 9200.00 } ]
}
```

`por_producto.total` suma subtotales de línea (antes del descuento); `total` suma `ventas.total`.

**CSV**: `agrupar=dia` → `dia,cantidad_ventas,total`; `agrupar=producto` → `producto,unidades,total`.
