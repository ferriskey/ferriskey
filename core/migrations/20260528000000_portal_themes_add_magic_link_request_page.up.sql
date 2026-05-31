-- Add the per-page JSONB trees for the new portal pages introduced in this
-- branch. Mirrors the columns from 20260517120000_portal_themes_per_page_trees:
-- all default to an empty array so existing themes stay valid until an
-- admin customises the page.
--
-- - `page_magic_link_request`: request side of the magic-link flow (user
--   enters email, backend sends the link).
-- - `page_email_verified`    : success screen rendered after a verify-email
--   link has been clicked and confirmed by the backend.
-- - `page_totp_setup`        : first-time TOTP enrolment screen (QR code +
--   code confirmation), shown when a user must configure an authenticator
--   before continuing.

ALTER TABLE portal_themes
  ADD COLUMN page_magic_link_request JSONB NOT NULL DEFAULT '[]'::jsonb,
  ADD COLUMN page_email_verified     JSONB NOT NULL DEFAULT '[]'::jsonb,
  ADD COLUMN page_totp_setup         JSONB NOT NULL DEFAULT '[]'::jsonb;
