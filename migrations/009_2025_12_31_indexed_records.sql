-- Migration: Indexed tables for AT Protocol Firehose/Jetstream consumption

CREATE TABLE repo_sync_state (
    did TEXT PRIMARY KEY,
    latest_rev TEXT NOT NULL,        -- TID of last processed commit
    indexed_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_repo_sync_state_indexed_at ON repo_sync_state(indexed_at);

-- Indexed decks from remote users
CREATE TABLE indexed_decks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    at_uri TEXT NOT NULL UNIQUE,
    did TEXT NOT NULL,
    rkey TEXT NOT NULL,
    -- Full record content (denormalized for query performance)
    title TEXT NOT NULL,
    description TEXT,
    tags TEXT[] DEFAULT '{}',
    card_refs TEXT[] DEFAULT '{}',   -- AT-URIs to cards in this deck
    source_refs TEXT[] DEFAULT '{}', -- AT-URIs to source materials
    license TEXT,
    record_created_at TIMESTAMPTZ NOT NULL, -- createdAt from the record
    indexed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ           -- Soft delete for tombstones
);

CREATE INDEX idx_indexed_decks_did ON indexed_decks(did);
CREATE INDEX idx_indexed_decks_at_uri ON indexed_decks(at_uri);
CREATE INDEX idx_indexed_decks_indexed_at ON indexed_decks(indexed_at);
CREATE INDEX idx_indexed_decks_tags ON indexed_decks USING GIN(tags);
CREATE INDEX idx_indexed_decks_deleted ON indexed_decks(deleted_at) WHERE deleted_at IS NULL;

-- Indexed cards from remote users
CREATE TABLE indexed_cards (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    at_uri TEXT NOT NULL UNIQUE,
    did TEXT NOT NULL,
    rkey TEXT NOT NULL,
    deck_ref TEXT NOT NULL,          -- AT-URI to parent deck
    front TEXT NOT NULL,
    back TEXT NOT NULL,
    card_type TEXT DEFAULT 'basic',
    hints TEXT[] DEFAULT '{}',
    record_created_at TIMESTAMPTZ NOT NULL,
    indexed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE INDEX idx_indexed_cards_did ON indexed_cards(did);
CREATE INDEX idx_indexed_cards_at_uri ON indexed_cards(at_uri);
CREATE INDEX idx_indexed_cards_deck_ref ON indexed_cards(deck_ref);
CREATE INDEX idx_indexed_cards_deleted ON indexed_cards(deleted_at) WHERE deleted_at IS NULL;

-- Indexed notes from remote users
CREATE TABLE indexed_notes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    at_uri TEXT NOT NULL UNIQUE,
    did TEXT NOT NULL,
    rkey TEXT NOT NULL,
    title TEXT NOT NULL,
    body TEXT NOT NULL,
    tags TEXT[] DEFAULT '{}',
    visibility TEXT DEFAULT 'public',
    record_created_at TIMESTAMPTZ NOT NULL,
    indexed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE INDEX idx_indexed_notes_did ON indexed_notes(did);
CREATE INDEX idx_indexed_notes_at_uri ON indexed_notes(at_uri);
CREATE INDEX idx_indexed_notes_tags ON indexed_notes USING GIN(tags);
CREATE INDEX idx_indexed_notes_visibility ON indexed_notes(visibility);
CREATE INDEX idx_indexed_notes_deleted ON indexed_notes(deleted_at) WHERE deleted_at IS NULL;
