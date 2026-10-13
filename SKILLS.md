# SKILLS.md - Catálogo Completo de Habilidades y Patrones Técnicos

Este documento contiene los patrones de diseño, snippets de código obligatorios y guías paso a paso que el agente debe consultar y replicar para mantener consistencia en la arquitectura.

---

## 🚦 Reglas Globales de Disparo

Antes de escribir código, el agente debe identificar el tipo de tarea y consultar la habilidad correspondiente:

| Situación / Tarea | Habilidad a aplicar |
|---|---|
| Crear o modificar endpoints, lógica o servicios que registren movimientos de stock (entrada, salida, merma, ajuste, transferencia) | **Habilidad 1** |
| Escribir migraciones, consultas SQLx (`query!`, `query_as!`) o structs `FromRow` | **Habilidad 2** (siempre primero) |
| Implementar notificaciones, paneles de alertas o detectar productos bajo su umbral crítico | **Habilidad 3** |
| Consumir la API desde el frontend o crear componentes Svelte que muestren datos de inventario | **Habilidad 4** |
| Agregar capacidades al chatbot (consultar BD o ejecutar acciones de inventario mediante Tools) | **Habilidad 5** |
| Implementar escaneo, búsqueda por código o generación de etiquetas / códigos de barra | **Habilidad 6** |
| Implementar dashboards, reportes o cálculos de valorización financiera del inventario | **Habilidad 7** |

**Orden de ejecución:** toda tarea que toque la base de datos comienza con la **Habilidad 2** (verificación de esquema vía MCP) antes de aplicar cualquier otro patrón.

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
