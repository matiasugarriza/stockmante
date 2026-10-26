# Quickstart — Depósitos y ubicaciones (005)

**Objetivo**: verificar los escenarios de la spec contra el backend real. Requiere PostgreSQL con
`stockmante` migrada y el backend en `:3000`.

## Preparación

```powershell
cd C:\Users\Matias\Documents\tpOpencode\stockmante-v0\stockmante
$env:DATABASE_URL = "postgres://stockmante:stockmante@localhost:5432/stockmante"
cargo sqlx migrate run --source migrations   # no hay migración nueva; debe quedar igual
cd backend
cargo run
```

Login con un usuario GESTION y guardar la cookie (`-c cookies.txt`).

## Pasos

1. `POST /api/depositos` con `{ "nombre": "Depósito Norte", "identificador": "DEP-NORTE" }` → 201.
2. Repetir el mismo nombre con otro identificador → 409 `deposito_nombre_duplicado`.
3. `GET /api/depositos?activo=true` → incluye el depósito creado.
4. `POST /api/ubicaciones` con `{ "deposito_id": <id>, "nombre": "Pasillo 1", "tipo": "PASILLO" }`
   → 201 con `padre_id: null`.
5. Crear una hija con `padre_id` del paso 4 → 201; repetir el mismo nombre en el mismo nivel → 409.
6. Intentar una hija con `padre_id` de otro depósito → 422 `ubicacion_huerfana`.
7. `PUT /api/ubicaciones/<raíz>` con `{ "padre_id": <hija> }` → 422 (ciclo).
8. `DELETE /api/depositos/<id>` → 409 `deposito_con_dependencias`.
9. `POST /api/depositos/<id>/desactivar` → 204; el depósito sigue visible con
   `GET /api/depositos?activo=all`.
10. `POST /api/depositos/<id>/reactivar` → 204.
11. `PUT /api/productos/<producto>/ubicaciones` con `{ "ubicacion_ids": [<u1>, <u2>] }` → 204;
    `GET` devuelve las dos.
12. `PUT` con `[]` → 204; `GET` devuelve `{ "items": [] }`.
13. Desactivar `<u2>` y `PUT` con `[<u2>]` → 404.
14. REPETIR 1–13 con una sesión OPERACION: todas las rutas → 403 `sin_permisos`.
15. Sin cookie → 401.
16. Frontend: `/admin/depositos` como GESTION; crear depósito, agregar Pasillo → Estantería →
    Nivel, editar, desactivar; asignar dos ubicaciones a un producto y guardar.

## Verificación de integridad (SC-005)

```sql
SELECT count(*) AS huerfanas
FROM ubicaciones u
LEFT JOIN depositos d ON d.id = u.deposito_id
WHERE d.id IS NULL;   -- debe ser 0
```

## Evidencia a registrar

- Status y cuerpo de cada paso.
- Captura/nota del árbol en `/admin/depositos`.
- Resultado de la consulta de huérfanas.
