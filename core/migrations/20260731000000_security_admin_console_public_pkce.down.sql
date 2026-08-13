-- Restores the confidential shape. The original secret is unrecoverable, so a
-- fresh one is generated; anything that stored the old value must be updated.
UPDATE clients
SET public_client = false,
    secret        = gen_random_uuid()::text,
    client_type   = 'confidential',
    require_pkce  = false,
    updated_at    = NOW()
WHERE client_id = 'security-admin-console';
