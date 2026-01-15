ALTER TABLE magic_links
ADD COLUMN token_id TEXT NOT NULL;

CREATE UNIQUE INDEX IF NOT EXISTS magic_links_token_id_unique
ON magic_links(token_id);
