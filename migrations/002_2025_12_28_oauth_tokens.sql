-- OAuth tokens and AT-URI storage for AT Protocol integration
-- Adds tables for OAuth session management and AT-URI references

-- OAuth sessions for tracking authorization flow state
CREATE TABLE oauth_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    state TEXT NOT NULL UNIQUE,
    code_verifier TEXT NOT NULL,
    dpop_private_key BYTEA NOT NULL,
    did TEXT,
    pds_url TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL DEFAULT NOW() + INTERVAL '10 minutes'
);

CREATE INDEX idx_oauth_sessions_state ON oauth_sessions(state);
CREATE INDEX idx_oauth_sessions_expires_at ON oauth_sessions(expires_at);

-- OAuth tokens for authenticated users
CREATE TABLE oauth_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    did TEXT NOT NULL UNIQUE,
    pds_url TEXT NOT NULL,
    access_token TEXT NOT NULL,
    refresh_token TEXT,
    token_type TEXT NOT NULL DEFAULT 'DPoP',
    expires_at TIMESTAMPTZ,
    dpop_private_key BYTEA NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_oauth_tokens_did ON oauth_tokens(did);

CREATE TRIGGER update_oauth_tokens_updated_at BEFORE UPDATE ON oauth_tokens
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Add AT-URI columns to existing tables for tracking PDS record references
ALTER TABLE decks ADD COLUMN at_uri TEXT;
ALTER TABLE cards ADD COLUMN at_uri TEXT;
ALTER TABLE notes ADD COLUMN at_uri TEXT;

CREATE INDEX idx_decks_at_uri ON decks(at_uri) WHERE at_uri IS NOT NULL;
CREATE INDEX idx_cards_at_uri ON cards(at_uri) WHERE at_uri IS NOT NULL;
CREATE INDEX idx_notes_at_uri ON notes(at_uri) WHERE at_uri IS NOT NULL;

-- Cleanup job for expired sessions (run periodically via cron or similar)
-- DELETE FROM oauth_sessions WHERE expires_at < NOW();
