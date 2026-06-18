ALTER TABLE realm_settings
    DROP COLUMN IF EXISTS seawatch_pii_mode,
    DROP COLUMN IF EXISTS seawatch_pseudo_key;
