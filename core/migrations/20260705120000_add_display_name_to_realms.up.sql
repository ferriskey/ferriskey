-- Add a human-readable display name to realms, separate from `name` (the URL slug).
-- Nullable: existing realms keep their slug as the only label and fall back to it.
ALTER TABLE realms ADD COLUMN display_name VARCHAR(255);
