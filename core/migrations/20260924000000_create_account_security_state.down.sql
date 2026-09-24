ALTER TABLE credentials
  DROP COLUMN IF EXISTS last_used_at;

DROP TABLE IF EXISTS passkey_registrations;

DROP TABLE IF EXISTS account_elevations;
