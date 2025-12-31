-- Migration: Extend search_items view to include indexed remote records

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
    visibility,
    'local' AS source
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
    d.visibility,
    'local' AS source
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
    visibility,
    'local' AS source
FROM notes
UNION ALL
-- Indexed remote decks
SELECT
    'deck' AS item_type,
    id::text AS item_id,
    did AS creator_did,
    setweight(to_tsvector('english', unaccent(coalesce(title, ''))), 'A') ||
    setweight(to_tsvector('english', unaccent(coalesce(description, ''))), 'B') AS tsv_content,
    jsonb_build_object(
        'id', id,
        'title', title,
        'description', description,
        'owner_did', did,
        'at_uri', at_uri,
        'tags', tags
    ) AS data,
    jsonb_build_object('type', 'Public') AS visibility,
    'remote' AS source
FROM indexed_decks
WHERE deleted_at IS NULL
UNION ALL
-- Indexed remote cards
SELECT
    'card' AS item_type,
    id::text AS item_id,
    did AS creator_did,
    setweight(to_tsvector('english', unaccent(coalesce(front, ''))), 'A') ||
    setweight(to_tsvector('english', unaccent(coalesce(back, ''))), 'B') AS tsv_content,
    jsonb_build_object(
        'id', id,
        'deck_ref', deck_ref,
        'front', front,
        'back', back,
        'owner_did', did,
        'at_uri', at_uri
    ) AS data,
    jsonb_build_object('type', 'Public') AS visibility,
    'remote' AS source
FROM indexed_cards
WHERE deleted_at IS NULL
UNION ALL
-- Indexed remote notes
SELECT
    'note' AS item_type,
    id::text AS item_id,
    did AS creator_did,
    setweight(to_tsvector('english', unaccent(coalesce(title, ''))), 'A') ||
    setweight(to_tsvector('english', unaccent(coalesce(body, ''))), 'B') AS tsv_content,
    jsonb_build_object(
        'id', id,
        'title', title,
        'owner_did', did,
        'at_uri', at_uri
    ) AS data,
    jsonb_build_object('type', visibility) AS visibility,
    'remote' AS source
FROM indexed_notes
WHERE deleted_at IS NULL;

CREATE UNIQUE INDEX idx_search_items_unique ON search_items (item_type, item_id, source);
CREATE INDEX idx_search_items_tsv ON search_items USING GIN (tsv_content);
CREATE INDEX idx_search_items_meta ON search_items (item_type, creator_did);
CREATE INDEX idx_search_items_visibility ON search_items USING GIN (visibility);
CREATE INDEX idx_search_items_source ON search_items (source);

CREATE OR REPLACE FUNCTION refresh_search_items()
RETURNS void AS $$
BEGIN
    REFRESH MATERIALIZED VIEW CONCURRENTLY search_items;
END;
$$ LANGUAGE plpgsql;
