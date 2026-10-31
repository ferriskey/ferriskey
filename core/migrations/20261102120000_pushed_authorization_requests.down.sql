DROP TABLE IF EXISTS pushed_authorization_requests;

ALTER TABLE realm_settings DROP COLUMN IF EXISTS require_pushed_authorization_requests;

ALTER TABLE clients DROP COLUMN IF EXISTS require_pushed_authorization_requests;
