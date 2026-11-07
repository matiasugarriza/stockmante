# Quickstart — Búsqueda de caja (004)

**Objetivo**: verificar el endpoint y la pantalla con datos reales. Requiere PostgreSQL con
`stockmante` y el backend en `:3000`.

## Preparación

```powershell
# Migración 0007
cd C:\Users\Matias\Documents\tpOpencode\stockmante-v0\stockmante
$env:DATABASE_URL = "postgres://stockmante:stockmante@localhost:5432/stockmante"
cargo sqlx migrate run --source migrations

# Backend
cd backend
cargo run
```

## Pasos

1. **Login GESTION** y guardar cookie (`-c cookies.txt`).
2. Crear categoría `Almacén` y producto `Yerba 1 kg` con SKU `YERBA-1K` y código
   `7791234567890`.
3. `GET /api/productos/buscar?q=YERBA-1K` → 200; el primero es el producto exacto.
4. `GET /api/productos/buscar?q=7791234567890&campo=codigo` → 200; el producto exacto.
5. `GET /api/productos/buscar?q=yerb` → 200; incluye `Yerba 1 kg`, ordenado por similitud.
6. `GET /api/productos/buscar?q=almac&campo=categoria` → 200; solo productos de `Almacén`.
7. `GET /api/productos/buscar?categoria_id=<id>` (sin `q`) → 200; lista la categoría.
8. Desactivar el producto y repetir 3: no aparece. Con
   `incluir_inactivos=true` (GESTION) sí aparece.
9. Repetir 8 con una sesión OPERACION + `incluir_inactivos=true` → 403 `sin_permisos`.
10. `GET /api/productos/buscar?q=a&campo=nombre` → 422 (mínimo 2).
11. `GET /api/productos/buscar?q=YERBA-1K&limite=0` → 422.
12. Sin cookie → 401.
13. Frontend: entrar a `/caja` con OPERACION; escribir `yerb` y ver resultados en menos de 1 s;
    el checkbox de inactivos no aparece. Con GESTION sí aparece.
14. `EXPLAIN ANALYZE` de la consulta por nombre: registrar tiempo y uso del índice GIN.

## Ampliación 2026-11-07

### V2 — Aviso de truncados

15. En `/caja`, buscar un término con 20 coincidencias o más (con el seed de V3, `SEED4`):
    aparece «Se muestran los primeros 20 resultados. Refiná la búsqueda para encontrar el
    producto.». Con un término de menos de 20 coincidencias, el aviso no aparece.

### V3 — SC-002 sobre el catálogo de referencia

Requiere una base **de desarrollo** (no `stockmante_test`) y el backend apuntando a ella.

16. Verificar el esquema real de `productos` y `categorias` contra
    [data-model.md](./data-model.md) (Principio V) antes de sembrar.
17. Ejecutar `scripts/seed_catalogo_10000.sql`. **Esperado**: 10.000 productos `SEED4-` y 20
    categorías `SEED4-CAT-`.
18. Con el backend corriendo y una sesión válida:

    ```powershell
    $env:STOCKMANTE_URL = "http://localhost:3000"
    $env:STOCKMANTE_EMAIL = "<email de un usuario de desarrollo>"
    $env:STOCKMANTE_PASSWORD = "<su contraseña de desarrollo>"
    node scripts/medir_sc002.mjs
    ```

    **Esperado**: p95 ≤ 1000 ms para `sku`, `codigo`, `nombre` y `categoria` (40 consultas cada
    uno). Si alguno supera la meta, guardar `EXPLAIN (ANALYZE, BUFFERS)` de esa consulta y la
    decisión en `progress.md`; SC-002 no se da por cumplido.
19. `EXPLAIN (ANALYZE, BUFFERS)` de la búsqueda por nombre con el seed: registrar si usa
    `idx_productos_nombre_trgm`.
20. Limpiar: volver a ejecutar solo el bloque `DELETE` del seed (o el script completo y luego el
    `DELETE`) y confirmar que no quedan filas `SEED4-`.

## Evidencia a registrar

- Status y cuerpo de cada paso.
- p50/p95/máximo por tipo de búsqueda sobre 10.000 productos (objetivo SC-002: p95 ≤ 1 s,
  medido como respuesta de la API vista desde el cliente).
- Plan de ejecución de la consulta por nombre con el seed.
- Captura/nota de `/caja` con ambas familias y con el aviso de truncados.
