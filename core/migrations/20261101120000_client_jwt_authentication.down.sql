DROP TABLE IF EXISTS client_assertion_jtis;

ALTER TABLE clients
    DROP CONSTRAINT IF EXISTS clients_single_key_source,
    DROP COLUMN IF EXISTS jwks_uri,
    DROP COLUMN IF EXISTS jwks,
    DROP COLUMN IF EXISTS token_endpoint_auth_method;
