-- Firehose cursor and indexed records for AT Protocol Jetstream consumption
-- Tracks cursor position for reconnection and indexes discovered records

-- Table for tracking Jetstream cursor position
CREATE TABLE firehose_cursors (
    id SERIAL PRIMARY KEY,
    endpoint TEXT NOT NULL UNIQUE,
    cursor_us BIGINT NOT NULL,  -- Unix microseconds timestamp
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_firehose_cursors_endpoint ON firehose_cursors(endpoint);

CREATE TABLE indexed_records (
    id SERIAL PRIMARY KEY,
    at_uri TEXT NOT NULL UNIQUE,
    did TEXT NOT NULL,
    collection TEXT NOT NULL,
    rkey TEXT NOT NULL,
    indexed_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_indexed_records_did ON indexed_records(did);
CREATE INDEX idx_indexed_records_collection ON indexed_records(collection);
CREATE INDEX idx_indexed_records_at_uri ON indexed_records(at_uri);
