CREATE TABLE IF NOT EXISTS jwt_keys_superseded (
    id UUID PRIMARY KEY,
    realm_id UUID NOT NULL REFERENCES realms(id) ON DELETE CASCADE,
    private_key TEXT NOT NULL,
    public_key TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL,
    superseded_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

WITH ranked_realm_keys AS (
    SELECT
        id,
        realm_id,
        private_key,
        public_key,
        created_at,
        ROW_NUMBER() OVER (PARTITION BY realm_id ORDER BY ctid) AS scan_position
    FROM jwt_keys
),
archived_realm_keys AS (
    INSERT INTO jwt_keys_superseded (id, realm_id, private_key, public_key, created_at)
    SELECT id, realm_id, private_key, public_key, created_at
    FROM ranked_realm_keys
    WHERE scan_position > 1
    ON CONFLICT (id) DO NOTHING
    RETURNING id
)
DELETE FROM jwt_keys
WHERE id IN (SELECT id FROM archived_realm_keys);

ALTER TABLE jwt_keys ADD COLUMN IF NOT EXISTS certificate TEXT;

CREATE UNIQUE INDEX IF NOT EXISTS jwt_keys_realm_id_key ON jwt_keys (realm_id);
