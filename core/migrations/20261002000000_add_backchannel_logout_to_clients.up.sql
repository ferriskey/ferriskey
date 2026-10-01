ALTER TABLE clients
    ADD COLUMN IF NOT EXISTS backchannel_logout_uri TEXT,
    ADD COLUMN IF NOT EXISTS backchannel_logout_session_required BOOLEAN NOT NULL DEFAULT true;
