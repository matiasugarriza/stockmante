-- Módulo 1 — usuarios y sesiones (data-model.md §usuarios, §sesiones)
-- Verificación de esquema previa (T006): BD sin tablas de negocio.

CREATE TABLE usuarios (
    id INT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    nombre VARCHAR(120) NOT NULL,
    email VARCHAR(255) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    password_temporal BOOLEAN NOT NULL DEFAULT TRUE,
    rol VARCHAR(20) NOT NULL CHECK (rol IN ('ADMIN', 'GERENTE', 'OPERARIO', 'CAJERO')),
    familia_rol VARCHAR(10) NOT NULL CHECK (familia_rol IN ('GESTION', 'OPERACION')),
    activo BOOLEAN NOT NULL DEFAULT TRUE,
    creado_en TIMESTAMPTZ NOT NULL DEFAULT now(),
    actualizado_en TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT chk_familia_rol_coherente CHECK (
        (familia_rol = 'GESTION' AND rol IN ('ADMIN', 'GERENTE'))
        OR (familia_rol = 'OPERACION' AND rol IN ('OPERARIO', 'CAJERO'))
    )
);

CREATE TABLE sesiones (
    id UUID PRIMARY KEY,
    usuario_id INT NOT NULL REFERENCES usuarios(id) ON DELETE CASCADE,
    creada_en TIMESTAMPTZ NOT NULL DEFAULT now(),
    expira_en TIMESTAMPTZ NOT NULL,
    ultima_actividad TIMESTAMPTZ NOT NULL DEFAULT now(),
    revocada_en TIMESTAMPTZ NULL
);

-- Sesiones vigentes por usuario (sesión única: se filtra revocada_en IS NULL)
CREATE INDEX idx_sesiones_usuario_vigente
    ON sesiones (usuario_id)
    WHERE revocada_en IS NULL;
