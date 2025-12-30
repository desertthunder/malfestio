# System Architecture

This document describes the technical architecture of Malfestio.

## Frontend (SolidJS)

- App shell + router-driven workspaces (Library / Study / Create / Social).
- Signals as primary state primitive; keep study session state in signals/store.

## Backend (Rust)

- Axum API gateway: REST/XRPC-ish endpoints, tower middleware, typed extractors.
- Services (logical, not necessarily microservices):
    - Identity/Auth service (local + optional ATProto OAuth integration)
    - Content service (notes/cards/decks/sources)
    - Study service (queue generation + grading + scheduling)
    - Social service (follows, feeds, comments, notifications)
    - Search service (indexing + query)
    - Moderation service (reports, takedowns, rules)

## Storage

- Postgres: canonical app DB (users, private study state, cache of published records).
- Object storage: images/audio, extracted article snapshots (if you store them).
- Search index: separate system (Meilisearch/Typesense/ZincSearch-pick one later).

## Eventing

- Internal outbox pattern (DB table) for:
    - reindex jobs, notification fanout, federation publish steps

## Data Model

See [Data Model Mapping](./data-model-mapping.md) for the mapping between public Lexicon records and internal database tables.

### Core Entities

- **Note**: markdown + structure + citations + links to sources.
- **Card**: front/back (+ optional cloze, audio, image, code block).
- **Deck**: ordered/clustered cards (+ metadata, tags).
- **Lecture**: external URL + outline + timestamps + linked notes/cards.
- **Article**: URL + extracted text (readability style heuristics) + highlights + linked notes/cards.
- **Collection/Path**: curated bundle of decks + notes + sources.
