# Product + Technical Roadmap

## Auth direction

- ATProto is moving toward OAuth for client↔PDS authorization.
- Plan for OAuth support even if MVP starts centralized.

## Roadmap Milestones

> [!NOTE]
> Completed milestones (A-K) have been moved to [CHANGELOG.md](/CHANGELOG.md).

### Milestone L - ATProto Integration Pass

#### Deliverables

**Identity & Auth:**

- [x] OAuth login directly to user's PDS
- [x] Handle resolution via DNS TXT or `/.well-known/atproto-did`
- [x] DPoP token binding for secure API calls

**Local Development:**

- [x] Document local testing with real Bluesky accounts
- [x] Add justfile commands for common dev tasks
- [x] Environment variable configuration guide
- [x] Update health check endpoint for service monitoring
- [x] Add logging for OAuth flow steps

**Sync & Conflict Resolution:**

- [x] Bi-directional sync infrastructure
- [x] Conflict resolution strategy
- [x] API endpoints for sync operations
- [ ] Offline queue for pending publishes
    - [ ] Frontend sync store with IndexedDB persistence
- [ ] Sync status UI indicators

**Deep Linking:**

- [ ] AT-URI deep linking from external clients
- [ ] Handle `at://` URL scheme in app
- [ ] Link preview generation for shared content

#### Acceptance

- User can log in with existing Bluesky/PDS identity
- OAuth flow works with production bsky.social accounts
- Developers can test locally using real accounts (see [Local Development Guide](./local-dev.md))
- Local drafts sync correctly after reconnecting

### Milestone M - Reliability, Observability, Launch (v0.1.0)

#### Deliverables

**Observability:**

- [ ] Structured logging with correlation IDs
- [ ] Metrics collection (Tracing spans)
- [ ] Error tracking

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

### Milestone N - Custom Feed Generator (v0.2.0)

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

- [ ] Subscribe to `com.atproto.sync.subscribeRepos` (or Jetstream) for `org.stormlightlabs.malfestio.*` records
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

### Milestone O - Moderation + Abuse Resistance (v0.3.0)

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

### Milestone P - Readability Updates

#### Deliverables

**Multi-page Support:**

- [ ] `single_page_link` directive - find "view full article" link
- [ ] `next_page_link` directive - paginate through article pages
- [ ] Concatenate content from multiple pages
- [ ] Avoid circular pagination

**Advanced Directives:**

- [ ] `http_header(name)` directive - custom headers for fetching
- [ ] `replace_string(find): replace` directive - text replacement
- [ ] `find_string` directive - text pattern matching

**Quality:**

- [ ] Better table handling in markdown
- [ ] Image caption extraction
- [ ] JSON-LD support

**Performance:**

- [ ] LRU cache for parsed configs
- [ ] Parallel candidate scoring
- [ ] Lazy XPath evaluation

**Markdown Conversion:**

- [ ] Custom markdown converter (more control than html2md)
- [ ] Code block language detection

#### Acceptance

- [ ] Can correctly extract multi-page articles (e.g., long news reports).
- [ ] Advanced string manipulation allows for cleaner output on tricky sites.
- [ ] Performance remains stable under high load.

## Open Question/Parked Decisions

- Full offline authoring + later publish
- Federation depth: publish-to-PDS in the first public beta
- Content extraction: store extracted article snapshots locally (browser)
    - Persist only metadata + highlights
