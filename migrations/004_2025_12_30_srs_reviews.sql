-- SRS (Spaced Repetition System) schema
--
-- Tracks per-user review state for each card

CREATE TABLE card_reviews (
    id UUID PRIMARY KEY,
    card_id UUID NOT NULL REFERENCES cards(id) ON DELETE CASCADE,
    user_did TEXT NOT NULL,
    -- SM-2 algorithm fields
    ease_factor REAL NOT NULL DEFAULT 2.5,
    interval_days INTEGER NOT NULL DEFAULT 0,
    repetitions INTEGER NOT NULL DEFAULT 0,
    -- Scheduling
    due_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_reviewed_at TIMESTAMPTZ,
    -- Stats
    total_reviews INTEGER NOT NULL DEFAULT 0,
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- Each user has one review state per card
    UNIQUE(card_id, user_did)
);

CREATE INDEX idx_card_reviews_user_due ON card_reviews(user_did, due_at);
CREATE INDEX idx_card_reviews_card_id ON card_reviews(card_id);

CREATE TRIGGER update_card_reviews_updated_at BEFORE UPDATE ON card_reviews
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TABLE user_study_stats (
    id UUID PRIMARY KEY,
    user_did TEXT NOT NULL UNIQUE,
    current_streak INTEGER NOT NULL DEFAULT 0,
    longest_streak INTEGER NOT NULL DEFAULT 0,
    last_study_date DATE,
    total_cards_reviewed INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_user_study_stats_did ON user_study_stats(user_did);

CREATE TRIGGER update_user_study_stats_updated_at BEFORE UPDATE ON user_study_stats
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
