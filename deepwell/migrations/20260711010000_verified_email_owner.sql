-- Legacy updates could mark more than one active regular user as the owner of
-- the same email. None of those claims can be selected safely, so require all
-- members of each duplicate group to verify ownership again.
WITH duplicate_verified_emails AS (
    SELECT email
    FROM "user"
    WHERE user_type = 'regular'
      AND email_verified_at IS NOT NULL
      AND deleted_at IS NULL
    GROUP BY email
    HAVING count(*) > 1
)
UPDATE "user" AS owner
SET email_verified_at = NULL
FROM duplicate_verified_emails AS duplicate
WHERE owner.email = duplicate.email
  AND owner.user_type = 'regular'
  AND owner.email_verified_at IS NOT NULL
  AND owner.deleted_at IS NULL;

CREATE UNIQUE INDEX user_verified_email_active_unique_idx
    ON "user" (email)
    WHERE user_type = 'regular'
      AND email_verified_at IS NOT NULL
      AND deleted_at IS NULL;
