# SKILLS.md - Catálogo Completo de Habilidades y Patrones Técnicos

Este documento contiene los patrones de diseño, snippets de código obligatorios y guías paso a paso que el agente debe consultar y replicar para mantener consistencia en la arquitectura.

Fuente de verdad del diseño del Módulo 1: `specs/README.md` (mapa por caso de uso, partición FR/SC y ledger de migraciones) y, para la base técnica, `specs/001-autenticacion-sesion/` (`spec.md`, `plan.md`, `data-model.md`, `contracts/`). Constitución: `.specify/memory/constitution.md` (v1.1.1).

---

## 🚦 Reglas Globales de Disparo

Antes de escribir código, el agente debe identificar el tipo de tarea y consultar la habilidad correspondiente:

| Situación / Tarea | Habilidad a aplicar |
|---|---|
| Migración SQL, `query!`/`query_as!`, structs `FromRow` (cualquier módulo) | **Habilidad 2** (siempre primero) |
| Login, sesión, logout, cambio de contraseña, RBAC, cookie de sesión | **Habilidad M1-A** (Módulo 1) |
| CRUD de usuarios, roles, baja lógica, contraseña temporal | **Habilidad M1-B** (Módulo 1) |
| CRUD de productos/categorías, SKU, código de barras, búsqueda de caja | **Habilidad M1-C** (Módulo 1) |
| Depósitos, ubicaciones jerárquicas, asignación producto–ubicación | **Habilidad M1-D** (Módulo 1) |
| `AppError`, mapeo HTTP 4xx/500, mensajes de error (Principios III y V) | **Habilidad M1-E** (Módulo 1) |
| Frontend SvelteKit: clientes `$lib/api`, Zod, stores de sesión, forms | **Habilidad M1-F** (Módulo 1) |
| Comandos de entorno, migraciones, tests, quality gates del Módulo 1 | **Habilidad M1-G** (Módulo 1) |
| Crear/modificar endpoints o servicios de movimientos de stock (entrada, salida, merma, ajuste, transferencia) | **Habilidad 1** (post-Módulo 1) |
| Notificaciones, alertas de stock mínimo | **Habilidad 3** (post-Módulo 1) |
| Consumo tipado de API de inventario en pantallas Svelte (genérico) | **Habilidad 4** |
| Tools del chatbot (consultar BD / acciones vía API) | **Habilidad 5** (post-Módulo 1) |
| Escaneo, EAN-13, utilidades de etiquetas | **Habilidad 6** |
| Dashboards, reportes, valorización financiera | **Habilidad 7** (post-Módulo 1) |

**Orden de ejecución:** toda tarea que toque la base de datos comienza con la **Habilidad 2** (verificación de esquema vía MCP o `sqlx-cli`) antes de aplicar cualquier otro patrón.

---

# Módulo 1 — Habilidades de ejecución (Usuarios, Auth, Catálogo, Depósitos)

Referencias: `specs/README.md` (partición y ledger) y los contratos de cada spec en `specs/*/contracts/`; la base compartida (auth, guards, sobre de error) vive en `specs/001-autenticacion-sesion/`.

## 📌 Habilidad M1-G: Comandos, scripts y quality gates (Módulo 1)

**Entorno local (raíz del repo):**

```bash
# Migraciones (siempre en /migrations de la raíz — constitución)
sqlx migrate run
sqlx migrate info

# Backend
cd backend
cargo sqlx prepare          # regenerar caché de queries tras cambiar SQL
cargo sqlx prepare --check  # CI / quality gate: sin diferencias
cargo test
cargo run                   # imprime en DEV el admin bootstrap + contraseña temporal

# Frontend (SvelteKit + Tailwind — no es "app Node"; toolchain de build/dev)
cd frontend
npm install
npm run dev                 # SPA en http://localhost:5173
npm test                    # vitest: $lib/api, schemas Zod, stores
```

**Checklist al cerrar cada tarea del Módulo 1:**

1. Verificar esquema real antes de SQL (Habilidad 2) — MCP `postgres_list_tables` / `postgres_describe_table` o fallback `sqlx table list` / `sqlx describe`.
2. `cargo test` en verde.
3. `cargo sqlx prepare --check` sin diferencias.
4. Sin `unwrap()` / `panic!` / `expect()` en rutas de producción.
5. Errores de dominio con variante `AppError` específica (Habilidad M1-E).
6. Spec/docs actualizados si cambió el comportamiento (`specs/<NNN-feature>/`, `specs/README.md`, `README.md` raíz).
7. Unidad de trabajo = un commit convencional (`feat: …`, `fix: …`, `test: …`, `docs: …`).

