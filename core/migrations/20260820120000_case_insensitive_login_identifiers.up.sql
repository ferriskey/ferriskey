UPDATE users SET email = trim(email) WHERE email IS NOT NULL AND email <> trim(email);
UPDATE users SET username = trim(username) WHERE username <> trim(username);
UPDATE users SET email = NULL WHERE email = '';

DO $$
DECLARE
    conflicts text;
BEGIN
    SELECT string_agg(format('realm %s: %s', realm_id, identifiers), E'\n')
    INTO conflicts
    FROM (
        SELECT realm_id, string_agg(DISTINCT email, ', ') AS identifiers
        FROM users
        WHERE email IS NOT NULL
        GROUP BY realm_id, lower(email)
        HAVING count(*) > 1
        UNION ALL
        SELECT realm_id, string_agg(DISTINCT username, ', ') AS identifiers
        FROM users
        GROUP BY realm_id, lower(username)
        HAVING count(*) > 1
    ) AS duplicated;

    IF conflicts IS NOT NULL THEN
        RAISE EXCEPTION E'Accounts differing only by letter case share a login identifier and must be resolved before this migration can enforce uniqueness. Merge or rename them, then run the migration again.\n%', conflicts;
    END IF;
END $$;

ALTER TABLE users DROP CONSTRAINT IF EXISTS unique_email_per_realm;
ALTER TABLE users DROP CONSTRAINT IF EXISTS unique_username_realm_id;

CREATE UNIQUE INDEX unique_lower_email_per_realm ON users (realm_id, lower(email)) WHERE email IS NOT NULL;
CREATE UNIQUE INDEX unique_lower_username_per_realm ON users (realm_id, lower(username));
