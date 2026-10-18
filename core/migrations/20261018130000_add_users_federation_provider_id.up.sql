ALTER TABLE users
    ADD COLUMN IF NOT EXISTS federation_provider_id UUID REFERENCES user_federation_providers(id) ON DELETE SET NULL;

UPDATE users u
SET federation_provider_id = m.provider_id
FROM (
    SELECT DISTINCT ON (user_id) user_id, provider_id
    FROM user_federation_mappings
    ORDER BY user_id, last_synced_at DESC
) m
WHERE m.user_id = u.id;