---

## 📌 Habilidad M1-E: AppError y mapeo HTTP (Principios III y V)

**Regla constitucional:** error de dominio → variante específica de `AppError` con 409/422 (o 401/403/404); error de infraestructura/persistencia → `AppError::Interno` con 500 genérico, sin exponer detalles de BD.

**Patrón (backend Rust):**

```rust
// backend/src/error.rs
#[derive(Debug)]
pub enum AppError {
    // Auth / dominio
    CredencialesInvalidas,          // 401 — SIEMPRE el mismo mensaje (anti-enumeración)
    SesionExpirada,                 // 401
    NoAutorizado,                   // 403
    NoEncontrado(&'static str),     // 404
    EmailDuplicado,                 // 409
    UltimoAdministrador,            // 409
    SkuDuplicado(String),           // 409
    CodigoBarrasDuplicado,          // 409
    ProductoConReferencias,         // 409
    DepositoConDependencias,        // 409
    Validacion(Vec<String>),        // 422
    // Infraestructura
    Database(sqlx::Error),          // 500 genérico al cliente
    Interno,                        // 500 genérico
}

// IntoResponse: dominio → status + { "error": { "codigo", "mensaje" } }
// Database/Interno → 500 { "codigo": "interno", "mensaje": "Error interno. Contacte al administrador." }
// NUNCA enviar el detalle de sqlx::Error al cliente.
```

**Mensajes en español de negocio, accionables** (Principio III), p. ej. `SkuDuplicado`: *"Ya existe un producto con SKU TEST-001."*

**Login:** todos los fallos (email inexistente, password mal, usuario inactivo) → `CredencialesInvalidas` → 401 con texto idéntico. Excepción deliberada documentada en la spec (Supuestos).

---

## 📌 Habilidad M1-A: Sesión, login y RBAC (Axum)

**Decisiones (research R1, R5):** cookie `HttpOnly` + `SameSite=Lax`; una sola sesión vigente por usuario (fila en `sesiones`; nuevo login revoca la anterior); expiración por `expira_en` (default 8h); Argon2id.

**Patrón de sesión (servicio):**

```rust
pub async fn login(pool: &PgPool, email: &str, password: &str) -> Result<(Uuid, Usuario), AppError> {
    let user = sqlx::query_as!(Usuario, r#"SELECT * FROM usuarios WHERE email = $1 AND activo = TRUE"#, email)
        .fetch_optional(pool).await
        .map_err(AppError::Database)?;

    // Verificar SIEMPRE argon2 aunque user sea None (evitar timing oracle simple)
    let hash_ok = match &user {
        Some(u) => argon2_verify(&u.password_hash, password).unwrap_or(false),
        None => { argon2_verify_dummy(password); false }
    };
    let Some(user) = user.filter(|_| hash_ok) else {
        return Err(AppError::CredencialesInvalidas); // mismo mensaje que inactivo / hash mal
    };

    // Revocar sesión anterior (una sola vigente)
    sqlx::query!(
        r#"UPDATE sesiones SET revocada_en = now() WHERE usuario_id = $1 AND revocada_en IS NULL"#,
        user.id
    ).execute(pool).await.map_err(AppError::Database)?;

    let session_id = Uuid::new_v4();
    let expira = Utc::now() + chrono::Duration::hours(8); // SESSION_EXPIRY_HOURS
    sqlx::query!(
        r#"INSERT INTO sesiones (id, usuario_id, expira_en) VALUES ($1, $2, $3)"#,
        session_id, user.id, expira
    ).execute(pool).await.map_err(AppError::Database)?;

    Ok((session_id, user))
}
```

**Extractor de autorización (axum):**

```rust
// AuthUsuario: lee cookie → sesión vigente + usuario activo; si no, 401.
// RequiereGestion: wrapper/extractor que exige familia_rol == "GESTION"; si no, 403.
// Handlers de /api/usuarios, /api/productos (escritura), /api/depositos usan RequiereGestion.
// /api/productos/buscar y /auth/* solo requieren AuthUsuario (o público en login).
```

**Cambio de contraseña temporal:** al login con `password_temporal = true`, el frontend fuerza `POST /auth/cambiar-password`; el servicio valida política y pone `password_temporal = FALSE`.

