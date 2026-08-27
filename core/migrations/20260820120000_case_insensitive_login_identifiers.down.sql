DROP INDEX IF EXISTS unique_lower_username_per_realm;
DROP INDEX IF EXISTS unique_lower_email_per_realm;

ALTER TABLE users ADD CONSTRAINT unique_username_realm_id UNIQUE (username, realm_id);
ALTER TABLE users ADD CONSTRAINT unique_email_per_realm UNIQUE (email, realm_id);
