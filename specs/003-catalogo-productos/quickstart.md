# Quickstart — Catálogo de productos y categorías (003)

**Propósito**: validar el caso de uso US3 (administración del catálogo y baja lógica) de punta a
punta. El setup, las variables de entorno y el arranque están en
[001/quickstart.md](../001-autenticacion-sesion/quickstart.md); acá solo se listan los escenarios
observables de esta feature.

## Prerrequisitos

- Backend y frontend de 001 y 002 funcionando.
- Migraciones `0001`–`0003` y `0005` aplicadas, más `0006_catalogo_descripcion_codigo_barras.sql`.
  Verificar antes contra el esquema real (`\d productos`, `\d categorias`).
- Sesión con un usuario de familia **GESTION** (para escritura) y otro de **OPERACION** (para los
  casos de 403).

## V6a — Categorías

1. Abrir `/admin/categorias` y crear `{ "nombre": "Almacén", "descripcion": "Productos secos" }`.
   **Esperado**: 201; el listado la muestra ordenada por nombre.
2. Crear `{ "nombre": "Bebidas" }` sin descripción. **Esperado**: 201 con `descripcion: null`.
3. Repetir el alta con `{ "nombre": "  BEBIDAS " }`. **Esperado**: 409 `categoria_duplicada`.
4. `GET /api/categorias` sin parámetros. **Esperado**: 200 con las categorías activas, sin inactivas.
5. `GET /api/categorias?activo=all`. **Esperado**: incluye las dadas de baja.
6. `GET /api/categorias/{id}` de una categoría inactiva. **Esperado**: 200 (sigue consultable).
7. `PUT /api/categorias/{id}` con `{ "nombre": "Almacén seco" }`. **Esperado**: 200; `descripcion` y
   `activo` no cambian.
8. `PUT /api/categorias/{id}` con `{ "descripcion": "" }`. **Esperado**: 200 con `descripcion: null`.
9. `PUT /api/categorias/{id}` con `{ "activo": false }`. **Esperado**: 422 `validacion`.
10. `PUT /api/categorias/{id}` con `{}`. **Esperado**: 422 `validacion`.
11. `DELETE /api/categorias/{id}`. **Esperado**: **405**; `GET /api/categorias/{id}` sigue
    devolviendo la misma fila con el mismo `activo`.
12. `POST /api/categorias/{id}/desactivar` dos veces. **Esperado**: 204 las dos veces.
13. `POST /api/categorias/{id}/reactivar`. **Esperado**: 204; vuelve al listado por defecto.

## V6b — Productos

1. Abrir `/admin/productos` y crear (usar el `categoria_id` de una categoría **activa** de tu base;
   `2` es solo un ejemplo):

   ```json
   {
     "nombre": "Yerba 1 kg",
     "sku": "yerba-1k",
     "codigo_barras": "",
     "categoria_id": 2,
     "unidad_medida": "UNIDAD",
     "costo": 1200.50,
     "precio_venta": 1800,
     "stock_minimo": 6
   }
   ```

   **Esperado**: 201 con `sku: "YERBA-1K"` (normalizado) y `codigo_barras: null`.

2. Repetir el alta con el mismo SKU en minúsculas. **Esperado**: 409 `sku_duplicado`.
3. Crear un segundo producto **sin** `codigo_barras` y un tercero también sin él. **Esperado**: 201
   en ambos (varios `NULL` conviven).
4. Crear dos productos con el mismo `codigo_barras`. **Esperado**: el segundo devuelve 409
   `codigo_barras_duplicado`.
5. Alta con SKU `bad!`, `AB` o 31 caracteres. **Esperado**: 422 `validacion` con el formato esperado.
6. Alta con `costo: -1`, `precio_venta: 10.999` o `stock_minimo: -3`. **Esperado**: 422 por campo.
7. Alta con `unidad_medida: "LITRO"`. **Esperado**: 422 indicando las unidades válidas
   (`UNIDAD`, `KG`, `METRO`, `CAJA`).
8. Alta con `categoria_id` de una categoría inactiva o inexistente. **Esperado**: 422.
9. `GET /api/productos?q=yerba`. **Esperado**: devuelve el producto por coincidencia en nombre.
   `GET /api/productos?q=YERBA-1K` lo devuelve por SKU.
10. `GET /api/productos?categoria_id=<id activo>&activo=all`. **Esperado**: solo los de esa
    categoría, incluidos los inactivos.
11. `GET /api/productos?activo=otro`. **Esperado**: 422 `validacion`.
12. `DELETE /api/productos/{id}`. **Esperado**: **405**; `GET /api/productos/{id}` sigue devolviendo
    el producto con el mismo `id` y `activo`.
13. `POST /api/productos/{id}/desactivar`. **Esperado**: 204; el listado por defecto ya no lo muestra
    y `?activo=all` sí; el detalle sigue en 200.
14. `POST /api/productos/{id}/reactivar`. **Esperado**: 204; vuelve al listado por defecto con SKU,
    precios y categoría intactos.
15. `PUT /api/productos/{id}` con `{ "activo": false }`. **Esperado**: 422 y el producto sigue activo.

## RBAC

| Acción | OPERACION | GESTION |
|---|---|---|
| `GET /categorias`, `GET /categorias/{id}` | 200 | 200 |
| `GET /productos/{id}` | 200 | 200 |
| `GET /productos` | 403 `sin_permisos` | 200 |
| `POST` / `PUT` / `desactivar` / `reactivar` | 403 `sin_permisos` | 200/201/204 |
| Cualquier ruta sin sesión | 401 `no_autenticado` | — |

## Invariantes a verificar después de la batería

1. `SELECT count(*) FROM productos` y `FROM categorias` no cambió por ninguna baja lógica: solo
   cambió `activo`.
2. Ningún `DELETE` modificó una fila.
3. La numeración de `id` no tiene huecos **por borrados**: los saltos solo pueden venir de altas
   rechazadas que consumieron la secuencia (PostgreSQL no la retrocede), nunca de una baja lógica.
4. `GET /api/usuarios` sin filtro sigue devolviendo activos e inactivos (regresión de
   `parsear_activo`).

## V8 — Regresión de calidad

```bash
cd backend && cargo test && cargo sqlx prepare --check
cd frontend && npm test && npm run check && npm run build
```

Todos en verde; sin `unwrap`/`panic!`/`expect` en rutas de producción.

## Fuera de esta guía

- Búsqueda por SKU/código/nombre y tiempos de caja → [004](../004-busqueda-caja/spec.md).
- Asignación de ubicaciones → [005](../005-depositos-ubicaciones/spec.md).
- Rechazo de movimientos sobre productos inactivos → módulo de stock (futuro); contrato en
  [spec.md](./spec.md) FR-022.
