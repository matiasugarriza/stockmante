-- Módulo 1 — categorías y productos (data-model.md §categorias, §productos)

CREATE TABLE categorias (
    id INT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    nombre VARCHAR(80) NOT NULL UNIQUE,
    activo BOOLEAN NOT NULL DEFAULT TRUE
);

CREATE TABLE productos (
    id INT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    nombre VARCHAR(160) NOT NULL,
    sku VARCHAR(30) NOT NULL UNIQUE
        CHECK (sku ~ '^[A-Z0-9][A-Z0-9-]{2,29}$'),
    -- NULLS NOT DISTINCT (PG15+): dos NULL no colisionan, dos iguales sí
    codigo_barras VARCHAR(64) UNIQUE NULLS NOT DISTINCT,
    categoria_id INT NOT NULL REFERENCES categorias(id) ON DELETE RESTRICT,
    unidad_medida VARCHAR(20) NOT NULL,
    costo NUMERIC(12, 2) NOT NULL CHECK (costo >= 0),
    precio_venta NUMERIC(12, 2) NOT NULL CHECK (precio_venta >= 0),
    stock_minimo INT NOT NULL DEFAULT 0 CHECK (stock_minimo >= 0),
    activo BOOLEAN NOT NULL DEFAULT TRUE,
    creado_en TIMESTAMPTZ NOT NULL DEFAULT now(),
    actualizado_en TIMESTAMPTZ NOT NULL DEFAULT now()
);
