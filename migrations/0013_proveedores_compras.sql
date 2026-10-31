-- Módulo 3 — proveedores y compras (spec 012)

CREATE TABLE proveedores (
    id INT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    nombre VARCHAR(120) NOT NULL,
    cuit VARCHAR(11) NULL,
    email VARCHAR(200) NULL,
    telefono VARCHAR(60) NULL,
    direccion VARCHAR(500) NULL,
    activo BOOLEAN NOT NULL DEFAULT TRUE,
    creado_en TIMESTAMPTZ NOT NULL DEFAULT now(),
    actualizado_en TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT uq_proveedores_nombre UNIQUE (nombre),
    CONSTRAINT uq_proveedores_cuit UNIQUE (cuit)
);

CREATE INDEX idx_proveedores_nombre ON proveedores (nombre);
CREATE INDEX idx_proveedores_activo ON proveedores (activo);

CREATE TABLE compras (
    id INT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    proveedor_id INT NOT NULL REFERENCES proveedores(id) ON DELETE RESTRICT,
    deposito_id INT NOT NULL REFERENCES depositos(id) ON DELETE RESTRICT,
    numero VARCHAR(40) NULL,
    estado VARCHAR(12) NOT NULL DEFAULT 'BORRADOR' CHECK (estado IN ('BORRADOR','CONFIRMADA','ANULADA')),
    observacion TEXT NULL CHECK (observacion IS NULL OR length(observacion) <= 500),
    total_costo NUMERIC(18,2) NOT NULL DEFAULT 0,
    usuario_id INT NOT NULL REFERENCES usuarios(id) ON DELETE RESTRICT,
    creado_en TIMESTAMPTZ NOT NULL DEFAULT now(),
    actualizado_en TIMESTAMPTZ NOT NULL DEFAULT now(),
    confirmado_en TIMESTAMPTZ NULL,
    anulado_en TIMESTAMPTZ NULL
);

CREATE TABLE compra_lineas (
    id INT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    compra_id INT NOT NULL REFERENCES compras(id) ON DELETE CASCADE,
    producto_id INT NOT NULL REFERENCES productos(id) ON DELETE RESTRICT,
    cantidad INT NOT NULL CHECK (cantidad > 0),
    costo_unitario NUMERIC(18,2) NOT NULL CHECK (costo_unitario >= 0),
    subtotal NUMERIC(18,2) NOT NULL,
    orden INT NOT NULL DEFAULT 0
);

CREATE INDEX idx_compras_proveedor ON compras (proveedor_id, creado_en DESC);
CREATE INDEX idx_compras_estado ON compras (estado);
CREATE INDEX idx_compras_creado_en ON compras (creado_en DESC);
CREATE INDEX idx_compra_lineas_compra ON compra_lineas (compra_id);

-- Movimiento referencia a compra (para trazabilidad)
ALTER TABLE movimientos
    ADD COLUMN compra_id INT NULL REFERENCES compras(id) ON DELETE SET NULL;
CREATE INDEX idx_movimientos_compra ON movimientos (compra_id) WHERE compra_id IS NOT NULL;
