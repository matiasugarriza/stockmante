-- 006: auditoría básica de acciones sensibles (FR-019).
--
-- Registra autor, acción, registro afectado y timestamp de altas, cambios y
-- bajas de usuarios y productos. No es un log rico: no guarda diffs ni valores.
-- `usuario_id` queda en NULL solo si la cuenta autora se borra físicamente
-- (situación que en producción no ocurre: los usuarios se dan de baja lógica).

CREATE TABLE IF NOT EXISTS auditoria (
    id          BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    tabla_ref   VARCHAR(32) NOT NULL,
    registro_id INT NOT NULL,
    accion      VARCHAR(24) NOT NULL,
    usuario_id  INT REFERENCES usuarios(id) ON DELETE SET NULL,
    cuando      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_auditoria_registro
    ON auditoria (tabla_ref, registro_id, cuando DESC);

CREATE INDEX IF NOT EXISTS idx_auditoria_usuario
    ON auditoria (usuario_id, cuando DESC);
