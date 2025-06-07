-- Drop the existing foreign key constraint
ALTER TABLE users DROP CONSTRAINT IF EXISTS users_client_id_fkey;

-- Re-add the constraint with ON DELETE CASCADE
ALTER TABLE users ADD CONSTRAINT users_client_id_fkey
    FOREIGN KEY (client_id) REFERENCES clients(id) ON DELETE CASCADE;