---

## 📌 Habilidad M1-B: Usuarios — CRUD y última regla de admin

**Alta de usuario (servicio):**

1. Validar email (formato) y unicidad → 409 `EmailDuplicado`.
2. Mapear rol: ADMIN/GERENTE → `familia_rol = GESTION`; OPERARIO/CAJERO → `OPERACION`.
3. Generar contraseña temporal con CSPRNG (≥12 chars), hashear Argon2id, `password_temporal = TRUE`.
4. Insertar usuario; enviar contraseña temporal por `TraitEnvioCorreo` (en dev: log/consola; prod: SMTP).
5. Nunca devolver el hash ni la contraseña temporal en respuestas de producción (solo en dev bootstrap).

**Desactivar (último admin):**

```rust
// Dentro de una transacción:
// SELECT count(*) FROM usuarios WHERE familia_rol = 'GESTION' AND activo = TRUE
//   [AND id <> $usuario_a_desactivar] FOR UPDATE;
// Si count == 0 → Err(AppError::UltimoAdministrador) // 409
// UPDATE usuarios SET activo = FALSE; UPDATE sesiones SET revocada_en = now() WHERE usuario_id = $1 AND revocada_en IS NULL;
```

**Baja lógica:** nunca `DELETE` de usuarios en este módulo (trazabilidad de movimientos futuros).

---

## 📌 Habilidad M1-C: Productos, SKU y búsqueda de caja

**Normalización de SKU (antes de validar unicidad):**

```rust
pub fn normalizar_sku(entrada: &str) -> Result<String, AppError> {
    let sku = entrada.trim().to_uppercase().replace(' ', "");
    // Regex: ^[A-Z0-9][A-Z0-9-]{2,29}$
    if !SKU_RE.is_match(&sku) {
        return Err(AppError::Validacion(vec!["SKU inválido: use 3–30 caracteres (letras, números y guiones).".into()]));
    }
    Ok(sku)
}
// Insert/update: unicidad global; violación unique → AppError::SkuDuplicado(sku) // 409
// codigo_barras: NULLS NOT DISTINCT en PG15+; unique → CodigoBarrasDuplicado
```

**Búsqueda caja (SC-002 ≤1s):**

- SKU o código de barras: match exacto (post-normalizar); traer **solo activos** por defecto.
- Nombre: `q` mínimo 2 caracteres; `pg_trgm` + índice GIN o `LOWER(nombre) LIKE 'q%'` según migración.
- Endpoint: `GET /api/productos/buscar?q=&campo=sku|codigo|nombre|categoria`.

**Borrado físico vs lógico:** DELETE solo si no hay referencias (movimientos, asignaciones, …); si hay → 409 `ProductoConReferencias` y ofrecer `POST /productos/{id}/desactivar`. Inactivos no salen en búsqueda de caja (`?incluir_inactivos=true` solo gestión).

---

## 📌 Habilidad M1-D: Depósitos y ubicaciones jerárquicas

**Sin huérfanas:** toda INSERT/UPDATE de `ubicaciones` exige `deposito_id` NOT NULL; si viene `padre_id`, el `deposito_id` debe ser el mismo que el del padre (validar en servicio → 422 `Validacion` / mensaje de ubicación inválida).

```sql
-- Garantías en migración
ALTER TABLE ubicaciones
  ADD CONSTRAINT ubicaciones_deposito_fk FOREIGN KEY (deposito_id) REFERENCES depositos(id) ON DELETE RESTRICT;
-- UNIQUE (deposito_id, padre_id, nombre) para gemelos
```

**DELETE depósito:** solo si no hay ubicaciones/movimientos → si no, 409 `DepositoConDependencias`.
**Asignación producto–ubicación:** `PUT /api/productos/{id}/ubicaciones` reemplaza el conjunto en `producto_ubicacion_permitida` (DELETE+INSERT en transacción o UPSERT diferencia).

---

## 📌 Habilidad M1-F: Frontend SvelteKit + Tailwind (Módulo 1)

**Stack frontend:** SvelteKit modo SPA (`adapter-static`) + Tailwind CSS + Zod. No se trata como "app Node": Node solo corre el toolchain de build/dev.

**Cliente API central (Principio II):**

