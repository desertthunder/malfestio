-- Social Layer: Follows and Comments
-- Implements Milestone H requirements

-- Follows table: User A follows User B
CREATE TABLE follows (
    follower_did TEXT NOT NULL,
    subject_did TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (follower_did, subject_did)
);

CREATE INDEX idx_follows_follower ON follows(follower_did);
CREATE INDEX idx_follows_subject_did ON follows(subject_did);

-- Comments table: Threaded comments on Decks (and potentially Cards in future)
CREATE TABLE comments (
    id UUID PRIMARY KEY,
    deck_id UUID NOT NULL REFERENCES decks(id) ON DELETE CASCADE,
    author_did TEXT NOT NULL,
    content TEXT NOT NULL,
    parent_id UUID REFERENCES comments(id) ON DELETE CASCADE, -- For threading
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_comments_deck_id ON comments(deck_id);
CREATE INDEX idx_comments_parent_id ON comments(parent_id);
CREATE INDEX idx_comments_author_did ON comments(author_did);
CREATE INDEX idx_comments_created_at ON comments(created_at);

CREATE TRIGGER update_comments_updated_at BEFORE UPDATE ON comments
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
