-- Add down migration script here

-- clients: remove ON DELETE CASCADE from realm_id
ALTER TABLE clients DROP CONSTRAINT IF EXISTS clients_realm_id_fkey;
ALTER TABLE clients ADD CONSTRAINT clients_realm_id_fkey
    FOREIGN KEY (realm_id) REFERENCES realms(id);

-- users: remove ON DELETE CASCADE from realm_id
ALTER TABLE users DROP CONSTRAINT IF EXISTS users_realm_id_fkey;
ALTER TABLE users ADD CONSTRAINT users_realm_id_fkey
    FOREIGN KEY (realm_id) REFERENCES realms(id);
