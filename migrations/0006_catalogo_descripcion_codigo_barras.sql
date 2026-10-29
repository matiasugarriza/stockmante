-- 003 — catálogo: descripción de categoría, unicidad normalizada y código de barras opcional.
--
-- Contexto (data-model.md «Delta de esquema respecto de 0002»):
--   1. `categorias.descripcion` no existe en 0002 y es un atributo del requerimiento.
--   2. El nombre de categoría debe ser único sin distinguir mayúsculas ni espacios de extremo.
--   3. `productos.codigo_barras` nació como `UNIQUE NULLS NOT DISTINCT`: con esa forma solo un
--      producto podía quedar sin código. Debe ser `NULLS DISTINCT` (el default).
--   4. `0002` nunca creó el índice de `productos.categoria_id`.
--
-- La `0004` del WIP deshecho no se reutiliza: la numeración de SQLx es lineal y ese número ya
-- pudo aplicarse en entornos locales.

ALTER TABLE categorias ADD COLUMN descripcion VARCHAR(255) NULL;

CREATE UNIQUE INDEX uq_categorias_nombre_normalizado
    ON categorias (lower(btrim(nombre)));

-- La constraint de `codigo_barras` se localiza por columna y no por nombre: en una base creada
-- desde cero viene de 0002 y en las bases locales se corrigió a mano, sin fila en
-- `_sqlx_migrations`. Se reemplaza por la forma final en ambos casos.
DO $$
DECLARE
    constraint_actual text;
BEGIN
    SELECT c.conname
      INTO constraint_actual
      FROM pg_constraint c
      JOIN pg_attribute a
        ON a.attrelid = c.conrelid
       AND a.attnum = ANY (c.conkey)
     WHERE c.conrelid = 'productos'::regclass
       AND c.contype = 'u'
       AND a.attname = 'codigo_barras'
       AND array_length(c.conkey, 1) = 1;

    IF constraint_actual IS NOT NULL THEN
        EXECUTE format('ALTER TABLE productos DROP CONSTRAINT %I', constraint_actual);
    END IF;
END $$;

ALTER TABLE productos
    ADD CONSTRAINT productos_codigo_barras_key UNIQUE (codigo_barras);

CREATE INDEX idx_productos_categoria ON productos (categoria_id);
