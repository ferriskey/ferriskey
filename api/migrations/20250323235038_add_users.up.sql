-- Add up migration script here
CREATE TABLE users (
    id UUID PRIMARY KEY,
    realm_id UUID REFERENCES realms(id) NOT NULL,
    client_id UUID REFERENCES clients(id),
    username VARCHAR(255) NOT NULL,
    firstname VARCHAR(255) NOT NULL,
    lastname VARCHAR(255) NOT NULL,
    email VARCHAR(255) NOT NULL,
    email_verified BOOLEAN NOT NULL DEFAULT FALSE,
    enabled BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);


-- Add unique constraint to username and realm_id
ALTER TABLE users ADD CONSTRAINT unique_username_realm_id UNIQUE (username, realm_id);

-- Add unique constraint to realm_id
ALTER TABLE users ADD CONSTRAINT unique_client_id UNIQUE (client_id);