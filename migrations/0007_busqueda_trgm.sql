-- 004 — búsqueda de caja: similitud por trigramas sobre el nombre del producto.
--
-- `pg_trgm` es una extensión trusted: el dueño de la base puede crearla sin superusuario.
-- El índice GIN acelera `%` (similitud) e `ILIKE '%texto%'`; la unicidad exacta de SKU y
-- código de barras ya usa los índices btree de 0002/0006.

CREATE EXTENSION IF NOT EXISTS pg_trgm;

CREATE INDEX idx_productos_nombre_trgm
    ON productos USING gin (nombre gin_trgm_ops);
