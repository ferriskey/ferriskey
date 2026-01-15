DROP INDEX IF EXISTS magic_links_token_id_unique;

ALTER TABLE magic_links
DROP COLUMN IF EXISTS token_id;
