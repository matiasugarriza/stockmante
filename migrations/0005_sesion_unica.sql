-- Módulo 1 — sesión única garantizada por la base (T027 / T-AUTH-09).
--
-- La 0001 creó `idx_sesiones_usuario_vigente` sin UNIQUE. El servicio revoca la
-- sesión anterior antes de insertar, pero dos logins concurrentes del mismo
-- usuario podían pasar ese UPDATE a la vez y dejar dos filas vigentes.
--
-- El índice único parcial hace que la invariante la imponga PostgreSQL: el
-- segundo INSERT de una fila con `revocada_en IS NULL` falla con 23505 y el
-- servicio responde con un error interno genérico (sin filtrar detalles).
-- No se edita la 0001 porque ya está aplicada; ésta es la corrección.

-- 1) Reparar duplicados heredados: por usuario se conserva la sesión vigente
--    más reciente y se revocan las demás. Es estado de sesión efímero, no un
--    dato de negocio, y sin esto el índice único no se puede crear.
WITH vigentes AS (
    SELECT id,
           ROW_NUMBER() OVER (
               PARTITION BY usuario_id
               ORDER BY creada_en DESC, id
           ) AS orden
    FROM sesiones
    WHERE revocada_en IS NULL
)
UPDATE sesiones s
SET revocada_en = now()
FROM vigentes v
WHERE s.id = v.id AND v.orden > 1;

-- 2) Reemplazar el índice no único por uno único parcial.
DROP INDEX idx_sesiones_usuario_vigente;

CREATE UNIQUE INDEX idx_sesiones_usuario_vigente
    ON sesiones (usuario_id)
    WHERE revocada_en IS NULL;
