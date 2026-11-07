# Contrato: consulta de stock (010, extiende 007 FR-029)

Ambos endpoints requieren sesión vigente (401 `no_autenticado` sin cookie). Cambios respecto de
007 marcados con **nuevo**.

## GET /api/stock

Query:

| Parámetro | Tipo | Regla |
|---|---|---|
| `q` | string | subcadena de nombre o SKU, sin distinguir mayúsculas |
| `deposito_id` | int | agrega `stock_deposito`; no descarta productos sin existencias |
| `categoria_id` | int | — |
| `bajo_minimo` | bool | solo `stock_minimo > 0 AND stock_actual < stock_minimo` |
| `activo` | `true` \| `false` \| `all` | ausente = `true` |
| `producto_ids` | string | **nuevo**: ids separados por comas, 1–50; 422 si no son enteros o son más de 50 |
| `pagina`, `por_pagina` | int | paginación común (422 si fuera de rango) |

200:

```json
{
  "items": [{
    "producto_id": 7, "nombre": "Yerba 1 kg", "sku": "YERBA-1K",
    "categoria_id": 2, "categoria_nombre": "Almacén", "unidad_medida": "UNIDAD",
    "activo": true, "stock_actual": 3, "stock_minimo": 10, "bajo_minimo": true,
    "faltante": 7,
    "stock_deposito": 2,
    "costo": 1500.0,
    "valorizado": 3000.0
  }],
  "total": 1, "pagina": 1, "por_pagina": 20
}
```

- **nuevo** `faltante`: siempre presente, `max(stock_minimo − stock_actual, 0)`.
- **nuevo** `costo` y `valorizado`: números para sesiones `GESTION`; `null` para `OPERACION`.
  `valorizado = costo × stock_deposito` si vino `deposito_id`, si no `costo × stock_actual`.

Errores: 401; 422 `validacion` con mensaje específico (p. ej. `"producto_ids admite hasta 50 ids."`).

## GET /api/stock/{producto_id}

200: igual que en 007 más **nuevo** `faltante`. 404 `recurso_no_encontrado`
`"Producto no encontrado."`.
