ALTER TABLE compass_flow_steps
    ADD COLUMN IF NOT EXISTS details JSONB;
