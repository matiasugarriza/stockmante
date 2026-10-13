-- Módulo 1 — depósitos, ubicaciones jerárquicas y asignaciones permitidas
-- (data-model.md §depositos, §ubicaciones, §producto_ubicacion_permitida)

CREATE TABLE depositos (
    id INT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    nombre VARCHAR(120) NOT NULL UNIQUE,
    identificador VARCHAR(40) NOT NULL UNIQUE,
    direccion TEXT NULL,
    activo BOOLEAN NOT NULL DEFAULT TRUE
);

CREATE TABLE ubicaciones (
    id INT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    -- nunca huérfana: deposito_id NOT NULL
    deposito_id INT NOT NULL REFERENCES depositos(id) ON DELETE RESTRICT,
    padre_id INT NULL REFERENCES ubicaciones(id) ON DELETE RESTRICT,
    nombre VARCHAR(80) NOT NULL,
    tipo VARCHAR(20) NOT NULL,
    activo BOOLEAN NOT NULL DEFAULT TRUE,
    -- evita gemelos confusos; NULLS NOT DISTINCT hace únicas las raíces (padre_id NULL)
    CONSTRAINT uq_ubicacion_unica
        UNIQUE NULLS NOT DISTINCT (deposito_id, padre_id, nombre)
);

CREATE INDEX idx_ubicaciones_deposito ON ubicaciones (deposito_id);
CREATE INDEX idx_ubicaciones_padre ON ubicaciones (padre_id);

CREATE TABLE producto_ubicacion_permitida (
    producto_id INT NOT NULL REFERENCES productos(id) ON DELETE CASCADE,
    ubicacion_id INT NOT NULL REFERENCES ubicaciones(id) ON DELETE RESTRICT,
    creada_en TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (producto_id, ubicacion_id)
);

CREATE INDEX idx_pup_ubicacion ON producto_ubicacion_permitida (ubicacion_id);
