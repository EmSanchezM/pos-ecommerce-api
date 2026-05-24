-- Rename the tenant 'system:admin' permission to 'organization:admin'.
-- This is a data-only rename — no schema change required.
-- All FKs (role_permissions, etc.) reference the UUID PK, not the code column,
-- so they are unaffected.
--
-- After this migration: api-gateway auth.rs and any seed code referencing
-- "system:admin" as a string literal must also be updated (see P1-T10).

UPDATE permissions
SET code = 'organization:admin'
WHERE code = 'system:admin';
