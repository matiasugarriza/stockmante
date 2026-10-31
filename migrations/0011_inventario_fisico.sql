-- Módulo 2 — inventario físico por conteo (spec 009)
--
-- Un conteo es una cabecera con estado (ABIERTO/CERRADO/CANCELADO) y líneas que
-- capturan cantidad_sistema (snapshot al abrir) y cantidad_contada (nullable hasta
-- cargar). El cierre genera un único movimiento AJUSTE (motivo INVENTARIO) por las
-- líneas con diferencia distinta de cero.

CREATE TABLE conteos (
    id INT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    deposito_id INT NOT NULL REFERENCES depositos(id) ON DELETE RESTRICT,
    estado VARCHAR(10) NOT NULL DEFAULT 'ABIERTO'
        CHECK (estado IN ('ABIERTO', 'CERRADO', 'CANCELADO')),
    usuario_id INT NOT NULL REFERENCES usuarios(id) ON DELETE RESTRICT,
    creado_en TIMESTAMPTZ NOT NULL DEFAULT now(),
    cerrado_en TIMESTAMPTZ NULL,
    cancelado_en TIMESTAMPTZ NULL,
    movimiento_ajuste_id INT NULL REFERENCES movimientos(id) ON DELETE RESTRICT,
    observacion TEXT NULL CHECK (observacion IS NULL OR length(observacion) <= 500)
);

CREATE INDEX idx_conteos_deposito ON conteos (deposito_id, creado_en DESC);
CREATE INDEX idx_conteos_estado ON conteos (estado, creado_en DESC);
CREATE INDEX idx_conteos_creado_en ON conteos (creado_en DESC);

CREATE TABLE conteo_lineas (
    id INT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    conteo_id INT NOT NULL REFERENCES conteos(id) ON DELETE CASCADE,
    producto_id INT NOT NULL REFERENCES productos(id) ON DELETE RESTRICT,
    ubicacion_id INT NULL REFERENCES ubicaciones(id) ON DELETE RESTRICT,
    cantidad_sistema INT NOT NULL CHECK (cantidad_sistema >= 0),
    cantidad_contada INT NULL CHECK (cantidad_contada IS NULL OR cantidad_contada >= 0),
    diferencia INT NULL,
    CONSTRAINT uq_conteo_linea_producto_ubicacion
        UNIQUE NULLS NOT DISTINCT (conteo_id, producto_id, ubicacion_id)
);

CREATE INDEX idx_conteo_lineas_conteo ON conteo_lineas (conteo_id);
CREATE INDEX idx_conteo_lineas_producto ON conteo_lineas (producto_id);