ALTER TABLE realm_settings
    DROP COLUMN IF EXISTS webhook_retry_max_attempts,
    DROP COLUMN IF EXISTS webhook_retry_base_delay_ms,
    DROP COLUMN IF EXISTS webhook_retry_max_delay_ms,
    DROP COLUMN IF EXISTS webhook_retry_max_total_delay_ms;

ALTER TABLE webhooks
    DROP COLUMN IF EXISTS retry_max_attempts,
    DROP COLUMN IF EXISTS retry_base_delay_ms,
    DROP COLUMN IF EXISTS retry_max_delay_ms,
    DROP COLUMN IF EXISTS retry_max_total_delay_ms;

DROP TABLE IF EXISTS webhook_deliveries;
