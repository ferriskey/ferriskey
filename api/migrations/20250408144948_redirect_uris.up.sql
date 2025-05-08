-- Add up migration script here
CREATE TABLE redirect_uris (
   id UUID PRIMARY KEY,
   client_id UUID NOT NULL,
   value TEXT NOT NULL,
   enabled BOOLEAN DEFAULT true NOT NULL,
   created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
   updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,

   CONSTRAINT fk_client
       FOREIGN KEY (client_id)
           REFERENCES clients(id)
           ON DELETE CASCADE
);
