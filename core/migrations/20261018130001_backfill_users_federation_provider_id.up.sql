LOCK TABLE user_federation_mappings IN SHARE MODE;

UPDATE users u
SET federation_provider_id = m.provider_id
FROM (
    SELECT DISTINCT ON (user_id) user_id, provider_id
    FROM user_federation_mappings
    ORDER BY user_id, last_synced_at DESC
) m
WHERE m.user_id = u.id
  AND u.federation_provider_id IS DISTINCT FROM m.provider_id;
