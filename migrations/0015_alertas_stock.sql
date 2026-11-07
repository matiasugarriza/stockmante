-- 011: alertas de stock mínimo (FR-041 … FR-043).
--
-- Una fila por producto, destinatario y día local (UTC-03:00). Es la fuente de
-- verdad de la deduplicación del digest por correo: `enviada` es terminal en el
-- día; `fallida` se reintenta en el ciclo siguiente.

CREATE TABLE notificaciones_stock (
    id INT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    producto_id INT NOT NULL REFERENCES productos(id) ON DELETE CASCADE,
    usuario_id INT NOT NULL REFERENCES usuarios(id) ON DELETE CASCADE,
    fecha DATE NOT NULL,
    estado VARCHAR(10) NOT NULL DEFAULT 'pendiente'
        CHECK (estado IN ('pendiente', 'enviada', 'fallida')),
    intentos INT NOT NULL DEFAULT 0 CHECK (intentos >= 0),
    enviada_en TIMESTAMPTZ NULL,
    error TEXT NULL,
    CONSTRAINT uq_notificacion_producto_usuario_fecha
        UNIQUE (producto_id, usuario_id, fecha),
    CONSTRAINT ck_notificacion_enviada_con_fecha
        CHECK (estado <> 'enviada' OR enviada_en IS NOT NULL)
);

CREATE INDEX idx_notificaciones_pendientes
    ON notificaciones_stock (fecha, estado)
    WHERE estado <> 'enviada';
