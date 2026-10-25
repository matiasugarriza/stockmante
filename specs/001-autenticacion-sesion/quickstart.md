# Quickstart — Base y autenticación (001)

**Propósito**: guía de arranque y validación end-to-end de la base y del caso de uso de
autenticación. No incluye código de negocio.

## Prerrequisitos

- Rust (rustup, estable) — backend Axum
- **SvelteKit (modo SPA) + Tailwind CSS** — frontend (el toolchain de SvelteKit puede requerir un runtime JS solo para build/dev; la app en sí es SvelteKit + Tailwind, no Node.js)
- PostgreSQL 15+ corriendo (local o contenedor)
- `sqlx-cli` (`cargo install sqlx-cli --no-default-features --features postgres`)

## Variables de entorno

Copiar `.env.example` a `.env` (valores reales en `.env.example`):

```bash
DATABASE_URL=postgres://usuario:clave@localhost:5432/stockmante
APP_URL=http://localhost:5173
PORT=3000
APP_ENV=development          # development imprime la temporal; production la silencia
SESSION_COOKIE_NAME=stockmante_sesion
SESSION_EXPIRY_HOURS=8
EMAIL_FROM=soporte@stockmante.local
SMTP_URL=                    # vacío = stub de consola (CorreoDev)
PUBLIC_API_URL=http://localhost:3000
```

## Setup

```bash
# 1. Migraciones (desde la raíz del repo)
sqlx migrate run

# 2. Bootstrap: admin inicial + depósito "Principal"
cd backend
cargo run --bin bootstrap

# 3. Backend
cargo sqlx prepare     # solo si cambió SQL; para CI: cargo sqlx prepare --check
cargo run              # imprime en consola de DEV la URL y contraseña temporal del admin bootstrap

# 4. Frontend SvelteKit + Tailwind (otra terminal)
cd frontend
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

### V4 — Login genérico (SC-003)

1. Login con password mala, email inexistente y usuario desactivado.
2. Esperado: mismo mensaje "Credenciales inválidas" en los tres casos.

### V8 — Regresión de calidad

```bash
cd backend && cargo test && cargo sqlx prepare --check
cd frontend && npm test && npm run check && npm run build
```

Todos en verde; sin `unwrap`/`panic`/`expect` en rutas de producción (`grep` o clippy).

## Escenarios de otras specs

| Escenario | Spec |
|---|---|
| V3 — Alta de usuario y rol | [002](../002-gestion-usuarios/quickstart.md) |
| V5 — Último administrador | [002](../002-gestion-usuarios/quickstart.md) |
| V6 — Catálogo y búsqueda | [003](../003-catalogo-productos/quickstart.md) y [004](../004-busqueda-caja/spec.md) |
| V7 — Depósitos y ubicaciones | [005](../005-depositos-ubicaciones/spec.md) |

## Fuera de alcance de esta guía

Movimientos de stock, transferencias, reportes, notificaciones de stock mínimo, importación CSV — módulos posteriores.
