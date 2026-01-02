-- Migration: Add language, visibility, updatedAt for lexicon consistency
-- Date: 2026-01-02
-- Description: Adds optional fields to match lexicon schema updates

-- Local tables: Add language to all content types
ALTER TABLE decks ADD COLUMN IF NOT EXISTS language TEXT;
ALTER TABLE cards ADD COLUMN IF NOT EXISTS language TEXT;
ALTER TABLE notes ADD COLUMN IF NOT EXISTS language TEXT;

-- Local tables: Add visibility to cards (decks and notes already have it)
ALTER TABLE cards ADD COLUMN IF NOT EXISTS visibility JSONB;

-- Indexed tables: Add visibility, updatedAt, language for remote records
ALTER TABLE indexed_decks ADD COLUMN IF NOT EXISTS visibility TEXT;
ALTER TABLE indexed_decks ADD COLUMN IF NOT EXISTS updated_at TIMESTAMPTZ;
ALTER TABLE indexed_decks ADD COLUMN IF NOT EXISTS language TEXT;

ALTER TABLE indexed_cards ADD COLUMN IF NOT EXISTS visibility TEXT;
ALTER TABLE indexed_cards ADD COLUMN IF NOT EXISTS updated_at TIMESTAMPTZ;
ALTER TABLE indexed_cards ADD COLUMN IF NOT EXISTS language TEXT;

ALTER TABLE indexed_notes ADD COLUMN IF NOT EXISTS updated_at TIMESTAMPTZ;
ALTER TABLE indexed_notes ADD COLUMN IF NOT EXISTS language TEXT;

-- Add indexes for common visibility queries
CREATE INDEX IF NOT EXISTS idx_cards_visibility ON cards USING GIN(visibility);
CREATE INDEX IF NOT EXISTS idx_indexed_decks_visibility ON indexed_decks(visibility);
CREATE INDEX IF NOT EXISTS idx_indexed_cards_visibility ON indexed_cards(visibility);
