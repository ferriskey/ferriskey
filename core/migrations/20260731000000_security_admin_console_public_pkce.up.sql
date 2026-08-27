-- The admin console is a browser SPA and was seeded as a confidential client
-- holding a secret it can never actually send. Now that the authorization_code
-- grant authenticates confidential clients, that secret would block the console
-- login outright, and it never provided any protection anyway.
--
-- Convert it to what it really is: a public client, with PKCE as the mechanism
-- that binds an authorization code to the browser that requested it.
UPDATE clients
SET public_client = true,
    secret        = NULL,
    client_type   = 'public',
    require_pkce  = true,
    updated_at    = NOW()
WHERE client_id = 'security-admin-console';