```typescript
// frontend/src/lib/api/client.ts
const API_BASE = import.meta.env.PUBLIC_API_URL ?? 'http://localhost:8080/api';

export class ApiError extends Error {
  constructor(public status: number, public codigo: string, message: string) {
    super(message);
  }
}

export async function api<T>(path: string, init?: RequestInit): Promise<T> {
  const res = await fetch(`${API_BASE}${path}`, {
    credentials: 'include', // cookie de sesión
    headers: { 'Content-Type': 'application/json', ...init?.headers },
    ...init,
  });
  if (!res.ok) {
    const body = await res.json().catch(() => null);
    throw new ApiError(res.status, body?.error?.codigo ?? 'desconocido',
      body?.error?.mensaje ?? 'Error inesperado');
  }
  if (res.status === 204) return undefined as T;
  return res.json();
}
```

**Zod en `$lib/schemas/`** (ver `contracts/validation.md`): validar en forms ANTES del submit; el backend es la autoridad final.

**Store de sesión:**

```typescript
// $lib/stores/sesion.ts — cargar GET /auth/sesion en el arranque del layout;
// si 401 → redirigir a /login; si debe_cambiar_password → forzar pantalla de cambio.
```

**UI:** layouts `admin` (menú gestión) y `caja` (búsqueda SKU/código); tablas paginadas; modales de alta/edición; mensajes de error con el `mensaje` de `ApiError` (Principio III). Ocultar botones de gestión a familia operación (UX); **el backend igual deniega** con 403.

---

# Habilidades de módulos posteriores (referencia)

Las habilidades 1–7 de abajo se usan al implementar stock, alertas, chatbot, etc. No las invoques para tareas del Módulo 1 salvo que el patrón sea genérico (Habilidad 2 siempre; Habilidad 4/6 si aplica).

---

## 📌 Habilidad 1: Transacciones Atómicas para Movimientos de Stock (Rust / Axum)

**Descripción:** Patrón obligatorio para registrar cualquier entrada, salida o ajuste manual de inventario en PostgreSQL garantizando integridad de datos.

**Patrón de Implementación:**

```rust
use axum::{extract::Extension, http::StatusCode, Json};
use sqlx::PgPool;
use serde::{Deserialize, Serialize};

/// Tipos de movimiento permitidos. Se recomienda además crear un ENUM en
/// PostgreSQL (`CREATE TYPE tipo_movimiento AS ENUM (...)`) para que la
/// validez esté garantizada también en la base de datos.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[sqlx(type_name = "tipo_movimiento", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TipoMovimiento {
    Compra,
    Salida,
    Merma,
    Ajuste,
    Transferencia,
}

#[derive(Deserialize)]
pub struct MovimientoRequest {
    pub producto_id: i32,
    pub deposito_id: i32,
    pub cantidad: i32, // Positivo para entrada, negativo para salida/merma
    pub tipo: TipoMovimiento,
    pub justificacion: Option<String>,
    pub usuario_id: i32,
}

#[derive(Serialize)]
pub struct MovimientoResponse {
    pub success: bool,
    pub nuevo_stock: i32,
}

pub async fn registrar_movimiento(
    Extension(pool): Extension<PgPool>,
    Json(payload): Json<MovimientoRequest>,
) -> Result<(StatusCode, Json<MovimientoResponse>), AppError> {
    // 1. Iniciar transacción atómica
    let mut tx = pool.begin().await.map_err(AppError::Database)?;

    // 2. Actualizar el stock del depósito con guarda de seguridad: no puede
    //    quedar negativo. El upsert crea la fila si el producto ingresa por
    //    primera vez a este depósito; si la guarda falla, no hay fila
    //    afectada y se rechaza. (Si el INSERT inicializara una fila con
    //    cantidad negativa, el CHECK `cantidad >= 0` de la tabla lo rechaza.)
    let record = sqlx::query!(
        r#"
        INSERT INTO stock_por_deposito (producto_id, deposito_id, cantidad)
        VALUES ($1, $2, $3)
        ON CONFLICT (producto_id, deposito_id)
        DO UPDATE SET cantidad = stock_por_deposito.cantidad + EXCLUDED.cantidad
        WHERE stock_por_deposito.cantidad + EXCLUDED.cantidad >= 0
        RETURNING cantidad
        "#,
        payload.producto_id,
        payload.deposito_id,
        payload.cantidad
    )
    .fetch_optional(&mut *tx)
    .await
    .map_err(AppError::Database)?;

    let Some(record) = record else {
        // Sin fila afectada: stock insuficiente en ese depósito. `tx` se
        // descarta sin commit al salir del scope (sin cambios parciales).
        return Err(AppError::StockInsuficiente(payload.producto_id));
    };

    // 3. Mantener el cache denormalizado del total global en la MISMA transacción
    sqlx::query!(
        r#"
        UPDATE productos
        SET stock_actual = stock_actual + $1
        WHERE id = $2
        "#,
        payload.cantidad,
        payload.producto_id
    )
    .execute(&mut *tx)
    .await
    .map_err(AppError::Database)?;

    // 4. Registrar en la bitácora de auditoría de movimientos
    sqlx::query!(
        r#"
        INSERT INTO movimientos (producto_id, deposito_id, cantidad, tipo, justificacion, usuario_id)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
        payload.producto_id,
        payload.deposito_id,
        payload.cantidad,
        payload.tipo,
        payload.justificacion,
        payload.usuario_id
    )
    .execute(&mut *tx)
    .await
    .map_err(AppError::Database)?;

    // 5. Confirmar transacción
    tx.commit().await.map_err(AppError::Database)?;

    Ok((
        StatusCode::CREATED,
        Json(MovimientoResponse {
            success: true,
            nuevo_stock: record.cantidad, // stock resultante en el depósito movido
        }),
    ))
}
```

