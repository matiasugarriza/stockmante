# Stockmante

Sistema de gestión y control de inventario del Módulo 1: autenticación con sesión única,
administración de usuarios, catálogo de productos y categorías, búsqueda rápida para caja,
depósitos con ubicaciones jerárquicas, RBAC de dos familias y auditoría básica.

- **Backend**: Rust + Axum 0.8 + SQLx (PostgreSQL)
- **Frontend**: SvelteKit (SPA) + Tailwind CSS 4 + Zod
- **Base de datos**: PostgreSQL 15+ (probado con 18)
- **Documentación de API**: OpenAPI 3.1 + Swagger UI (`utoipa`)

## Estructura

```text
backend/        API Axum, servicios, modelos, tests unitarios e integración
frontend/       SPA SvelteKit (rutas /login, /caja, /admin/*)
migrations/     Migraciones SQLx (numeración global, no se reutiliza)
specs/          Spec-Driven Development: una spec por caso de uso
docs/           API, arquitectura y servidores MCP
opencode.json   MCP de proyecto (PostgreSQL readonly)
dbhub.toml      Configuración de DBHub (DSN desde DATABASE_URL)
```

## Requisitos

- Rust 1.88 o superior (probado con 1.99).
- Node.js 20 o superior.
- PostgreSQL 15 o superior con las bases `stockmante` y `stockmante_test`.
- `sqlx-cli` (`cargo install sqlx-cli --no-default-features --features rustls,postgres`) para
  migraciones y `cargo sqlx prepare --check`.

## Puesta en marcha

1. Crear las bases (una sola vez):

   ```sql
   CREATE ROLE stockmante LOGIN PASSWORD 'stockmante';
   CREATE DATABASE stockmante OWNER stockmante;
   CREATE DATABASE stockmante_test OWNER stockmante;
   ```

2. Copiar `.env.example` a `.env` y ajustar `DATABASE_URL` si hace falta.

3. Aplicar migraciones (desde la raíz, para que SQLx lea `migrations/` y `.env`):

   ```bash
   sqlx migrate run
   ```

4. Crear el depósito "Principal" y el admin inicial:

   ```bash
   cd backend
   cargo run --bin bootstrap
   ```

   En desarrollo imprime la contraseña temporal; el primer login obliga a cambiarla.
   Para regenerarla en desarrollo: `cargo run --bin bootstrap -- --reset-admin-password`
   (prohibido con `APP_ENV=production`).

5. Levantar el backend:

   ```bash
   cd backend
   cargo run --bin stockmante-backend
   ```

6. Levantar el frontend:

   ```bash
   cd frontend
   npm install
   npm run dev
   ```

   El frontend usa `PUBLIC_API_URL` (default `http://localhost:3000`).

## Documentación

- **Swagger UI**: <http://localhost:3000/docs>
- **OpenAPI JSON**: <http://localhost:3000/api-docs/openapi.json>
- [`docs/api.md`](./docs/api.md): mapa de endpoints, auth y errores.
- [`docs/arquitectura.md`](./docs/arquitectura.md): capas, transacciones y testing.
- [`docs/mcp.md`](./docs/mcp.md): MCP de PostgreSQL readonly y opcionales.
- [`specs/README.md`](./specs/README.md): mapa de specs, requisitos y ledger de migraciones.

La documentación se apaga con `DOCS_ENABLED=false`; en `APP_ENV=production` viene apagada por
defecto.

## Calidad

```bash
# Backend
cd backend
cargo test
cargo sqlx prepare --check

# Frontend
cd frontend
npm test
npm run check
npm run build
```

Reglas del proyecto: sin `unwrap()`, `expect()` ni `panic!` en rutas de producción; todo movimiento
de stock debe ser transaccional (módulos posteriores); la lógica de acceso vive en
`frontend/src/lib/api/*.ts` y los componentes no llaman `fetch` directo.

## MCP

`opencode.json` declara un servidor MCP de PostgreSQL en modo **solo lectura** (DBHub). Requiere
`DATABASE_URL` en `.env` y `npx` disponible. Ver [`docs/mcp.md`](./docs/mcp.md).

## Alcance del Módulo 1

Incluye login/sesión, usuarios, catálogo, búsqueda de caja, depósitos/ubicaciones, RBAC y auditoría
básica. **Fuera de alcance**: movimientos de stock, transferencias, facturación, reportes
financieros, notificaciones de stock mínimo, importación masiva y chatbot de inventario.
