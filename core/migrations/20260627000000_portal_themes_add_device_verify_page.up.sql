-- Per-page JSONB trees for the RFC 8628 device flow pages. Mirrors the
-- columns from 20260517120000_portal_themes_per_page_trees: default to an
-- empty array so existing themes stay valid until an admin customises them.
--
-- - `page_device_verify`  : device-flow consent screen where the user enters
--   the `user_code` shown on their device and approves / denies the request.
-- - `page_device_verified`: success screen rendered once the device request
--   has been approved ("you can return to your device").

ALTER TABLE portal_themes
  ADD COLUMN page_device_verify   JSONB NOT NULL DEFAULT '[]'::jsonb,
  ADD COLUMN page_device_verified JSONB NOT NULL DEFAULT '[]'::jsonb;
