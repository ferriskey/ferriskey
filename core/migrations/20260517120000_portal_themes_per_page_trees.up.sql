-- Restructure portal_themes:
--   * drop the 1-per-realm UNIQUE so a realm can own many themes
--   * rename `config` to `design_tokens`
--   * add metadata columns (name, layout_id) and seven per-page JSONB trees
-- Also expose the active theme through realm_settings.portal_theme_id.

ALTER TABLE portal_themes DROP CONSTRAINT uq_portal_themes_realm;

ALTER TABLE portal_themes RENAME COLUMN config TO design_tokens;

ALTER TABLE portal_themes
  ADD COLUMN name VARCHAR(255) NOT NULL DEFAULT 'Default',
  ADD COLUMN layout_id UUID NULL,
  ADD COLUMN page_login              JSONB NOT NULL DEFAULT '[]'::jsonb,
  ADD COLUMN page_register           JSONB NOT NULL DEFAULT '[]'::jsonb,
  ADD COLUMN page_totp               JSONB NOT NULL DEFAULT '[]'::jsonb,
  ADD COLUMN page_forgot_password    JSONB NOT NULL DEFAULT '[]'::jsonb,
  ADD COLUMN page_reset_password     JSONB NOT NULL DEFAULT '[]'::jsonb,
  ADD COLUMN page_magic_link_verify  JSONB NOT NULL DEFAULT '[]'::jsonb,
  ADD COLUMN page_verify_email       JSONB NOT NULL DEFAULT '[]'::jsonb,
  ADD CONSTRAINT fk_portal_themes_layout
    FOREIGN KEY (layout_id)
    REFERENCES portal_layouts (id)
    ON DELETE RESTRICT;

CREATE INDEX idx_portal_themes_layout_id
  ON portal_themes (layout_id);

ALTER TABLE realm_settings
  ADD COLUMN portal_theme_id UUID NULL,
  ADD CONSTRAINT fk_realm_settings_portal_theme
    FOREIGN KEY (portal_theme_id)
    REFERENCES portal_themes (id)
    ON DELETE SET NULL;

-- Backfill: for every realm that already has a portal_themes row, point
-- realm_settings.portal_theme_id at it, and preselect the realm's default
-- portal_layouts row (if any) as the theme's layout.
UPDATE portal_themes pt
SET layout_id = pl.id
FROM portal_layouts pl
WHERE pl.realm_id = pt.realm_id
  AND pl.is_default = TRUE;

UPDATE realm_settings rs
SET portal_theme_id = pt.id
FROM portal_themes pt
WHERE pt.realm_id = rs.realm_id;
