# Share vs Private Rules

This document defines the strict separation between public "Artifacts" and private "Learning State".
This distinction is critical for the application's architecture and user privacy.

## The Separation Principle

- **Public Layer ("Artifacts")**: Content that is created to be shared.
  These are immutable (or append-only) records published to the AT Protocol.
- **Private Layer ("Learning State")**: Personal study data.
  This includes grades, schedules, and progress.
  This data **NEVER** leaks into the public record automatically.

## Public Record Types (The Lexicon)

These entities are visible to anyone with access to the PDS (essentially public).

- Deck (`org.stormlightlabs.malfestio.deck`): The collection of cards/notes.
- Card (`org.stormlightlabs.malfestio.card`): The flashcard content (Front/Back).
- Note (`org.stormlightlabs.malfestio.note`): The source knowledge note.
- Article (`org.stormlightlabs.malfestio.source.article`): Metadata/snapshot of an external article.
- Lecture (`org.stormlightlabs.malfestio.source.lecture`): Metadata/outline of an external video/audio.
- Collection (`org.stormlightlabs.malfestio.collection`): Curated lists of decks.
- Comment (`org.stormlightlabs.malfestio.thread.comment`): Public discussion.

> **Rule**: If a user puts sensitive information in a Card, they must be warned that
> publishing the Deck makes it public.

## Private Data Types (Local/Private Sync)

These entities are stored in the user's private database (Local -> Private User Sync).

- **Scheduling Data**:
    - Next due date
    - Ease Factor (EF)
    - Interval history
    - Retrievability estimates
- **Performance Metrics**:
    - Grades (Forget, Hard, Good, Easy)
    - Session duration
    - Response times
- **Drafts**:
    - Decks/Cards currently being written but not yet published.

## Interaction Rules

1. **Forking**: When a user forks a public deck, they create a *new* public record (referencing the original). Their *private* study data for the original deck does NOT transfer to the new fork (or is reset).
2. **Study Session**: A study session is a purely private event. It reads Public Artifacts but writes only to the Private Layer.
3. **Sync**: Public records sync via ATProto (Relay/PDS). Private data syncs via a separate, private channel (e.g., encrypted backup or private PDS blob).
