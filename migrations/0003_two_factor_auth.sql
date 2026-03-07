-- Two-factor authentication support

ALTER TABLE users
  ADD COLUMN twofa_method VARCHAR(10) NOT NULL DEFAULT 'none',
  ADD COLUMN totp_secret VARCHAR(255),
  ADD COLUMN totp_enabled BOOLEAN NOT NULL DEFAULT false;

CREATE TABLE recovery_codes (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  code_hash VARCHAR(128) NOT NULL,
  used BOOLEAN NOT NULL DEFAULT false,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_recovery_codes_user ON recovery_codes(user_id);

CREATE TABLE twofa_pending (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  token_hash VARCHAR(128) NOT NULL UNIQUE,
  method VARCHAR(10) NOT NULL,
  expires_at TIMESTAMPTZ NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_twofa_pending_token ON twofa_pending(token_hash);
CREATE INDEX idx_twofa_pending_user ON twofa_pending(user_id);
