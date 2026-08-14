-- Deliberately empty: re-inserting `^/*` would restore an unauthenticated path to
-- full administrator account takeover (FK-002). A rollback of this migration must
-- not reopen it. Operators who relied on the catch-all should register their
-- console callback explicitly instead:
--   INSERT INTO redirect_uris (id, client_id, value, enabled)
--   SELECT gen_random_uuid(), id, 'https://console.example/realms/master/authentication/callback', true
--   FROM clients WHERE client_id = 'security-admin-console';
SELECT 1;
