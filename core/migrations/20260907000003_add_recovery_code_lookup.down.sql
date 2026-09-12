DROP INDEX IF EXISTS idx_credentials_recovery_code_lookup;

ALTER TABLE credentials
    DROP COLUMN recovery_code_lookup;
