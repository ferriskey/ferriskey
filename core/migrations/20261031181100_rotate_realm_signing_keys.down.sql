WITH retired_keys AS (
    DELETE FROM jwt_keys
    WHERE rotated_at IS NOT NULL OR revoked_at IS NOT NULL
    RETURNING id, realm_id, private_key, public_key, created_at
)
INSERT INTO jwt_keys_superseded (id, realm_id, private_key, public_key, created_at)
SELECT id, realm_id, private_key, public_key, created_at
FROM retired_keys
ON CONFLICT (id) DO NOTHING;

DROP INDEX IF EXISTS jwt_keys_one_active_per_realm;
DROP INDEX IF EXISTS jwt_keys_realm_id_idx;

CREATE UNIQUE INDEX IF NOT EXISTS jwt_keys_realm_id_key ON jwt_keys (realm_id);

ALTER TABLE jwt_keys
    DROP CONSTRAINT IF EXISTS jwt_keys_rotation_has_retirement,
    DROP COLUMN IF EXISTS revoked_at,
    DROP COLUMN IF EXISTS retire_at,
    DROP COLUMN IF EXISTS rotated_at;
