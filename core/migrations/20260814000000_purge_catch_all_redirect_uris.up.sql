-- FK-002: every realm's `security-admin-console` was seeded with the redirect-URI
-- pattern `^/*`. Registered values were compiled as regexes and matched with
-- `is_match` (a substring search), and `^/*` means "start of string, then zero or
-- more slashes" — it matches the empty string at position 0, hence every URI. Any
-- host an attacker named was therefore an accepted redirect target for the admin
-- console, on a default installation.
--
-- `^http://localhost:[0-9]+/.*` goes with it: it was seeded unconditionally,
-- including in production, and allowed exfiltration to a listener on the victim's
-- own machine.
--
-- Removing them from the seeding code is not enough — the rows are already in the
-- database, and the startup back-fill would have re-inserted them. Matching is
-- exact from now on, so both values are dead weight at best.
DELETE FROM redirect_uris
WHERE value IN ('^/*', '^http://localhost:[0-9]+/.*');

DELETE FROM post_logout_redirect_uris
WHERE value IN ('^/*', '^http://localhost:[0-9]+/.*');
