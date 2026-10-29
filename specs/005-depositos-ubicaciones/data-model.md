# Data Model — Depósitos y ubicaciones (005)

**Spec**: [spec.md](./spec.md) | **Plan**: [plan.md](./plan.md)

## Alcance

Sin delta de esquema: se documentan y consumen las tablas de `0003_depositos_ubicaciones.sql`.
La propiedad lógica del esquema queda en esta spec.

## `depositos`

| Columna | Tipo | Restricción | Uso |
|---|---|---|---|
| `id` | `INT IDENTITY` | PK | identificador |
| `nombre` | `VARCHAR(120)` | `NOT NULL UNIQUE` | nombre visible |
| `identificador` | `VARCHAR(40)` | `NOT NULL UNIQUE` | código corto (`DEP-CENTRAL`) |
| `direccion` | `TEXT` | `NULL` | dirección libre, opcional |
| `activo` | `BOOLEAN` | `NOT NULL DEFAULT TRUE` | baja lógica |

## `ubicaciones`

| Columna | Tipo | Restricción | Uso |
|---|---|---|---|
| `id` | `INT IDENTITY` | PK | identificador |
| `deposito_id` | `INT` | `NOT NULL` → `depositos(id)` `ON DELETE RESTRICT` | nunca huérfana |
| `padre_id` | `INT` | `NULL` → `ubicaciones(id)` `ON DELETE RESTRICT` | jerarquía |
| `nombre` | `VARCHAR(80)` | `NOT NULL` | nombre en el nivel |
| `tipo` | `VARCHAR(20)` | `NOT NULL` | enum de aplicación |
| `activo` | `BOOLEAN` | `NOT NULL DEFAULT TRUE` | baja lógica |

Restricciones e índices:

- `uq_ubicacion_unica UNIQUE NULLS NOT DISTINCT (deposito_id, padre_id, nombre)`:
  dos hermanas no pueden llamarse igual; las raíces (`padre_id NULL`) también compiten.
- `idx_ubicaciones_deposito`, `idx_ubicaciones_padre`: lecturas del árbol por depósito/padre.

## `producto_ubicacion_permitida`

| Columna | Tipo | Restricción |
|---|---|---|
| `producto_id` | `INT` | PK compuesta, FK a `productos` |
| `ubicacion_id` | `INT` | PK compuesta, FK a `ubicaciones` |

Representa las ubicaciones donde el producto puede recibir stock; los módulos de movimientos la
consumirán. La sustitución del conjunto es transaccional.

## Modelos de aplicación

```rust
pub struct Deposito { id, nombre, identificador, direccion: Option<String>, activo }
pub struct Ubicacion { id, deposito_id, padre_id: Option<i32>, nombre, tipo, activo }
pub struct ListaUbicaciones { items: Vec<Ubicacion> }
```

Enum de tipo (validado en `models/deposito.rs`): `PASILLO`, `ESTANTERIA`, `NIVEL`, `POSICION`.
La API lo devuelve en mayúsculas y acepta minúsculas en la entrada.

## Invariantes

1. Toda `ubicacion.deposito_id` referencia un depósito existente (NOT NULL + FK).
2. El `padre_id` de una ubicación pertenece al mismo `deposito_id` y está activo.
3. No hay ciclos en la jerarquía.
4. `(deposito_id, padre_id, nombre)` es único, incluidas las raíces.
5. Un depósito con ubicaciones o asignaciones no se borra físicamente.
6. Una ubicación con hijos o asignaciones no se borra físicamente ni cambia de depósito.
7. Las asignaciones solo referencian ubicaciones y depósitos activos.
8. `direccion` vacía se guarda como `NULL`.

## Verificación de SC-005

```sql
SELECT count(*) FROM ubicaciones u
LEFT JOIN depositos d ON d.id = u.deposito_id
WHERE d.id IS NULL;
```

Debe devolver `0`; además `deposito_id` es `NOT NULL`, por lo que la base lo impide estructuralmente.
