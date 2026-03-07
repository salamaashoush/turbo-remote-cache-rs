-- User activation/deactivation
ALTER TABLE users
  ADD COLUMN is_active BOOLEAN NOT NULL DEFAULT true;

-- Organization limits
ALTER TABLE organizations
  ADD COLUMN cache_size_limit_bytes BIGINT,
  ADD COLUMN max_tokens INT;
