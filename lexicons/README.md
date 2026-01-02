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

## Publishing Lexicons to AT Protocol Network

### Prerequisites

**Goat CLI**: Install the official AT Protocol CLI tool

```bash
# macOS
brew install goat
```

### Publishing Workflow

1. **Validate schemas locally**:

   ```bash
   goat lexicon lint lexicons/
   ```

2. **Check DNS configuration**:

   ```bash
   goat lexicon check-dns org.stormlightlabs.malfestio.card
   ```

3. **Publish to network**:

   ```bash
   goat lexicon publish lexicons/org/stormlightlabs/malfestio/
   ```

### PDS Validation Modes

AT Protocol PDSs support three lexicon validation modes:

1. **Explicit validation required**: Record must validate against schema; fails if PDS doesn't know the lexicon
   - This is the current mode causing `Lexicon not found` errors
   - Requires publishing lexicons or using optimistic validation

2. **Optimistic validation** (default): Validates if PDS knows the schema, allows creation if unknown
   - Most flexible for custom lexicons during development
   - Set via `validate: undefined` in create/update record calls

3. **Explicit no validation**: Skips validation even if PDS knows the schema
   - Set via `validate: false` in create/update record calls

### Version Updates

When updating lexicon schemas:

1. **Minor Updates** (additive only):
   - Add new optional fields
   - Update descriptions
   - Add new `knownValues` (don't remove old ones)
   - Document version with timestamp (use `date +%s` to get Unix timestamp)
   - Note in changelog: `// Updated: 1735862400 (2026-01-02)`

2. **Breaking Changes** (avoid if possible):
   - Create new lexicon with new NSID (e.g., `org.stormlightlabs.malfestio.cardV2`)
   - Maintain both versions during migration period
   - Update code to support both old and new schemas
   - Document migration path

3. **Republishing**:

   ```bash
   goat lexicon lint lexicons/
   goat lexicon publish lexicons/org/stormlightlabs/malfestio/
   ```

## Evolution Rules

1. **Additive Changes Only**: You can add new optional fields to existing records.
2. **No Renaming**: Do not rename fields.
   If a semantic change is needed, add a new field and deprecate the old one.
3. **No Type Changes**: Once published, a field's type is fixed.
4. **Version by Copying**: If a breaking change is absolutely required, create a new Lexicon with a new major version or a new name (e.g., `org.stormlightlabs.malfestio.noteV2`).
