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
- Versioning: add fields, don't rename; never rely on being able to rewrite history.

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
- **(Done) Milestone E**: Internal component library/UI Foundation + Animations.
- **(Done) Milestone F**: OAuth + PDS Record Publishing.
    - OAuth 2.1 client flow (PKCE, DPoP, handle/DID resolution, token refresh).
    - PDS client for `putRecord`, `deleteRecord`, `uploadBlob`.
    - TID generation and AT-URI builder in core crate.
- **(Done) Milestone G**: Content Authoring (Notes + Cards + Deck Builder).
- **(Done) Milestone H**: Study Engine (SRS) + Daily Review UX.
    - SM-2 spaced repetition scheduler.
- **(Done) Milestone I**: Social Layer v1: Follow graph, Feeds (Follows/Trending), Forking workflow, and Threaded comments.
    - Full-text search with pg_trgm/unaccent, visibility filtering, and unified search index.
    - Tag taxonomy and Discovery page with top tags.

### Milestone J - Static Content & Landing Page

#### Deliverables

**Marketing/Static Site:**

- [ ] Landing page with hero section, feature highlights, social proof
    - Hero should have a graph paper/grid background
    - Floating flash cards, notes
- [ ] "How it works" section with study flow visualization
- [ ] About page with team/mission

**App Vision Content:**

- [ ] Onboarding flow with persona selection (Learner/Creator/Curator)
- [ ] Empty states with helpful prompts for new users
- [ ] Tutorial/walkthrough for first deck creation
- [ ] Help center or FAQ section -> Should mention that the app is still in development and subject to change.

**SEO & Meta:**

- [ ] Open Graph / Twitter Card meta tags
    - [ ] Scripted with HTML2Canvas/PNG generation
        - Graph paper background
        - Floating flash cards, notes
- [ ] Sitemap.xml generation
- [ ] robots.txt configuration

#### Acceptance

- New visitors understand the value proposition within 10 seconds.
- Onboarding flow guides users to create their first deck.

### Milestone K - AppView Indexing

#### Deliverables

**Firehose Enhancement:**

- [ ] Upgrade firehose consumer to store full record content (not just metadata)
- [ ] Add `indexed_decks`, `indexed_cards`, `indexed_notes` tables for remote content
- [ ] Track latest processed revision per repo; handle deletions

**Search & Discovery:**

- [ ] Implement search over indexed remote records (extend `search_items` view)
- [ ] User profile aggregation: follower counts, deck counts from federated sources

**Export & Interop:**

- [ ] Export local records as valid Lexicon JSON (`/api/export/:collection`)
- [ ] Read-only "federated library" view showing remote decks

#### Acceptance

- A deck published from Malfestio can be discovered via another AT Protocol client.
- Remote decks from followed users appear in search results.

### Milestone L - ATProto Integration Pass

#### Deliverables

**Identity & Auth:**

- [ ] OAuth login directly to user's PDS (vs. local-only auth)
- [ ] Handle resolution via DNS TXT or `/.well-known/atproto-did`
- [ ] DPoP token binding for secure API calls

**Sync & Conflict Resolution:**

- [ ] Bi-directional sync: local drafts → PDS records, PDS records → local cache
- [ ] Conflict resolution strategy for concurrent edits (last-write-wins or merge UI)
- [ ] Offline queue for pending publishes

**Deep Linking:**

- [ ] AT-URI deep linking from external clients
- [ ] Handle `at://` URL scheme in app

#### Acceptance

- User can log in with their existing Bluesky/PDS identity.
- Local drafts sync correctly after reconnecting.

#### Implementation Details

**Considerations:**

- Scalability: substantial compute; caching, DB optimization, distributed processing
- Lexicon Validation: validate schemas, ignore invalid records gracefully
- Account State: track latest processed revision per repo; handle deletions
- Bluesky's AppView uses PostgreSQL or ScyllaDB + image proxy + AppView core

**Identity:**

- Use `did:web` for simplicity, `did:plc` for long-term stability
- ATProto OAuth is the forward path

### Milestone M - Custom Feed Generator

#### Deliverables

**Infrastructure:**

- [ ] Feed Generator service with `did:web` identity
- [ ] Publish `app.bsky.feed.generator` declaration record to creator's repo
- [ ] DID document with service endpoint for feed requests

