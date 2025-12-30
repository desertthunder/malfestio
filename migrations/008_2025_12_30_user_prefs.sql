-- User preferences for onboarding and personalization
-- Tracks onboarding completion and user persona selection

CREATE TABLE user_prefs (
    id UUID PRIMARY KEY,
    user_did TEXT NOT NULL UNIQUE,
    persona TEXT,  -- 'learner' | 'creator' | 'curator' | NULL
    onboarding_completed_at TIMESTAMPTZ,
    tutorial_deck_completed BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_user_prefs_did ON user_prefs(user_did);

CREATE TRIGGER update_user_prefs_updated_at BEFORE UPDATE ON user_prefs
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
