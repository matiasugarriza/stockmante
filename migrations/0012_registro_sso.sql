-- Módulo 1 (enmienda) — registro y SSO (spec 001)
--
-- Agrega soporte para auto-registro por correo y SSO con Google/Microsoft.

ALTER TABLE usuarios
    ADD COLUMN email_verificado BOOLEAN NOT NULL DEFAULT FALSE;

-- Cuentas creadas por admin (002) se consideran verificadas.
UPDATE usuarios SET email_verificado = TRUE WHERE password_temporal = FALSE OR password_temporal IS NULL;

CREATE TABLE tokens_verificacion (
    id INT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    usuario_id INT NOT NULL REFERENCES usuarios(id) ON DELETE CASCADE,
    token_hash VARCHAR(128) NOT NULL UNIQUE,
    expira_en TIMESTAMPTZ NOT NULL,
    usado BOOLEAN NOT NULL DEFAULT FALSE,
    creado_en TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_tokens_verificacion_usuario ON tokens_verificacion (usuario_id);
CREATE INDEX idx_tokens_verificacion_expira ON tokens_verificacion (expira_en);

CREATE TABLE identidades_externas (
    id INT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    usuario_id INT NOT NULL REFERENCES usuarios(id) ON DELETE CASCADE,
    proveedor VARCHAR(20) NOT NULL CHECK (proveedor IN ('google', 'microsoft')),
    subject VARCHAR(255) NOT NULL,
    email VARCHAR(255) NOT NULL,
    creado_en TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT uq_identidad_proveedor_subject UNIQUE (proveedor, subject)
);

CREATE INDEX idx_identidades_usuario ON identidades_externas (usuario_id);
CREATE UNIQUE INDEX uq_identidad_usuario_proveedor ON identidades_externas (usuario_id, proveedor);