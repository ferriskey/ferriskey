ALTER TABLE webhooks
    ADD COLUMN secret TEXT NOT NULL DEFAULT (
        replace(gen_random_uuid()::text, '-', '') || replace(gen_random_uuid()::text, '-', '')
    );

ALTER TABLE webhooks
    ALTER COLUMN secret DROP DEFAULT;

ALTER TABLE webhooks
    ADD COLUMN last_delivery_status VARCHAR(16) NULL;

ALTER TABLE webhooks
    ADD COLUMN last_delivery_error VARCHAR(255) NULL;
