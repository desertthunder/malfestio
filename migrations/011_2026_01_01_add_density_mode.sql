-- Add density_mode column to user_prefs table

ALTER TABLE user_prefs ADD COLUMN IF NOT EXISTS density_mode VARCHAR(20);

-- Valid values: 'compact', 'comfortable', 'spacious'
-- TODO: consider an enum
-- NULL means use default (comfortable)
