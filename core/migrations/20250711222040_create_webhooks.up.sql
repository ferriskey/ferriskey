-- Add up migration script here
CREATE TABLE webhooks (
  id UUID PRIMARY KEY,
  realm_id UUID NOT NULL,
  endpoint VARCHAR(255) NOT NULL,
  triggered_at TIMESTAMP NULL,
  updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,

  CONSTRAINT fk_realm
    FOREIGN KEY (realm_id)
    REFERENCES realms (id)
    ON DELETE CASCADE
);

CREATE TABLE webhook_subscribers (
  id UUID PRIMARY KEY,
  name VARCHAR(255) NOT NULL,
  webhook_id UUID NOT NULL,

  CONSTRAINT fk_webhook
    FOREIGN KEY (webhook_id)
    REFERENCES webhooks (id)
    ON DELETE CASCADE
)