> **Notas:**
> - `AppError` debe incluir una variante específica para stock insuficiente (por ejemplo `AppError::StockInsuficiente(i32)` → HTTP 409 o 422).
> - **Modelo de datos (decidido): stock por depósito.** La fuente de verdad del stock es `stock_por_deposito(producto_id, deposito_id, cantidad)`. `productos.stock_actual` se mantiene únicamente como **cache denormalizado del total global** (suma de todos los depósitos) y DEBE actualizarse en la misma transacción que la tabla puente. Para cualquier lógica por depósito, usar siempre `stock_por_deposito`.
> - **TRANSFERENCIA** tiene semántica propia (mover stock entre dos depósitos sin cambiar el total): usar el patrón dedicado de abajo, **no** el handler general de arriba.

**Migración requerida (modelo de stock por depósito):**

```sql
CREATE TABLE stock_por_deposito (
    producto_id INT NOT NULL REFERENCES productos(id),
    deposito_id INT NOT NULL REFERENCES depositos(id),
    cantidad INT NOT NULL DEFAULT 0 CHECK (cantidad >= 0),
    PRIMARY KEY (producto_id, deposito_id)
);
```

**Patrón de TRANSFERENCIA entre depósitos (dos filas en una sola transacción):**

```rust
pub async fn transferir_stock(
    pool: &PgPool,
    producto_id: i32,
    deposito_origen_id: i32,
    deposito_destino_id: i32,
    cantidad: i32,
    usuario_id: i32,
) -> Result<(), AppError> {
    if deposito_origen_id == deposito_destino_id {
        return Err(AppError::Validacion("Origen y destino deben ser distintos".into()));
    }

    let mut tx = pool.begin().await.map_err(AppError::Database)?;

    // 1. Descontar del origen con guarda de no negativo
    let origen = sqlx::query!(
        r#"
        UPDATE stock_por_deposito
        SET cantidad = cantidad - $1
        WHERE producto_id = $2 AND deposito_id = $3 AND cantidad >= $1
        RETURNING cantidad
        "#,
        cantidad,
        producto_id,
        deposito_origen_id
    )
    .fetch_optional(&mut *tx)
    .await
    .map_err(AppError::Database)?;

    if origen.is_none() {
        return Err(AppError::StockInsuficiente(producto_id));
    }

    // 2. Aumentar en el destino (crear la fila si no existía)
    sqlx::query!(
        r#"
        INSERT INTO stock_por_deposito (producto_id, deposito_id, cantidad)
        VALUES ($1, $2, $3)
        ON CONFLICT (producto_id, deposito_id)
        DO UPDATE SET cantidad = stock_por_deposito.cantidad + EXCLUDED.cantidad
        "#,
        producto_id,
        deposito_destino_id,
        cantidad
    )
    .execute(&mut *tx)
    .await
    .map_err(AppError::Database)?;

    // 3. El total global NO cambia: no se toca productos.stock_actual.
    // 4. Bitácora: registrar SALIDA en el origen y ENTRADA en el destino
    //    (dos filas en `movimientos`). Si se desea correlacionarlas, añadir
    //    un campo `transferencia_id` a la Spec de la tabla movimientos.
    tx.commit().await.map_err(AppError::Database)?;

    Ok(())
}
```

