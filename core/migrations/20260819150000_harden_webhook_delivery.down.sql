ALTER TABLE webhooks
    DROP COLUMN last_delivery_error;

ALTER TABLE webhooks
    DROP COLUMN last_delivery_status;

ALTER TABLE webhooks
    DROP COLUMN secret;
