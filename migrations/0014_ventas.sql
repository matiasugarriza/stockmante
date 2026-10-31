-- Módulo 3 — ventas de caja y clientes (spec 013)

CREATE TABLE clientes (
    id INT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    nombre VARCHAR(120) NOT NULL,
    documento VARCHAR(20) NULL,
    email VARCHAR(200) NULL,
    telefono VARCHAR(60) NULL,
    direccion VARCHAR(500) NULL,
    activo BOOLEAN NOT NULL DEFAULT TRUE,
    creado_en TIMESTAMPTZ NOT NULL DEFAULT now(),
    actualizado_en TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT uq_clientes_documento UNIQUE (documento)
);

CREATE INDEX idx_clientes_nombre ON clientes (nombre);
CREATE INDEX idx_clientes_activo ON clientes (activo);

CREATE SEQUENCE ventas_numero_seq START 1;

CREATE TABLE ventas (
    id INT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    numero VARCHAR(20) NOT NULL UNIQUE,
    deposito_id INT NOT NULL REFERENCES depositos(id) ON DELETE RESTRICT,
    cliente_id INT NULL REFERENCES clientes(id) ON DELETE SET NULL,
    medio_pago VARCHAR(20) NOT NULL CHECK (medio_pago IN ('EFECTIVO','TARJETA','TRANSFERENCIA')),
    subtotal NUMERIC(18,2) NOT NULL DEFAULT 0,
    descuento NUMERIC(18,2) NOT NULL DEFAULT 0,
    total NUMERIC(18,2) NOT NULL DEFAULT 0,
    estado VARCHAR(12) NOT NULL DEFAULT 'CONFIRMADA' CHECK (estado IN ('CONFIRMADA','ANULADA')),
    observacion TEXT NULL CHECK (observacion IS NULL OR length(observacion) <= 500),
    usuario_id INT NOT NULL REFERENCES usuarios(id) ON DELETE RESTRICT,
    creado_en TIMESTAMPTZ NOT NULL DEFAULT now(),
    actualizado_en TIMESTAMPTZ NOT NULL DEFAULT now(),
    anulado_en TIMESTAMPTZ NULL
);

CREATE TABLE venta_lineas (
    id INT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    venta_id INT NOT NULL REFERENCES ventas(id) ON DELETE CASCADE,
    producto_id INT NOT NULL REFERENCES productos(id) ON DELETE RESTRICT,
    cantidad INT NOT NULL CHECK (cantidad > 0),
    precio_unitario NUMERIC(18,2) NOT NULL,
    subtotal NUMERIC(18,2) NOT NULL
);

CREATE INDEX idx_ventas_deposito ON ventas (deposito_id, creado_en DESC);
CREATE INDEX idx_ventas_cliente ON ventas (cliente_id);
CREATE INDEX idx_ventas_estado ON ventas (estado);
CREATE INDEX idx_ventas_creado_en ON ventas (creado_en DESC);
CREATE INDEX idx_venta_lineas_venta ON venta_lineas (venta_id);

ALTER TABLE movimientos
    ADD COLUMN venta_id INT NULL REFERENCES ventas(id) ON DELETE SET NULL;
CREATE INDEX idx_movimientos_venta ON movimientos (venta_id) WHERE venta_id IS NOT NULL;
