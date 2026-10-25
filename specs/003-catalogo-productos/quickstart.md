# Quickstart — Catálogo de productos y categorías (003)

**Propósito**: validar el caso de uso US3 (administración del catálogo). El setup y las variables
de entorno están en [001/quickstart.md](../001-autenticacion-sesion/quickstart.md).

## Prerrequisitos

- Base y US1 funcionando (001).
- Migraciones aplicadas, incluida `0004_productos_codigo_barras_null_distinct.sql`.
- Sesión con un usuario de familia GESTION.

## V6a — Categorías

1. Abrir `/admin/categorias`.
2. Crear "Almacén" y "Bebidas".
3. Esperado: 201 y listado ordenado por nombre.
4. Repetir "Almacén": 409 `categoria_duplicada`.
5. Editar el nombre y el estado `activo`; el cambio se refleja en el listado.
6. Eliminar una categoría sin productos: 204. Eliminar una con productos: 409
   `categoria_con_productos`.

## V6b — Productos

1. Abrir `/admin/productos` y crear un producto:
   `{ nombre: "Yerba 1 kg", sku: "yerba-1k", categoria: "Almacén", unidad: "UNIDAD",
   costo: 1200.50, precio: 1800, stock_minimo: 6, codigo_barras: "" }`.
2. Esperado: 201 con `sku: "YERBA-1K"` (normalizado) y `codigo_barras: null`.
3. Repetir el alta con el mismo SKU (en minúsculas): 409 `sku_duplicado`.
4. Crear otro producto con un código de barras y repetirlo en un tercero: 409
   `codigo_barras_duplicado`.
5. SKU `bad!` o de 2 caracteres: 422 `validacion` con mensaje accionable.
6. Costo con 3 decimales o negativo: 422.
7. Intentar `DELETE` de un producto con asignación de ubicación: 409
   `producto_con_referencias` (sugiere desactivar).
8. `POST /productos/{id}/desactivar`: 204; el producto deja de aparecer entre los activos.

## RBAC

| Acción | Operación | Gestión |
|---|---|---|
| `GET /categorias`, `GET /productos/{id}` | ✅ | ✅ |
| `GET /productos`, escrituras | 403 `sin_permisos` | ✅ |

## Fuera de esta guía

- Búsqueda por SKU/código/nombre y tiempos de caja → [004](../004-busqueda-caja/spec.md).
- Asignación de ubicaciones → [005](../005-depositos-ubicaciones/spec.md).

## V8 — Regresión de calidad

```bash
cd backend && cargo test && cargo sqlx prepare --check
cd frontend && npm test && npm run check && npm run build
```

Todos en verde; sin `unwrap`/`panic`/`expect` en rutas de producción.
