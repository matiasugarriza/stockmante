-- 003 T038 — Seed de 5.000 productos para medir SC-012.
-- Solo desarrollo. Idempotente por el prefijo SEED-; no genera movimientos ni auditoría.
BEGIN;

INSERT INTO categorias (nombre)
SELECT 'SEED-CAT'
WHERE NOT EXISTS (SELECT 1 FROM categorias WHERE nombre = 'SEED-CAT');

DELETE FROM productos WHERE sku LIKE 'SEED-%';

INSERT INTO productos (
    nombre, sku, codigo_barras, categoria_id, unidad_medida, costo, precio_venta, stock_minimo
)
SELECT
    'SEED Producto ' || lpad(g::text, 5, '0'),
    'SEED-' || lpad(g::text, 5, '0'),
    NULL,
    (SELECT id FROM categorias WHERE nombre = 'SEED-CAT'),
    'UNIDAD',
    100.00,
    150.00,
    5
FROM generate_series(1, 5000) AS g;

COMMIT;
