ALTER TABLE auth_sessions
    ADD COLUMN acr_essential BOOLEAN NOT NULL DEFAULT false;
