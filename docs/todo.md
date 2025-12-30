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
- **(Done) Milestone C**: Foundations: Repo, CI, Axum API Skeleton, Solid Shell.
    - Monorepo layout, CI, Axum/Solid skeletons implemented.
    - Backend running on 8080, Frontend on 3000.
- **(Done) Milestone D**: Identity + Permissions + Publishing Model.
    - Auth MVP, Permission model (Private/Public/SharedWith), and basic Publishing flow implemented.
    - Backend API and Frontend Editor updated with tests covering permissions and publishing.
- **(Done) Milestone F**: OAuth + PDS Record Publishing.
    - OAuth 2.1 client flow (PKCE, DPoP, handle/DID resolution, token refresh).
    - PDS client for `putRecord`, `deleteRecord`, `uploadBlob`.
    - TID generation and AT-URI builder in core crate.
- **(Done) Milestone E**: Internal component library/UI Foundation + Animations.
- **(Done) Milestone F**: Content Authoring (Notes + Cards + Deck Builder).
- **(Done) Milestone G**: Study Engine (SRS) + Daily Review UX.
    - SM-2 spaced repetition scheduler.
- **(Done) Milestone H**: Social Layer v1: Follow graph, Feeds (Follows/Trending), Forking workflow, and Threaded comments.
- **(Done) Milestone I**: Search + Discovery + Taxonomy.
    - Full-text search with pg_trgm/unaccent, visibility filtering, and unified search index.
    - Tag taxonomy and Discovery page with top tags.

### Milestone J - Moderation + Abuse Resistance

#### Deliverables

- Look into [Ozone](https://github.com/bluesky-social/ozone)
- Reporting pipeline + review queue
- Rate limits + spam heuristics
- Takedown/visibility states (shadowed, removed, quarantined)
- Audit logging for moderation actions

#### Acceptance

- You can safely operate an open publishing surface.

### Milestone K - Federation / ATProto Integration Pass

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

### Milestone L - Reliability, Observability, Launch

#### Deliverables

- Metrics + tracing + structured logs
- Backups + restore drills
- Load test targets (study session + feed + search)
- Beta program + feedback loop + roadmap iteration

## Open Questions (Parked Decisions)

- Local-first mechanics: full offline authoring + later publish, or online-only creation?
- Federation depth: read-only ingest first, or publish-to-PDS in the first public beta?
- Content extraction: store extracted article snapshots (legal/ops implications), or store only metadata + highlights?