**Endpoints:**

- [ ] `app.bsky.feed.getFeedSkeleton` - Return post URIs + cursor for pagination
- [ ] `app.bsky.feed.describeFeedGenerator` - Feed metadata (DID, name, description)
- [ ] JWT authentication for user-personalized feeds

**Algorithms:**

- [ ] "Trending Decks" - Top decks by fork/like count in last 7 days
- [ ] "New from Following" - Latest decks from followed creators
- [ ] "Study Streak Leaders" - Decks with highest completion rates (anonymized)

**Indexing:**

- [ ] Subscribe to `com.atproto.sync.subscribeRepos` (or Jetstream) for `app.malfestio.*` records
- [ ] Index posts with compound cursor (timestamp::CID) for deterministic pagination
- [ ] Garbage collect indexed data older than 48 hours (except pinned content)

#### Acceptance

- Custom feed appears in Bluesky and other AT Protocol clients.
- Feed surfaces relevant learning content based on engagement signals.
- Pagination works correctly across feed refreshes.

#### Implementation Details

**Core Flow:** see [AT Notes](./at-notes.md#feeds)

**Skeleton Response Format:**

```json
{
  "feed": [
    {"post": "at://did:example/app.bsky.feed.post/1"},
    {"post": "at://did:example/app.bsky.feed.post/2"}
  ],
  "cursor": "1683654690921::bafyrei..."
}
```

**Skeleton Metadata Types:**

```typescript
type SkeletonItem = {
  post: string        // post URI
  reason?: Reason     // optional context (e.g., repost)
}
type ReasonRepost = {
  $type: 'app.bsky.feed.defs#skeletonReasonRepost'
  repost: string      // repost URI
}
```

**Considerations:**

- Validate user JWTs if feed depends on user state (follows, likes)
- Use compound cursor (timestamp::CID) for deterministic pagination
- Most feeds can garbage collect data older than 48 hours
- Reference: [Feed Generator Starter Kit](https://github.com/bluesky-social/feed-generator)

### Milestone N - Reliability, Observability, Launch

#### Deliverables

**Observability:**

- [ ] Structured logging with correlation IDs
- [ ] Metrics collection (Prometheus/OpenTelemetry)
- [ ] Distributed tracing for request flows
- [ ] Error tracking (Sentry or similar)

**Reliability:**

- [ ] Database backups + restore drills
- [ ] Health check endpoints (`/health`, `/ready`)
- [ ] Graceful shutdown handling
- [ ] Circuit breakers for external dependencies

**Load Testing:**

- [ ] Study session throughput targets
- [ ] Feed generation latency benchmarks
- [ ] Search query performance under load

**Launch Prep:**

- [ ] Beta program signup flow
- [ ] Feedback collection mechanism
- [ ] Feature flags for gradual rollout

#### Acceptance

- System handles 10x expected load without degradation.
- Mean time to recovery < 5 minutes for common failures.

### Milestone O - Moderation + Abuse Resistance

#### Deliverables

**Labeler Infrastructure:**

- [ ] Dedicated Bluesky account for labeler service
- [ ] Publish `app.bsky.labeler.service` record to make discoverable
- [ ] Self-host Ozone backend + UI (Docker setup in `HOSTING.md`)
- [ ] Configure report types via `goat` CLI

**Endpoints:**

- [ ] `com.atproto.label.subscribeLabels` - real-time label stream
- [ ] `com.atproto.label.queryLabels` - query published labels
- [ ] `com.atproto.report.createReport` - accept user reports

**Moderation Features:**

- [ ] Reporting pipeline + review queue UI
- [ ] Rate limits + spam heuristics
- [ ] Takedown/visibility states (shadowed, removed, quarantined)
- [ ] Audit logging for moderation actions

#### Acceptance

- You can safely operate an open publishing surface.
- Users can subscribe to your labeler and see moderation applied.

**Reference:** [Ozone Moderation Service](https://github.com/bluesky-social/ozone)

## Open Question/Parked Decisions

- Full offline authoring + later publish
- Federation depth: publish-to-PDS in the first public beta
- Content extraction: store extracted article snapshots locally (browser)
    - Persist only metadata + highlights