---

## 📌 Habilidad 2: Verificación de Esquemas mediante MCP de PostgreSQL

**Descripción:** Protocolo para inspeccionar y validar la base de datos antes de generar migraciones o consultas con SQLx.

**Pasos para el Agente:**

1. Ejecutar la herramienta MCP `postgres_list_tables` para conocer las tablas activas.
2. Usar `postgres_describe_table` para verificar tipos exactos (ej: `i32` vs `i64`, `UUID`, `TIMESTAMPTZ`).
3. Verificar que las macros de SQLx (`query!` o `query_as!`) utilicen los nombres exactos de columnas registrados en PostgreSQL.
4. Si falta una tabla o columna para cumplir una Spec, proponer un script de migración SQL en `/migrations`.
5. **Fallback sin MCP:** si el servidor MCP de PostgreSQL no está disponible, verificar el esquema con `sqlx-cli` (misma `DATABASE_URL` del `.env`):
   ```bash
   sqlx table list          # tablas activas
   sqlx describe productos  # columnas y tipos exactos
   sqlx migrate info        # estado de migraciones
   ```

---

## 📌 Habilidad 3: Sistema de Alertas y Stock Mínimo

**Descripción:** Lógica automatizada para detectar cuando un producto cae por debajo de su umbral crítico y **persistir** la alerta en la base de datos (una sola alerta abierta por producto).

**Patrón de Implementación (Rust Query):**

```rust
pub async fn verificar_alertas_stock(pool: &PgPool) -> Result<Vec<AlertaItem>, AppError> {
    let mut tx = pool.begin().await.map_err(AppError::Database)?;

    let alertas = sqlx::query_as!(
        AlertaItem,
        r#"
        SELECT 
            p.id as producto_id, 
            p.nombre, 
            p.sku, 
            p.stock_actual, 
            p.stock_minimo
        FROM productos p
        WHERE p.stock_actual <= p.stock_minimo
        "#
    )
    .fetch_all(&mut *tx)
    .await
    .map_err(AppError::Database)?;

    // Persistir cada alerta solo si no existe una abierta para ese producto.
    for a in &alertas {
        sqlx::query!(
            r#"
            INSERT INTO alertas (producto_id, mensaje)
            VALUES ($1, $2)
            ON CONFLICT (producto_id, tipo_alerta) WHERE resuelta_en IS NULL
            DO NOTHING
            "#,
            a.producto_id,
            format!("Stock crítico: {} unidades (mínimo {})", a.stock_actual, a.stock_minimo)
        )
        .execute(&mut *tx)
        .await
        .map_err(AppError::Database)?;
    }

    tx.commit().await.map_err(AppError::Database)?;

    Ok(alertas)
}
```

**Migración requerida (una sola alerta abierta por producto):**

```sql
CREATE TABLE alertas (
    id SERIAL PRIMARY KEY,
    producto_id INT NOT NULL REFERENCES productos(id),
    tipo_alerta TEXT NOT NULL DEFAULT 'STOCK_BAJO',
    mensaje TEXT NOT NULL,
    creada_en TIMESTAMPTZ NOT NULL DEFAULT now(),
    resuelta_en TIMESTAMPTZ
);

CREATE UNIQUE INDEX ux_alertas_abierta
    ON alertas (producto_id, tipo_alerta)
    WHERE resuelta_en IS NULL;
```

---

## 📌 Habilidad 4: Consumo Tipado de API en SvelteKit con Fetch

**Descripción:** Manejo estándar de peticiones HTTP en la SPA SvelteKit con componentes reactivos y Tailwind CSS.

**Patrón de Implementación (SvelteKit TypeScript):**

```typescript
// src/lib/api/inventory.ts
// En SvelteKit las variables visibles en el cliente deben usar el prefijo
// PUBLIC_ y leerse desde $env; documentarlas en .env.example.
import { env } from '$env/dynamic/public';

const API_BASE = env.PUBLIC_API_URL || 'http://localhost:8080/api';

export interface Producto {
    id: number;
    nombre: string;
    sku: string;
    precio: number;
    stock_actual: number;
    stock_minimo: number;
}

export async function getProductos(): Promise<Producto[]> {
    const res = await fetch(`${API_BASE}/productos`, {
        method: 'GET',
        headers: { 'Content-Type': 'application/json' }
    });

    if (!res.ok) {
        throw new Error(`Error en la API: ${res.statusText}`);
    }

    return await res.json();
}
```

