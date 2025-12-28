# Data Model Mapping

This document maps the public AT Protocol Lexicon records to our internal SQL database schema.

## Principles

1. **Separation of Concerns**
   We store "my view" of the world (private state) separate from "the network's view" (public records).
2. **Hydration**
   We allow hydrating public records into local DB rows for efficient query/indexing, but the source of truth for the record itself is the signed commit in the repository.
3. **Private State**
   User progress, scheduling, and local-only drafts live ONLY in the internal DB.

## Mapping Table

| Public Lexicon                 | Internal DB Table(s) | Notes                                                          |
| :----------------------------- | :------------------- | :------------------------------------------------------------- |
| `app.malfestio.deck`           | `decks`              | Public metadata.                                               |
| `app.malfestio.card`           | `cards`              | Content.                                                       |
| `app.malfestio.note`           | `notes`              | Standalone notes.                                              |
| `app.malfestio.source.*`       | `sources`            | Metadata for articles/lectures.                                |
| `app.malfestio.collection`     | `collections`        |                                                                |
| `app.malfestio.thread.comment` | `comments`           |                                                                |
| _(None)_                       | `reviews`            | **Private**. Logs of every review event.                       |
| _(None)_                       | `study_progress`     | **Private**. Current SRS state for a card (box/interval/ease). |
| _(None)_                       | `user_settings`      | **Private**. Daily goals, UI references.                       |
| _(None)_                       | `drafts`             | **Private**. Content being authored before publishing.         |

## Sync Strategy

- **Publishing**:
  Write to `drafts` -> User clicks "Publish" -> Sign record -> Push to PDS -> Move `drafts` content to `decks`/`cards` tables (or mark as synced).
- **Consuming**:
  Pull from PDS (firehose or direct sync) -> Validate signature -> Upsert into local tables.
