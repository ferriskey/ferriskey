ALTER TABLE portal_themes
    ADD COLUMN page_account_chooser JSONB NOT NULL DEFAULT '[]'::jsonb;
