CREATE TABLE IF NOT EXISTS jwt_keys_revoked (
    id UUID PRIMARY KEY,
    realm_id UUID NOT NULL REFERENCES realms(id) ON DELETE CASCADE,
    public_key TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL,
    revoked_at TIMESTAMP NOT NULL
);

WITH revoked_keys AS (
    DELETE FROM jwt_keys
    WHERE revoked_at IS NOT NULL
    RETURNING id, realm_id, public_key, created_at, revoked_at
)
INSERT INTO jwt_keys_revoked (id, realm_id, public_key, created_at, revoked_at)
SELECT id, realm_id, public_key, created_at, revoked_at
FROM revoked_keys
ON CONFLICT (id) DO NOTHING;

WITH retired_keys AS (
    DELETE FROM jwt_keys
    WHERE rotated_at IS NOT NULL
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
