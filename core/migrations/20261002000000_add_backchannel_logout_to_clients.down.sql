ALTER TABLE clients
    DROP COLUMN IF EXISTS backchannel_logout_session_required,
    DROP COLUMN IF EXISTS backchannel_logout_uri;
