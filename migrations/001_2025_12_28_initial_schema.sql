-- Initial schema for Malfestio
-- This migration creates the core tables for decks, notes, and cards
-- Note: ATProto lexicon records go to PDS, this DB is for blob storage & private data

CREATE TABLE IF NOT EXISTS schema_migrations (
    id SERIAL PRIMARY KEY,
    version TEXT NOT NULL UNIQUE,
    applied_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE decks (
    id UUID PRIMARY KEY,
    owner_did TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    tags TEXT[] NOT NULL DEFAULT '{}',
    visibility JSONB NOT NULL, -- Stores { type: "Private" | "Unlisted" | "Public" | "SharedWith", content?: [...] }
    published_at TIMESTAMPTZ,
    fork_of UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_decks_owner_did ON decks(owner_did);
CREATE INDEX idx_decks_visibility ON decks USING GIN(visibility);
CREATE INDEX idx_decks_created_at ON decks(created_at DESC);

CREATE TABLE cards (
    id UUID PRIMARY KEY,
    owner_did TEXT NOT NULL,
    deck_id UUID NOT NULL REFERENCES decks(id) ON DELETE CASCADE,
    front TEXT NOT NULL,
    back TEXT NOT NULL,
    media_url TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_cards_deck_id ON cards(deck_id);
CREATE INDEX idx_cards_owner_did ON cards(owner_did);

CREATE TABLE notes (
    id UUID PRIMARY KEY,
    owner_did TEXT NOT NULL,
    title TEXT NOT NULL,
    body TEXT NOT NULL,
    tags TEXT[] NOT NULL DEFAULT '{}',
    visibility JSONB NOT NULL,
    published_at TIMESTAMPTZ,
    links TEXT[] NOT NULL DEFAULT '{}', -- WikiLink style references to other notes
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_notes_owner_did ON notes(owner_did);
CREATE INDEX idx_notes_visibility ON notes USING GIN(visibility);
CREATE INDEX idx_notes_created_at ON notes(created_at DESC);
CREATE INDEX idx_notes_links ON notes USING GIN(links);

CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER update_decks_updated_at BEFORE UPDATE ON decks
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_cards_updated_at BEFORE UPDATE ON cards
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_notes_updated_at BEFORE UPDATE ON notes
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
