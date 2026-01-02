-- Migration: Add card_type column to cards table
-- Date: 2026-01-02
-- Issue: card_type was not being persisted, always returning 'basic'

ALTER TABLE cards ADD COLUMN IF NOT EXISTS card_type TEXT DEFAULT 'basic';
