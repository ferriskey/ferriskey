-- Add up migration script here
CREATE TABLE token_exchange_policies (
  id UUID PRIMARY KEY,
  realm_id UUID NOT NULL,
  client_id UUID NOT NULL,
  target_audience VARCHAR(255) NOT NULL,
  allowed_scopes TEXT NULL,
  allow_impersonation BOOLEAN NOT NULL DEFAULT FALSE,
  allow_delegation BOOLEAN NOT NULL DEFAULT FALSE,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,

  CONSTRAINT fk_token_exchange_policies_realm
    FOREIGN KEY (realm_id)
    REFERENCES realms (id)
    ON DELETE CASCADE,

  CONSTRAINT fk_token_exchange_policies_client
    FOREIGN KEY (client_id)
    REFERENCES clients (id)
    ON DELETE CASCADE
);

ALTER TABLE token_exchange_policies
  ADD CONSTRAINT unique_actor_audience
  UNIQUE (realm_id, client_id, target_audience);

CREATE INDEX idx_token_exchange_policies_client_id
  ON token_exchange_policies (client_id);
