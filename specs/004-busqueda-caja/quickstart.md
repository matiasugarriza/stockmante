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

## Evidencia a registrar

- Status y cuerpo de cada paso.
- Tiempo de la búsqueda por nombre (objetivo SC-002: ≤ 1 s).
- Captura/nota de `/caja` con ambas familias.
