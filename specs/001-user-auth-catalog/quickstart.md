# Quickstart — Validación Módulo 1

**Propósito**: guía de arranque y validación end-to-end tras la implementación. No incluye código de negocio.

## Prerrequisitos

- Rust (rustup, estable) — backend Axum
- **SvelteKit (modo SPA) + Tailwind CSS** — frontend (el toolchain de SvelteKit puede requerir un runtime JS solo para build/dev; la app en sí es SvelteKit + Tailwind, no Node.js)
- PostgreSQL 15+ corriendo (local o contenedor)
- `sqlx-cli` (`cargo install sqlx-cli --no-default-features --features postgres`)

## Variables de entorno

Copiar `.env.example` a `.env`:

```bash
DATABASE_URL=postgres://usuario:clave@localhost:5432/stockmante
SESSION_COOKIE_NAME=stockmante_session
SESSION_EXPIRY_HOURS=8
APP_URL=http://localhost:5173
# Correo: en dev se usa el logger; en prod SMTP
EMAIL_FROM=noreply@stockmante.local
SMTP_URL=            # vacío = no enviar red real
```

## Setup

```bash
# 1. Migraciones (desde la raíz del repo)
sqlx migrate run

# 2. Backend
cd backend
cargo sqlx prepare     # solo si cambió SQL; para CI: cargo sqlx prepare --check
cargo run              # imprime en consola de DEV la URL y contraseña temporal del admin bootstrap

# 3. Frontend SvelteKit + Tailwind (otra terminal)
cd frontend
# instalar dependencias del proyecto SvelteKit (gestor de paquetes del proyecto)
npm install
npm run dev            # http://localhost:5173 (SPA SvelteKit)
```

## Escenarios de validación manual (aceptación)

### V1 — Bootstrap y login (US1)

1. Abrir `/login`, autenticarse con el email del admin bootstrap y la contraseña temporal impresa en la consola del backend.
2. Esperado: cambio obligatorio de contraseña; al completar, panel de gestión visible.
3. Logout; login con la nueva contraseña → OK.

### V2 — Sesión única (US1 / FR-001)

1. Login en navegador A; copiar cookie.
2. Login en navegador B con el mismo usuario.
3. Esperado: A queda inválido en el siguiente request (401); B opera normalmente.

### V3 — Alta de usuario y rol (US2)

1. Como Admin: crear usuario Operario/Cajero.
2. Esperado: correo de contraseña temporal (en dev, log del backend); login del operario; **sin** menú de administración de usuarios/productos.
3. Intento directo de `POST /api/usuarios` con sesión de operario → 403.

### V4 — Login genérico (SC-003)

1. Login con password mala, email inexistente y usuario desactivado.
2. Esperado: mismo mensaje "Credenciales inválidas" en los tres casos.

### V5 — Último administrador (FR-006)

1. Con un solo Admin activo: intentar desactivarlo.
2. Esperado: 409 con mensaje claro; con un segundo Admin activo, la desactivación del primero funciona.

### V6 — Catálogo y búsqueda (US3–US4)

1. Crear categoría y producto con SKU `TEST-001`.
2. SKU duplicado → 409; SKU `bad sku` → 422.
3. En `/caja`, buscar `TEST-001` y por nombre parcial ≥2 letras → resultado ≤1s en LAN.
4. Desactivar producto con asignación de ubicación: DELETE físico → 409; desactivar → no aparece en caja.

### V7 — Depósitos y ubicaciones (US5)

1. Crear depósito; crear Estantería → Nivel bajo ese depósito.
2. POST ubicación sin depósito o con depósito del padre distinto → 422.
3. Asignar ubicaciones al producto; no se puede DELETE depósito con dependencias (409).

### V8 — Regresión de calidad

```bash
cd backend && cargo test && cargo sqlx prepare --check
cd frontend && npm test   # tests del frontend SvelteKit (vitest)
```

Todos en verde; sin `unwrap`/`panic`/`expect` en rutas de producción (`grep` o clippy).

## Fuera de alcance de esta guía

Movimientos de stock, transferencias, reportes, notificaciones de stock mínimo, importación CSV — módulos posteriores.
