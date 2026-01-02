-- Make dpop_private_key nullable to support app password sessions
-- App password sessions don't use DPoP, only OAuth sessions do

ALTER TABLE oauth_tokens ALTER COLUMN dpop_private_key DROP NOT NULL;

COMMENT ON COLUMN oauth_tokens.dpop_private_key IS 'DPoP private key for OAuth sessions. NULL for app password sessions.';
