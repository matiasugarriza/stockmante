-- Módulo 2 — stock y movimientos (spec 007)
--
-- `stock_por_deposito` es la fuente de verdad del stock (constitución, Principio V).
-- `productos.stock_actual` es un cache denormalizado: se recalcula dentro de la misma
-- transacción que escribe el movimiento, a partir de la suma por depósito.

ALTER TABLE productos
    ADD COLUMN stock_actual INT NOT NULL DEFAULT 0 CHECK (stock_actual >= 0);

CREATE TABLE stock_por_deposito (
    id INT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    producto_id INT NOT NULL REFERENCES productos(id) ON DELETE RESTRICT,
    deposito_id INT NOT NULL REFERENCES depositos(id) ON DELETE RESTRICT,
    -- NULL = stock imputado al depósito, sin ubicación concreta
    ubicacion_id INT NULL REFERENCES ubicaciones(id) ON DELETE RESTRICT,
    cantidad INT NOT NULL DEFAULT 0 CHECK (cantidad >= 0),
    actualizado_en TIMESTAMPTZ NOT NULL DEFAULT now(),
    -- NULLS NOT DISTINCT: dos filas "sin ubicación" del mismo producto y depósito colisionan
    CONSTRAINT uq_stock_producto_deposito_ubicacion
        UNIQUE NULLS NOT DISTINCT (producto_id, deposito_id, ubicacion_id)
);

CREATE INDEX idx_stock_producto ON stock_por_deposito (producto_id);
CREATE INDEX idx_stock_deposito ON stock_por_deposito (deposito_id);
CREATE INDEX idx_stock_ubicacion ON stock_por_deposito (ubicacion_id);

CREATE TABLE movimientos (
    id INT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    tipo VARCHAR(10) NOT NULL
        CHECK (tipo IN ('ENTRADA', 'SALIDA', 'MERMA', 'AJUSTE')),
    deposito_id INT NOT NULL REFERENCES depositos(id) ON DELETE RESTRICT,
    motivo VARCHAR(20) NOT NULL
        CHECK (motivo IN ('COMPRA', 'VENTA', 'MERMA', 'AJUSTE', 'DEVOLUCION',
                          'TRANSFERENCIA', 'INVENTARIO', 'IMPORTACION', 'OTRO')),
    observacion TEXT NULL,
    -- el autor sale siempre de la sesión, nunca del cuerpo
    usuario_id INT NOT NULL REFERENCES usuarios(id) ON DELETE RESTRICT,
    creado_en TIMESTAMPTZ NOT NULL DEFAULT now(),
    -- «OTRO» exige explicar el motivo
    CONSTRAINT ck_movimiento_otro_con_observacion
        CHECK (motivo <> 'OTRO' OR (observacion IS NOT NULL AND length(btrim(observacion)) > 0))
);

CREATE INDEX idx_movimientos_creado_en ON movimientos (creado_en DESC);
CREATE INDEX idx_movimientos_deposito ON movimientos (deposito_id, creado_en DESC);
CREATE INDEX idx_movimientos_tipo ON movimientos (tipo, creado_en DESC);

CREATE TABLE movimiento_lineas (
    id INT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    movimiento_id INT NOT NULL REFERENCES movimientos(id) ON DELETE CASCADE,
    producto_id INT NOT NULL REFERENCES productos(id) ON DELETE RESTRICT,
    ubicacion_id INT NULL REFERENCES ubicaciones(id) ON DELETE RESTRICT,
    -- con signo: ENTRADA > 0, SALIDA/MERMA < 0, AJUSTE ≠ 0
    cantidad_delta INT NOT NULL CHECK (cantidad_delta <> 0),
    -- saldo del producto en el depósito después de aplicar la línea
    stock_resultante INT NOT NULL CHECK (stock_resultante >= 0),
    CONSTRAINT uq_linea_producto_ubicacion
        UNIQUE NULLS NOT DISTINCT (movimiento_id, producto_id, ubicacion_id)
);

CREATE INDEX idx_lineas_movimiento ON movimiento_lineas (movimiento_id);
CREATE INDEX idx_lineas_producto ON movimiento_lineas (producto_id);
