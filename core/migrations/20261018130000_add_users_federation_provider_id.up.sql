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
    affected_user_ids UUID[];
    latest_provider UUID;
BEGIN
    IF TG_OP = 'INSERT' THEN
        affected_user_ids := ARRAY[NEW.user_id];
    ELSIF TG_OP = 'DELETE' THEN
        affected_user_ids := ARRAY[OLD.user_id];
    ELSE
        affected_user_ids := ARRAY[NEW.user_id, OLD.user_id];
    END IF;

    PERFORM 1 FROM users
    WHERE id = ANY (affected_user_ids)
    ORDER BY id
    FOR UPDATE;

    FOR affected_user IN
        SELECT DISTINCT u FROM unnest(affected_user_ids) AS u WHERE u IS NOT NULL
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
