ALTER TABLE jwt_keys
    ADD COLUMN rotated_at TIMESTAMP NULL,
    ADD COLUMN retire_at TIMESTAMP NULL,
    ADD COLUMN revoked_at TIMESTAMP NULL,
    ADD CONSTRAINT jwt_keys_rotation_has_retirement
        CHECK ((rotated_at IS NULL) = (retire_at IS NULL));

DROP INDEX IF EXISTS jwt_keys_realm_id_key;

CREATE INDEX jwt_keys_realm_id_idx ON jwt_keys (realm_id);

CREATE UNIQUE INDEX jwt_keys_one_active_per_realm
    ON jwt_keys (realm_id)
    WHERE rotated_at IS NULL AND revoked_at IS NULL;
