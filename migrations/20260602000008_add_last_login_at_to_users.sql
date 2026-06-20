-- Migration: track last successful login on tenant users
--
-- Nullable: existing users have never recorded a login under this column, and a
-- user that has never logged in legitimately has no value. The LoginUseCase
-- stamps it (via UPDATE) on each successful authentication.

ALTER TABLE users ADD COLUMN IF NOT EXISTS last_login_at TIMESTAMPTZ NULL;
