-- Add up migration script here
CREATE TABLE clients (
  id UUID PRIMARY KEY,
  realm_id UUID NOT NULL REFERENCES realms(id),
  name VARCHAR(255) NOT NULL,
  client_id VARCHAR(255) NOT NULL,
  secret VARCHAR(255) NULL,
  enabled BOOLEAN NOT NULL DEFAULT TRUE,
  protocol VARCHAR(255) NOT NULL,
  public_client BOOLEAN NOT NULL DEFAULT FALSE,
  service_account_enabled BOOLEAN NOT NULL DEFAULT FALSE,
  client_type VARCHAR(255) NOT NULL,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);



-- Unique constraint for client_id per realm_id
ALTER TABLE clients ADD CONSTRAINT unique_client_id_per_realm_id UNIQUE (client_id, realm_id);