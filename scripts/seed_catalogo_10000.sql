-- 004 T022 — Catálogo de referencia de SC-002: 10.000 productos en 20 categorías.
-- Solo desarrollo; nunca en stockmante_test (los tests asumen un catálogo chico).
-- Idempotente por el prefijo SEED4-: cada corrida borra y vuelve a crear los mismos datos.
-- No genera movimientos, stock por depósito ni auditoría.
-- Limpieza: ejecutar solo el bloque «Limpieza» (las dos sentencias DELETE).
BEGIN;

-- Limpieza (productos primero: la FK de categoria_id es ON DELETE RESTRICT).
DELETE FROM productos WHERE sku LIKE 'SEED4-%';
DELETE FROM categorias WHERE nombre LIKE 'SEED4-CAT-%';

INSERT INTO categorias (nombre)
SELECT 'SEED4-CAT-' || lpad(n::text, 2, '0')
FROM generate_series(1, 20) AS n;

-- Nombres variados para que la similitud por trigramas trabaje sobre texto realista:
-- rubro + marca + presentación + número correlativo.
WITH partes AS (
    SELECT
        ARRAY['Yerba', 'Café', 'Té', 'Azúcar', 'Harina', 'Arroz', 'Fideos', 'Aceite',
              'Leche', 'Galletitas', 'Mermelada', 'Detergente', 'Jabón', 'Shampoo',
              'Gaseosa', 'Agua mineral', 'Cerveza', 'Vino tinto', 'Atún', 'Polenta',
              'Lentejas', 'Sal fina', 'Vinagre'] AS rubros,
        ARRAY['La Serena', 'Don Aurelio', 'Campo Verde', 'El Ombú', 'Santa Clara',
              'Los Andes', 'Río Claro', 'Monte Alto', 'La Pradera', 'Sol de Mayo',
              'Tres Robles', 'Buen Día', 'Del Valle'] AS marcas,
        ARRAY['500 g', '1 kg', '250 g', '1 l', '1,5 l', '2 l', '750 ml', 'x 6', 'x 12',
              '3 kg', '200 g'] AS presentaciones
)
INSERT INTO productos (
    nombre, sku, codigo_barras, categoria_id, unidad_medida, costo, precio_venta, stock_minimo
)
SELECT
    rubros[1 + (g % array_length(rubros, 1))] || ' '
        || marcas[1 + (g % array_length(marcas, 1))] || ' '
        || presentaciones[1 + (g % array_length(presentaciones, 1))] || ' '
        || lpad(g::text, 5, '0'),
    'SEED4-' || lpad(g::text, 5, '0'),
    -- 13 dígitos únicos: 7790999 + correlativo de 6 dígitos.
    '7790999' || lpad(g::text, 6, '0'),
    (SELECT c.id FROM categorias c
      WHERE c.nombre = 'SEED4-CAT-' || lpad((1 + (g % 20))::text, 2, '0')),
    'UNIDAD',
    100.00 + (g % 50),
    150.00 + (g % 80),
    g % 10
FROM partes, generate_series(1, 10000) AS g;

COMMIT;
