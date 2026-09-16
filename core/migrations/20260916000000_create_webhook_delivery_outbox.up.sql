CREATE TABLE IF NOT EXISTS webhook_deliveries (
    id                UUID PRIMARY KEY,
    realm_id          UUID         NOT NULL REFERENCES realms(id) ON DELETE CASCADE,
    webhook_id        UUID         NOT NULL REFERENCES webhooks(id) ON DELETE CASCADE,
    event             VARCHAR(64)  NOT NULL,
    resource_id       UUID         NOT NULL,
    payload           JSONB        NOT NULL,
    status            VARCHAR(16)  NOT NULL,
    attempt_count     INTEGER      NOT NULL DEFAULT 0,
    next_attempt_at   TIMESTAMP    NULL,
    leased_until      TIMESTAMP    NULL,
    last_attempt_at   TIMESTAMP    NULL,
    last_status_code  INTEGER      NULL,
    last_error_code   VARCHAR(32)  NULL,
    last_error_detail VARCHAR(512) NULL,
    created_at        TIMESTAMP    NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at        TIMESTAMP    NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_webhook_deliveries_due
    ON webhook_deliveries (next_attempt_at)
    WHERE status = 'pending';

CREATE INDEX IF NOT EXISTS idx_webhook_deliveries_lease
    ON webhook_deliveries (leased_until)
    WHERE status = 'delivering';

CREATE INDEX IF NOT EXISTS idx_webhook_deliveries_webhook
    ON webhook_deliveries (webhook_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_webhook_deliveries_realm
    ON webhook_deliveries (realm_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_webhook_deliveries_purge
    ON webhook_deliveries (created_at)
    WHERE status IN ('succeeded', 'failed');

ALTER TABLE webhooks
    ADD COLUMN IF NOT EXISTS retry_max_attempts       INTEGER NULL,
    ADD COLUMN IF NOT EXISTS retry_base_delay_ms      INTEGER NULL,
    ADD COLUMN IF NOT EXISTS retry_max_delay_ms       INTEGER NULL,
    ADD COLUMN IF NOT EXISTS retry_max_total_delay_ms INTEGER NULL;

ALTER TABLE realm_settings
    ADD COLUMN IF NOT EXISTS webhook_retry_max_attempts       INTEGER NULL,
    ADD COLUMN IF NOT EXISTS webhook_retry_base_delay_ms      INTEGER NULL,
    ADD COLUMN IF NOT EXISTS webhook_retry_max_delay_ms       INTEGER NULL,
    ADD COLUMN IF NOT EXISTS webhook_retry_max_total_delay_ms INTEGER NULL;
