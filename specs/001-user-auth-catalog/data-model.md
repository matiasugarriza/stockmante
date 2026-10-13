# Data Model — Módulo 1

**Spec**: [spec.md](./spec.md) | **Plan**: [plan.md](./plan.md) | **Date**: 2026-10-12

> Tipos PostgreSQL; nombres en snake_case; fechas en `TIMESTAMPTZ`. Toda migración vive en `/migrations` (raíz). ANTES de cada migración/SQL: verificar esquema real (constitución — Verificación de esquema).

## Diagrama de relaciones (texto)

```
usuarios 1─* sesiones
categorias 1─* productos
depositos 1─* ubicaciones
ubicaciones 1─* ubicaciones (padre/hijo, misma tabla)
productos *─* ubicaciones  (producto_ubicacion_permitida)
```

## usuarios

| Campo | Tipo | Restricciones |
|-------|------|----------------|
| id | SERIAL / INT GENERATED ALWAYS AS IDENTITY | PK |
| nombre | VARCHAR(120) | NOT NULL, TRIM |
| email | VARCHAR(255) | NOT NULL, UNIQUE, formato validado en app (RFC básico) |
| password_hash | VARCHAR(255) | NOT NULL (Argon2id) |
| password_temporal | BOOLEAN | NOT NULL DEFAULT TRUE — obliga cambio en primer ingreso |
| rol | VARCHAR(20) | NOT NULL, CHECK IN ('ADMIN','GERENTE','OPERARIO','CAJERO') |
| familia_rol | VARCHAR(10) | Generated o mapeada en app: gestión (ADMIN/GERENTE) vs operación (OPERARIO/CAJERO). Preferencia: columna persistida `familia_rol CHECK IN ('GESTION','OPERACION')` actualizada en trigger o en servicio al crear usuario, para guardar RBAC simple. |
| activo | BOOLEAN | NOT NULL DEFAULT TRUE |
| creado_en | TIMESTAMPTZ | NOT NULL DEFAULT now() |
| actualizado_en | TIMESTAMPTZ | NOT NULL DEFAULT now() |

**Reglas de negocio**
- Unicidad de email (índice único).
- Baja lógica: `activo = FALSE`; no se borra la fila (trazabilidad de movimientos futuros).
- El último usuario con `familia_rol = 'GESTION' AND activo = TRUE` no puede ser desactivado (validación en servicio + constraint defensible en app, no solo CHECK SQL porque requiere conteo).
- Credenciales: hash Argon2id; temporal enviada por correo; `password_temporal` se pone en FALSE al cambiar.

**Transiciones de estado**
```
[activo=true] --desactivar--> [activo=false]  (sesiones revocadas)
[activo=false] --reactivar--> [activo=true]
cualquier login --cambio password ok--> password_temporal=false
```

## sesiones

| Campo | Tipo | Restricciones |
|-------|------|----------------|
| id | UUID | PK |
| usuario_id | INT | FK → usuarios(id) ON DELETE CASCADE, NOT NULL |
| creada_en | TIMESTAMPTZ | NOT NULL DEFAULT now() |
| expira_en | TIMESTAMPTZ | NOT NULL |
| ultima_actividad | TIMESTAMPTZ | NOT NULL DEFAULT now() (deslizante, p.ej. al actualizar en cada request autenticado) |
| revocada_en | TIMESTAMPTZ NULL | NULL = vigente |

**Reglas**
- Máximo UNA fila vigente por usuario: al crear sesión nueva, `UPDATE sesiones SET revocada_en = now() WHERE usuario_id = $1 AND revocada_en IS NULL`.
- Logout / desactivar usuario: marcar `revocada_en`.
- Expiración: `expira_en` = ahora + `SESSION_EXPIRY_HOURS` (default 8); sliding optional — decisión: **sin sliding** para cumplir 8h hábiles simple; refrescar `ultima_actividad` solo para auditoría. (Simpler: expiración absoluta desde login.)
- Cookie: valor = `sesiones.id` (UUID), flags HttpOnly, SameSite=Lax, Secure en prod.

## categorias

| Campo | Tipo | Restricciones |
|-------|------|----------------|
| id | SERIAL | PK |
| nombre | VARCHAR(80) | NOT NULL, UNIQUE |
| activo | BOOLEAN | NOT NULL DEFAULT TRUE |

Lista plana (spec Supuestos).

## productos