**Patrón de Componente Reusable Svelte 5:**

```svelte
<script lang="ts">
    import { onMount } from 'svelte';
    import { getProductos, type Producto } from '$lib/api/inventory';

    let productos = $state<Producto[]>([]);
    let cargando = $state(true);

    onMount(async () => {
        try {
            productos = await getProductos();
        } catch (e) {
            console.error(e);
        } finally {
            cargando = false;
        }
    });
</script>

{#if cargando}
    <div class="p-4 text-gray-500 animate-pulse">Cargando catálogo...</div>
{:else}
    <div class="grid grid-cols-1 md:grid-cols-3 gap-4 p-4">
        {#each productos as prod}
            <div class="border rounded-lg p-4 shadow-sm bg-white">
                <h3 class="font-bold text-lg">{prod.nombre}</h3>
                <p class="text-sm text-gray-500">SKU: {prod.sku}</p>
                <div class="mt-2 flex justify-between items-center">
                    <span class="text-green-600 font-semibold">${prod.precio}</span>
                    <span class="px-2 py-1 rounded text-xs {prod.stock_actual <= prod.stock_minimo ? 'bg-red-100 text-red-700' : 'bg-blue-100 text-blue-700'}">
                        Stock: {prod.stock_actual}
                    </span>
                </div>
            </div>
        {/each}
    </div>
{/if}
```

---

## 📌 Habilidad 5: Definición de Herramientas (Tools) para LangChain

**Descripción:** Cómo construir herramientas con docstrings explícitos para que el Chatbot Asistente consulte o ejecute acciones en la base de datos de inventario.

**Reglas de seguridad (obligatorias):**
- **Lecturas** (consultar stock, buscar productos): pueden ir directo a la BD con credenciales de solo lectura.
- **Escrituras** (entrada/salida/merma/ajuste): **nunca** escribir en la BD desde el chatbot. Deben llamar a la API de Rust (`POST /api/movimientos`), que valida auth/permisos y ejecuta la transacción atómica.
- El token de sesión lo inyecta el host autenticado; **nunca pedirle al usuario que pegue tokens en el chat**.

**Patrón de Implementación (Python / LangChain / LangGraph):**

```python
from langchain.tools import tool
import psycopg2
import os

@tool
def consultar_stock_producto(sku_o_nombre: str) -> str:
    """
    Busca la disponibilidad y ubicación exacta de un producto en el inventario
    utilizando su código SKU o su nombre comercial.
    Usa esta herramienta cuando el usuario pregunte por la cantidad de stock de un artículo.
    """
    database_url = os.getenv("DATABASE_URL")
    if not database_url:
        return "Error de configuración: la variable DATABASE_URL no está definida."

    conn = psycopg2.connect(database_url)
    cursor = conn.cursor()
    
    query = """
        SELECT p.nombre, p.sku, s.cantidad as stock, d.nombre as deposito
        FROM productos p
        JOIN stock_por_deposito s ON s.producto_id = p.id
        JOIN depositos d ON d.id = s.deposito_id
        WHERE (p.sku ILIKE %s OR p.nombre ILIKE %s)
          AND s.cantidad > 0
    """
    search_term = f"%{sku_o_nombre}%"
    try:
        cursor.execute(query, (search_term, search_term))
        results = cursor.fetchall()
    finally:
        conn.close()

    if not results:
        return f"No se encontraron productos que coincidan con '{sku_o_nombre}'."

    res_text = "Resultados encontrados:\n"
    for r in results:
        res_text += f"- {r[0]} (SKU: {r[1]}): {r[2]} unidades en Depósito '{r[3]}'.\n"
    
    return res_text
```

**Herramienta de escritura (siempre pasando por la API Rust):**

