-- Add unique constraint for email per realm
ALTER TABLE users ADD CONSTRAINT unique_email_per_realm UNIQUE (email, realm_id);
