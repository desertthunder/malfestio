-- Sync tracking infrastructure for bi-directional PDS synchronization
-- Adds version tracking and sync status to core tables

-- Sync status enum (idempotent creation)
DO $$ BEGIN
    CREATE TYPE sync_status AS ENUM (
        'local_only',    -- Never synced to PDS
        'synced',        -- In sync with PDS
        'pending_push',  -- Local changes need to be pushed
        'conflict'       -- Local and remote both changed
    );
EXCEPTION
    WHEN duplicate_object THEN null;
END $$;

ALTER TABLE decks
    ADD COLUMN IF NOT EXISTS version INTEGER NOT NULL DEFAULT 1,
    ADD COLUMN IF NOT EXISTS pds_cid TEXT,
    ADD COLUMN IF NOT EXISTS pds_uri TEXT,
    ADD COLUMN IF NOT EXISTS sync_status sync_status NOT NULL DEFAULT 'local_only',
    ADD COLUMN IF NOT EXISTS last_synced_at TIMESTAMPTZ;

ALTER TABLE cards
    ADD COLUMN IF NOT EXISTS version INTEGER NOT NULL DEFAULT 1,
    ADD COLUMN IF NOT EXISTS pds_cid TEXT,
    ADD COLUMN IF NOT EXISTS pds_uri TEXT,
    ADD COLUMN IF NOT EXISTS sync_status sync_status NOT NULL DEFAULT 'local_only',
    ADD COLUMN IF NOT EXISTS last_synced_at TIMESTAMPTZ;

ALTER TABLE notes
    ADD COLUMN IF NOT EXISTS version INTEGER NOT NULL DEFAULT 1,
    ADD COLUMN IF NOT EXISTS pds_cid TEXT,
    ADD COLUMN IF NOT EXISTS pds_uri TEXT,
    ADD COLUMN IF NOT EXISTS sync_status sync_status NOT NULL DEFAULT 'local_only',
    ADD COLUMN IF NOT EXISTS last_synced_at TIMESTAMPTZ;

CREATE TABLE IF NOT EXISTS sync_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    owner_did TEXT NOT NULL,
    entity_type TEXT NOT NULL, -- 'deck', 'card', 'note'
    entity_id UUID NOT NULL,
    operation TEXT NOT NULL, -- 'push', 'pull', 'conflict_resolve'
    status TEXT NOT NULL, -- 'pending', 'success', 'failed'
    pds_cid TEXT,
    error_message TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_sync_log_owner_did ON sync_log(owner_did);
CREATE INDEX IF NOT EXISTS idx_sync_log_entity ON sync_log(entity_type, entity_id);
CREATE INDEX IF NOT EXISTS idx_sync_log_status ON sync_log(status);
CREATE INDEX IF NOT EXISTS idx_sync_log_created_at ON sync_log(created_at DESC);

CREATE INDEX IF NOT EXISTS idx_decks_sync_status ON decks(sync_status) WHERE sync_status != 'synced';
CREATE INDEX IF NOT EXISTS idx_cards_sync_status ON cards(sync_status) WHERE sync_status != 'synced';
CREATE INDEX IF NOT EXISTS idx_notes_sync_status ON notes(sync_status) WHERE sync_status != 'synced';

CREATE OR REPLACE FUNCTION increment_version_on_update()
RETURNS TRIGGER AS $$
BEGIN
    -- Only increment if content changed (not just sync metadata)
    IF (TG_TABLE_NAME = 'decks' AND (NEW.title != OLD.title OR NEW.description != OLD.description OR NEW.tags != OLD.tags)) OR
       (TG_TABLE_NAME = 'cards' AND (NEW.front != OLD.front OR NEW.back != OLD.back OR NEW.media_url IS DISTINCT FROM OLD.media_url)) OR
       (TG_TABLE_NAME = 'notes' AND (NEW.title != OLD.title OR NEW.body != OLD.body OR NEW.tags != OLD.tags)) THEN
        NEW.version = OLD.version + 1;
        -- Mark as pending push if it was synced
        IF OLD.sync_status = 'synced' THEN
            NEW.sync_status = 'pending_push';
        END IF;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER increment_decks_version BEFORE UPDATE ON decks
    FOR EACH ROW EXECUTE FUNCTION increment_version_on_update();

CREATE TRIGGER increment_cards_version BEFORE UPDATE ON cards
    FOR EACH ROW EXECUTE FUNCTION increment_version_on_update();

CREATE TRIGGER increment_notes_version BEFORE UPDATE ON notes
    FOR EACH ROW EXECUTE FUNCTION increment_version_on_update();
