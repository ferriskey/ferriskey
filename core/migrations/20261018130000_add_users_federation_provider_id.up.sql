ALTER TABLE users
    ADD COLUMN IF NOT EXISTS federation_provider_id UUID;

ALTER TABLE users
    ADD CONSTRAINT users_federation_provider_id_fkey
    FOREIGN KEY (federation_provider_id) REFERENCES user_federation_providers(id) ON DELETE SET NULL
    NOT VALID;

CREATE OR REPLACE FUNCTION users_sync_federation_provider() RETURNS trigger
LANGUAGE plpgsql AS $$
DECLARE
    affected_user UUID;
    latest_provider UUID;
BEGIN
    PERFORM 1 FROM users
    WHERE id IN (NEW.user_id, OLD.user_id)
    ORDER BY id
    FOR UPDATE;

    FOR affected_user IN
        SELECT DISTINCT u FROM unnest(ARRAY[NEW.user_id, OLD.user_id]) AS u WHERE u IS NOT NULL
    LOOP
        SELECT provider_id INTO latest_provider
        FROM user_federation_mappings
        WHERE user_id = affected_user
        ORDER BY last_synced_at DESC
        LIMIT 1;

        UPDATE users
        SET federation_provider_id = latest_provider
        WHERE id = affected_user
          AND federation_provider_id IS DISTINCT FROM latest_provider;
    END LOOP;

    RETURN NULL;
END;
$$;

DROP TRIGGER IF EXISTS user_federation_mappings_sync_marker ON user_federation_mappings;
CREATE TRIGGER user_federation_mappings_sync_marker
    AFTER INSERT OR UPDATE OR DELETE ON user_federation_mappings
    FOR EACH ROW EXECUTE FUNCTION users_sync_federation_provider();
