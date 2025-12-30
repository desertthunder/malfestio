CREATE EXTENSION IF NOT EXISTS pg_trgm;
CREATE EXTENSION IF NOT EXISTS unaccent;

DROP MATERIALIZED VIEW IF EXISTS search_items;

CREATE MATERIALIZED VIEW search_items AS
SELECT
    'deck' AS item_type,
    id::text AS item_id,
    owner_did AS creator_did,
    setweight(to_tsvector('english', unaccent(coalesce(title, ''))), 'A') ||
    setweight(to_tsvector('english', unaccent(coalesce(description, ''))), 'B') AS tsv_content,
    jsonb_build_object(
        'id', id,
        'title', title,
        'description', description,
        'owner_did', owner_did
    ) AS data,
    visibility
FROM decks
UNION ALL
SELECT
    'card' AS item_type,
    c.id::text AS item_id,
    c.owner_did AS creator_did,
    setweight(to_tsvector('english', unaccent(coalesce(c.front, ''))), 'A') ||
    setweight(to_tsvector('english', unaccent(coalesce(c.back, ''))), 'B') AS tsv_content,
    jsonb_build_object(
        'id', c.id,
        'deck_id', c.deck_id,
        'front', c.front,
        'back', c.back,
        'owner_did', c.owner_did
    ) AS data,
    d.visibility
FROM cards c
JOIN decks d ON c.deck_id = d.id
UNION ALL
SELECT
    'note' AS item_type,
    id::text AS item_id,
    owner_did AS creator_did,
    setweight(to_tsvector('english', unaccent(coalesce(title, ''))), 'A') ||
    setweight(to_tsvector('english', unaccent(coalesce(body, ''))), 'B') AS tsv_content,
    jsonb_build_object(
        'id', id,
        'title', title,
        'owner_did', owner_did
    ) AS data,
    visibility
FROM notes;

CREATE UNIQUE INDEX idx_search_items_unique ON search_items (item_type, item_id);

CREATE INDEX idx_search_items_tsv ON search_items USING GIN (tsv_content);

CREATE INDEX idx_search_items_meta ON search_items (item_type, creator_did);

CREATE INDEX idx_search_items_visibility ON search_items USING GIN (visibility);

CREATE OR REPLACE FUNCTION refresh_search_items()
RETURNS void AS $$
BEGIN
    REFRESH MATERIALIZED VIEW CONCURRENTLY search_items;
END;
$$ LANGUAGE plpgsql;
