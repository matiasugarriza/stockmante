-- Módulo 2 — transferencias entre depósitos (spec 008)
--
-- Una transferencia es una cabecera que enlaza dos movimientos (SALIDA en origen,
-- ENTRADA en destino) mediante `movimientos.transferencia_id`. La operación es
-- inmediata: no hay estado "en tránsito".

CREATE TABLE transferencias (
    id INT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    producto_id INT NOT NULL REFERENCES productos(id) ON DELETE RESTRICT,
    deposito_origen_id INT NOT NULL REFERENCES depositos(id) ON DELETE RESTRICT,
    deposito_destino_id INT NOT NULL REFERENCES depositos(id) ON DELETE RESTRICT,
    cantidad INT NOT NULL CHECK (cantidad > 0),
    ubicacion_origen_id INT NULL REFERENCES ubicaciones(id) ON DELETE RESTRICT,
    ubicacion_destino_id INT NULL REFERENCES ubicaciones(id) ON DELETE RESTRICT,
    usuario_id INT NOT NULL REFERENCES usuarios(id) ON DELETE RESTRICT,
    observacion TEXT NULL CHECK (observacion IS NULL OR length(observacion) <= 500),
    creado_en TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT ck_transferencia_depositos_distintos
        CHECK (deposito_origen_id <> deposito_destino_id)
);

-- Las FK hacia movimientos se agregan después de la columna transferencia_id.
ALTER TABLE movimientos
    ADD COLUMN transferencia_id INT NULL REFERENCES transferencias(id);

CREATE INDEX idx_transferencias_producto ON transferencias (producto_id, creado_en DESC);
CREATE INDEX idx_transferencias_origen ON transferencias (deposito_origen_id, creado_en DESC);
CREATE INDEX idx_transferencias_destino ON transferencias (deposito_destino_id, creado_en DESC);
CREATE INDEX idx_transferencias_creado_en ON transferencias (creado_en DESC);
CREATE INDEX idx_movimientos_transferencia ON movimientos (transferencia_id) WHERE transferencia_id IS NOT NULL;

ALTER TABLE transferencias
    ADD COLUMN movimiento_salida_id INT NOT NULL UNIQUE REFERENCES movimientos(id),
    ADD COLUMN movimiento_entrada_id INT NOT NULL UNIQUE REFERENCES movimientos(id);
