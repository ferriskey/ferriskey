ALTER TABLE portal_themes
    ADD COLUMN page_consent JSONB NOT NULL DEFAULT '[]'::jsonb;