```python
import requests

API_BASE = os.getenv("API_BASE_URL", "http://localhost:8080/api")

@tool
def registrar_salida_stock(sku: str, deposito_id: int, cantidad: int, usuario_token: str, justificacion: str) -> str:
    """
    Registra una salida de inventario (venta/consumo) en un depósito específico.
    Requiere el token de un usuario autenticado con permisos de OPERARIO o ADMIN.
    Úsala solo cuando el usuario pida expresamente retirar mercadería; nunca la
    invoques por tu cuenta.
    """
    resp = requests.post(
        f"{API_BASE}/movimientos",
        json={
            "sku": sku,
            "deposito_id": deposito_id,
            "cantidad": -abs(cantidad),  # salidas siempre negativas
            "tipo": "SALIDA",
            "justificacion": justificacion,
        },
        headers={"Authorization": f"Bearer {usuario_token}"},
        timeout=10,
    )
    if resp.status_code in (401, 403):
        return "Permiso denegado: el usuario no está autenticado o no tiene permisos para registrar salidas."
    if not resp.ok:
        return f"Error al registrar la salida ({resp.status_code}): {resp.text}"

    nuevo_stock = resp.json().get("nuevo_stock")
    return f"Salida registrada. Stock resultante de {sku} en el depósito {deposito_id}: {nuevo_stock}."
```

> **Nota:** el stock vive en `stock_por_deposito` (modelo por depósito decidido); la consulta devuelve **una fila por depósito** con stock positivo. `productos.stock_actual` solo se usa para totales globales (cache denormalizado).

---

## 📌 Habilidad 6: Generación y Validación de Código de Barras / SKU

**Descripción:** Integración de utilidades frontend/backend para la lectura y generación dinámica de etiquetas y códigos de barra.

**Patrón de Implementación (TypeScript):**

```typescript
// Valida un EAN-13 completo incluyendo el dígito verificador (módulo 10).
export function isValidEan13(code: string): boolean {
    if (!/^\d{13}$/.test(code)) return false;
    const digits = code.split('').map(Number);
    const sum = digits
        .slice(0, 12)
        .reduce((acc, d, i) => acc + d * (i % 2 === 0 ? 1 : 3), 0);
    const checkDigit = (10 - (sum % 10)) % 10;
    return checkDigit === digits[12];
}

// Genera un EAN-13 a partir de 12 dígitos (el 13º se calcula).
export function generarEan13(codigoBase12: string): string {
    if (!/^\d{12}$/.test(codigoBase12)) {
        throw new Error('El código base debe tener exactamente 12 dígitos.');
    }
    const digits = codigoBase12.split('').map(Number);
    const sum = digits.reduce((acc, d, i) => acc + d * (i % 2 === 0 ? 1 : 3), 0);
    const checkDigit = (10 - (sum % 10)) % 10;
    return `${codigoBase12}${checkDigit}`;
}

// Parsea lecturas de escáner: EAN-13 solo si la secuencia es un EAN-13 válido.
export function parseBarCodeScan(scannedValue: string): { type: 'SKU' | 'EAN13', code: string } {
    const cleanCode = scannedValue.trim();
    if (isValidEan13(cleanCode)) {
        return { type: 'EAN13', code: cleanCode };
    }
    return { type: 'SKU', code: cleanCode.toUpperCase() };
}
```

---

## 📌 Habilidad 7: Reporte Financiero y Valorización de Inventario

**Descripción:** Consulta SQL optimizada para calcular capital invertido y margen proyectado sobre la totalidad del inventario.

**Patrón de Implementación (Consulta SQLx en Rust):**

```rust
#[derive(Serialize)]
pub struct ReporteFinanciero {
    pub capital_total_invertido: f64,
    pub valor_venta_proyectado: f64,
    pub margen_bruto_estimado: f64,
    pub total_articulos_diferentes: i64,
}

pub async fn obtener_reporte_valorizacion(pool: &PgPool) -> Result<ReporteFinanciero, AppError> {
    let reporte = sqlx::query!(
        r#"
        SELECT 
            COALESCE(SUM(costo * stock_actual), 0.0)::FLOAT8 as capital_invertido,
            COALESCE(SUM(precio * stock_actual), 0.0)::FLOAT8 as valor_venta,
            COUNT(id) as total_productos
        FROM productos
        "#
    )
    .fetch_one(pool)
    .await
    .map_err(AppError::Database)?;

    // COALESCE y COUNT nunca devuelven NULL, por lo que el record de `query!`
    // expone f64 / i64 directos (sin Option). Verificar la inferencia de
    // nullabilidad con `cargo sqlx prepare` contra el esquema real.
    Ok(ReporteFinanciero {
        capital_total_invertido: reporte.capital_invertido,
        valor_venta_proyectado: reporte.valor_venta,
        margen_bruto_estimado: reporte.valor_venta - reporte.capital_invertido,
        total_articulos_diferentes: reporte.total_productos,
    })
}
```
