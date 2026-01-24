-- Remove unique constraint for email per realm
ALTER TABLE users DROP CONSTRAINT unique_email_per_realm;
