# Lexicon Schemas

This directory contains the Lexicon definitions for the malfestio's public records.

## Protocol + Lexicon Strategy

- "Artifacts" are publishable records (ATProto Lexicon).
- "Learning state" is private (local DB + your backend sync; not public records).
- Records are distributed and hard to migrate globally; keep mutable/private state out.
- Lexicon evolution rules strongly encourage forward-compatible extensibility.

### Namespace + NSID conventions

- `org.stormlightlabs.malfestio.note`
- `org.stormlightlabs.malfestio.card`
- `org.stormlightlabs.malfestio.deck`
- `org.stormlightlabs.malfestio.source.article`
- `org.stormlightlabs.malfestio.source.lecture`
- `org.stormlightlabs.malfestio.collection`
- `org.stormlightlabs.malfestio.thread.comment`

### Lexicon basics

- Lexicon defines record types + XRPC endpoints; JSON-schema-like constraints.
- Use "optional fields" heavily; avoid enums that will calcify the product too early.
- Versioning: add fields, don't rename; never rely on being able to rewrite history.

### Schema boundaries (important)

- **Public share layer**:
    - decks, cards, notes, collections, comments
- **Private layer**:
    - review schedule, lapses, grades, per-card performance, streaks

## Evolution Rules

1. **Additive Changes Only**: You can add new optional fields to existing records.
2. **No Renaming**: Do not rename fields.
   If a semantic change is needed, add a new field and deprecate the old one.
3. **No Type Changes**: Once published, a field's type is fixed.
4. **Version by Copying**: If a breaking change is absolutely required, create a new Lexicon with a new major version or a new name (e.g., `org.stormlightlabs.malfestio.noteV2`).
