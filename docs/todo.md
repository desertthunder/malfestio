# Product + Technical Roadmap

## Protocol + Lexicon Strategy

- "Artifacts" are publishable records (ATProto Lexicon).
- "Learning state" is private (local DB + your backend sync; not public records).
- Records are distributed and hard to migrate globally; keep mutable/private state out.
- Lexicon evolution rules strongly encourage forward-compatible extensibility.

### Namespace + NSID conventions

- `app.malfestio.note`
- `app.malfestio.card`
- `app.malfestio.deck`
- `app.malfestio.source.article`
- `app.malfestio.source.lecture`
- `app.malfestio.collection`
- `app.malfestio.thread.comment`

### Lexicon basics

- Lexicon defines record types + XRPC endpoints; JSON-schema-like constraints.
- Use "optional fields" heavily; avoid enums that will calcify the product too early.
- Versioning: add fields, don’t rename; never rely on being able to rewrite history.

### Schema boundaries (important)

- **Public share layer**:
    - decks, cards, notes, collections, comments
- **Private layer**:
    - review schedule, lapses, grades, per-card performance, streaks

### Auth direction

- ATProto is moving toward OAuth for client↔PDS authorization.
- Plan for OAuth support even if MVP starts centralized.

## Roadmap Milestones

- **(Done) Milestone A**: Defined core user journeys, information architecture, and privacy rules for the platform.

- **(Done) Milestone B**: Designed AT Protocol Lexicons for all core types and documented data model mapping + publishing pipeline.

### Milestone C - Foundations: Repo, CI, Axum API Skeleton, Solid Shell

#### Deliverables

- Monorepo layout (crates/server/, web/, lexicons/, crates/cli/, crates/core/)
    - core: domain logic, data models
    - server: axum, postgres, search, eventing, auth
    - cli: entry point for local dev, PDS sync
    - web: solidjs app
- CI: format/lint/test + schema validation + typegen
- Axum:
    - health, auth stub, error model, request IDs, structured logging
- Solid:
    - tailwind, router shell, auth gate, initial pages + layout system

#### Acceptance

- End-to-end "hello world" for create/read deck locally.

### Milestone D - Identity + Permissions + Publishing Model

#### Deliverables

- Auth MVP:
    - BlueSky App Passwords
    - ATProto OAuth
- Permission model:
    - private / unlisted / public / shared-with
- Publishing:
    - draft editing, publish, update, deprecate, fork

#### Acceptance

- A user can publish a deck and another user can view it.

### Milestone E - Content Authoring (Notes + Cards + Deck Builder)

#### Deliverables

- Note editor (markdown + attachments + backlinks)
- Card editor:
    - basic front/back + cloze v1
    - images/audio attachments (optional)
- Deck builder:
    - tags, ordering, sections
- Importers v1:
    - article URL -> extracted snapshot + highlights
    - lecture URL -> outline + timestamps (manual entry initially)

#### Acceptance

- A creator can build a deck from an article and publish it.

### Milestone F - Study Engine (SRS) + Daily Review UX

#### Deliverables

- SRS scheduler v1 (SM-2 baseline)
    - grade 0–5, EF, interval, repetition count
- Review queue generation rules
- Study session UI:
    - keyboard-first review loop
    - quick edit card during review
- Progress views (private):
    - due count, retention proxy, streaks

#### Acceptance

- 30-day simulated study test produces stable, believable intervals.

#### Notes

- SM-2 reference behavior is well documented; start there and iterate.

### Milestone G - Social Layer v1 (Follow, Feed, Fork, Comments)

#### Deliverables

- Follow graph + notifications
- Feeds:
    - "New decks from follows"
    - "Trending this week" (simple scoring)
- Forking workflow:
    - fork deck -> edit -> republish
- Threaded comments on decks/cards

#### Acceptance

- A user can follow a curator and see new published decks in a feed.

### Milestone H - Search + Discovery + Taxonomy

#### Deliverables

- Full-text search over:
    - deck title/description, card text, note text, source metadata
- Tag taxonomy:
    - user tags + curator tags + system tags
- Discovery pages:
    - top tags, featured paths, editor picks

#### Acceptance

- Search is fast (<200ms typical) and results feel relevant.

### Milestone I - Moderation + Abuse Resistance

#### Deliverables

- Reporting pipeline + review queue
- Rate limits + spam heuristics
- Takedown/visibility states (shadowed, removed, quarantined)
- Audit logging for moderation actions

#### Acceptance

- You can safely operate an open publishing surface.

### Milestone J - Federation / ATProto Integration Pass

#### Deliverables

- Phase 1 (minimum):
    - export Lexicon records
    - ingest remote records into a read-only "federated library"
- Phase 2:
    - OAuth login to PDS + publish records directly (client or server mediated)
    - reconcile local drafts with remote published state

#### Acceptance

- A published artifact is portable beyond your app.

#### Notes

- ATProto OAuth is the forward path; plan on it.
- XRPC endpoint patterns and legacy session behavior exist, but treat them as transitional.

### Milestone K - Reliability, Observability, Launch

#### Deliverables

- Metrics + tracing + structured logs
- Backups + restore drills
- Load test targets (study session + feed + search)
- Beta program + feedback loop + roadmap iteration

#### Acceptance

- You can run this as a real product with confidence.

## "First Cut" Lexicon Fields (Draft)

### Note (app.malfestio.note)

- title: string
- body: richtext/markdown string
- tags: string[]
- links: { uri, title?, type? }[]
- createdAt, updatedAt
- visibility: "private|unlisted|public" (consider leaving as string + documented values)

### Card (app.malfestio.card)

- deckRef: at-uri / stable ref
- front: string (markdown)
- back: string (markdown)
- cardType: "basic|cloze" (optional)
- hints?: string[]
- media?: { kind, uri, alt? }[]

### Deck (app.malfestio.deck)

- title, description
- tags
- cardRefs: at-uri[]
- sourceRefs: at-uri[] (articles/lectures)
- license?: string (strongly recommended)

### Article (app.malfestio.source.article)

- url
- title
- author?
- publishedAt?
- extractedTextRef? (only if you store it)
- highlights?: { quote, start?, end? }[]

### Lecture (app.malfestio.source.lecture)

- url
- title
- creator?
- timestamps?: { t, label, noteRef? }[]

### Collection/Path (app.malfestio.collection)

- title, description
- items: { type, ref, note? }[]
- tags

### Comment (app.malfestio.thread.comment)

- subjectRef (deck/card/note ref)
- body
- replyTo?

(Keep everything extensible; avoid hard commitments early.)

## Open Questions (Parked Decisions)

- Local-first mechanics: full offline authoring + later publish, or online-only creation?
- Federation depth: read-only ingest first, or publish-to-PDS in the first public beta?
- Content extraction: store extracted article snapshots (legal/ops implications), or store only metadata + highlights?