| Campo | Tipo | Restricciones |
|-------|------|----------------|
| id | SERIAL | PK |
| nombre | VARCHAR(160) | NOT NULL |
| sku | VARCHAR(30) | NOT NULL, UNIQUE, CHECK ~ '^[A-Z0-9][A-Z0-9-]{2,29}$' |
| codigo_barras | VARCHAR(64) | NULL, UNIQUE (NULLS NOT DISTINCT en PG15+), solo dígitos/alfanuméricos |
| categoria_id | INT | FK → categorias(id) ON DELETE RESTRICT, NOT NULL |
| unidad_medida | VARCHAR(20) | NOT NULL (ej. UNIDAD, KG, CAJA) — catálogo simple en app o CHECK |
| costo | NUMERIC(12,2) | NOT NULL, CHECK (costo >= 0) |
| precio_venta | NUMERIC(12,2) | NOT NULL, CHECK (precio_venta >= 0) |
| stock_minimo | INT | NOT NULL DEFAULT 0, CHECK (stock_minimo >= 0) |
| activo | BOOLEAN | NOT NULL DEFAULT TRUE |
| creado_en / actualizado_en | TIMESTAMPTZ | DEFAULT now() |

**Reglas**
- SKU: se normaliza a mayúsculas en servicio antes de insertar/buscar; unicidad global (activos e inactivos) para no reutilizar SKU de producto histórico.
- Código de barras: opcional; si se informa, único entre todos los productos; se puede limpiar (SET NULL) si no hay historial que lo referencie.
- Borrado físico solo sin referencias (movimientos, ventas, asignaciones). En este módulo: sin referencias → DELETE; con referencias → `activo = FALSE` (FR-012).
- Índices de búsqueda: `idx_productos_sku` (único), `idx_productos_codigo_barras` (único parcial WHERE NOT NULL si no se usa NULLS NOT DISTINCT), `idx_productos_nombre_trgm` o `idx_productos_nombre_lower` con `LOWER(nombre) LIKE` / `pg_trgm` — decisión: **pg_trgm + GIN** para búsqueda parcial rápida (SC-002).
- Productos inactivos: excluidos de `GET /api/productos/buscar` por defecto (`?incluir_inactivos=true` solo gestión).

## depositos

| Campo | Tipo | Restricciones |
|-------|------|----------------|
| id | SERIAL | PK |
| nombre | VARCHAR(120) | NOT NULL, UNIQUE |
| identificador | VARCHAR(40) | NOT NULL UNIQUE (código corto o dirección resumida) |
| direccion | TEXT | NULL (detalle largo opcional) |
| activo | BOOLEAN | NOT NULL DEFAULT TRUE |

Borrado físico solo sin ubicaciones/movimientos; si tiene dependencias → bloqueo (FR-018) o baja lógica.

## ubicaciones

| Campo | Tipo | Restricciones |
|-------|------|----------------|
| id | SERIAL | PK |
| deposito_id | INT | FK → depositos(id) ON DELETE RESTRICT, NOT NULL — **nunca huérfana** |
| padre_id | INT NULL | FK → ubicaciones(id) ON DELETE RESTRICT |
| nombre | VARCHAR(80) | NOT NULL (ej. 'Estantería A', 'Nivel 2') |
| tipo | VARCHAR(20) | NOT NULL (ej. PASILLO, ESTANTERIA, NIVEL, POSICION) — enum simple |
| activo | BOOLEAN | NOT NULL DEFAULT TRUE |
| UNIQUE | | (deposito_id, padre_id, nombre) para evitar gemelos confusos |

**Reglas**
- Toda ubicación tiene `deposito_id` explícito (se propaga desde el padre si se crea anidada; se valida igualdad si `padre_id` informado).
- Borrado: RESTRICT si tiene hijos o stock (stock en módulos posteriores).
- Mover a otro depósito: solo si no tiene hijos ni asignaciones (caso borde de spec).

## producto_ubicacion_permitida

| Campo | Tipo | Restricciones |
|-------|------|----------------|
| producto_id | INT | FK → productos(id) ON DELETE CASCADE, PK compuesto |
| ubicacion_id | INT | FK → ubicaciones(id) ON DELETE RESTRICT, PK compuesto |
| creada_en | TIMESTAMPTZ | DEFAULT now() |

Registra ubicaciones iniciales permitidas (FR-017). En módulos de stock, el movimiento valida pertenencia a este conjunto.

## Índices transversales (resumen)

- `usuarios(email)` UNIQUE
- `sesiones(usuario_id)` WHERE revocada_en IS NULL (vigentes)
- `productos(sku)` UNIQUE; `productos(codigo_barras)` UNIQUE; GIN trigram en nombre
- `ubicaciones(deposito_id)`; `ubicaciones(padre_id)`
- `producto_ubicacion_permitida(ubicacion_id)` para reverse lookup

## Validaciones de integridad (app + BD)

| Regla | Nivel |
|-------|-------|
| Email único y formato | BD UNIQUE + app validator |
| SKU normalizado + único + regex | BD CHECK + UNIQUE + app |
| Último admin no se desactiva | Servicio (SELECT COUNT FOR UPDATE) |
| Ubicación sin depósito | FK NOT NULL |
| Código de barras único | BD UNIQUE + app |
| Familia rol coherente con rol | CHECK o servicio |
