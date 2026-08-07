-- Add up migration script here
CREATE TABLE token_exchange_policies (
  id UUID PRIMARY KEY,
  realm_id UUID NOT NULL REFERENCES realms(id),
  client_id UUID NOT NULL REFERENCES clients(id),
  target_audience VARCHAR(255) NOT NULL,
  allowed_scopes TEXT NULL,
  allow_impersonation BOOLEAN NOT NULL DEFAULT TRUE,
  allow_delegation BOOLEAN NOT NULL DEFAULT FALSE,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

ALTER TABLE token_exchange_policies
  ADD CONSTRAINT unique_actor_audience
  UNIQUE (realm_id, client_id, target_audience);
