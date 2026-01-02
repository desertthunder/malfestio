# AT Protocol Research Notes

Reference material for AT Protocol integration. For implementation details, see [todo.md](todo.md).

## OAuth 2.1 Specification

AT Protocol uses a specific profile of OAuth 2.1 for client↔PDS authorization.

### Required Components

- **Client Metadata Endpoint**: Serve `client_metadata.json` at a public HTTPS URL (this URL becomes the `client_id`)

  ```json
  {
    "client_id": "https://your-app.com/oauth/client-metadata.json",
    "application_type": "web",
    "grant_types": ["authorization_code", "refresh_token"],
    "scope": "atproto transition:generic",
    "response_types": ["code"],
    "redirect_uris": ["https://your-app.com/oauth/callback"],
    "client_name": "Malfestio",
    "client_uri": "https://your-app.com"
  }
  ```

- **PKCE (Mandatory)**: Generate `code_verifier` and `code_challenge` (S256 only)
- **DPoP (Mandatory)**: Bind tokens to client instances with proof-of-possession JWTs
- **Handle/DID Resolution**: Resolve user identity to discover their PDS
- **Token Exchange**: Authorization code flow with token refresh

### DPoP (Demonstrating Proof-of-Possession)

DPoP (RFC 9449) binds access tokens to specific client instances, preventing token theft/replay.

**Proof JWT Structure:**

- **Header**: `typ: dpop+jwt`, `alg: EdDSA` (or ES256), `jwk: <public key>`
- **Payload Claims**:
    - `jti` — Unique identifier (nonce) per request
    - `htm` — HTTP method (e.g., "POST", "GET")
    - `htu` — HTTP target URI (without query/fragment)
    - `iat` — Issued-at timestamp
    - `ath` — SHA-256 hash of access token (for resource requests)
    - `nonce` — Server-provided nonce (if required)

**Usage:**

1. Client generates DPoP keypair per session (not reused across devices/users)
2. Each request includes `Authorization: DPoP <token>` and `DPoP: <proof JWT>`
3. Server validates signature, checks claims match request, verifies token binding

**Server Behavior:**

- May return `DPoP-Nonce` header; client must include in subsequent proofs
- Validates `jti` uniqueness to prevent replay attacks
- Checks `ath` matches provided access token

## Record Publishing

### XRPC Endpoints

- `com.atproto.repo.putRecord` — Create or update records
- `com.atproto.repo.deleteRecord` — Remove records
- `com.atproto.repo.uploadBlob` — Upload media attachments

### Record Keys

Use TID (timestamp-based identifiers) per Lexicon spec.

### AT-URIs

Format: `at://<did>/<collection>/<rkey>`

Example: `at://did:plc:abc123/org.stormlightlabs.malfestio.deck/3k5abc123`

## Firehose / Jetstream

### Overview

The AT Protocol provides two main options for consuming real-time repository events:

1. **Raw Firehose** (`com.atproto.sync.subscribeRepos`) - Full-fidelity, CBOR-encoded, cryptographically signed
2. **Jetstream** - Simplified JSON format, lower bandwidth, easier to consume

### Raw Firehose

- **WebSocket**: Subscribe to `com.atproto.sync.subscribeRepos` from a Relay
- **CBOR Decoding**: Parse CAR files containing MST blocks
- **Cryptographic Verification**: Validate commit signatures against DID signing keys
- **Cursor Management**: Track `seq` position for reliable reconnection

**Event Types:**

- `#commit` - Repository changes (record create/update/delete)
- `#identity` - DID/handle updates
- `#account` - Account status changes (active, deactivated, etc.)

### Jetstream (Simplified)

Bluesky's simplified JSON firehose - ideal for indexing and discovery:

- **JSON format**: No CBOR decoding required
- **zstd compression**: Reduced bandwidth (enable with `compress=true`)
- **Collection filtering**: Subscribe to specific NSIDs
- **DID filtering**: Watch specific accounts
- **Cursor-based reconnection**: Microsecond timestamps

**Public Endpoints:**

- `wss://jetstream1.us-east.bsky.network/subscribe`
- `wss://jetstream2.us-west.bsky.network/subscribe`

**Tradeoffs:**

- ⚠️ Events are NOT cryptographically signed (trust the Jetstream operator)
- ⚠️ Not self-authenticating data
- ✅ Much simpler to implement
- ✅ Lower bandwidth and compute requirements

### Reliable Synchronization

**Cursor Tracking:**

- Store cursor position (microsecond timestamp) per endpoint
- Resume from last processed cursor on reconnect
- Handle gaps by fetching missing commits via `getRepo` if needed

**Per-Repo Revision Tracking:**

- Track latest `rev` (TID) for each DID
- Compare incoming `rev` against stored value to detect gaps
- Use `since` field to detect out-of-order events

**Deletion Handling:**

- Handle `operation: "delete"` in commit events
- Mark records as deleted (soft or hard delete)

**Best Practices:**

- Process events sequentially per-DID (partition by DID)
- Ignore events with `rev` ≤ stored latest rev
- Validate records against Lexicon schema before indexing

## Well-Known Endpoints

- `/.well-known/atproto-did` — Domain verification for handle claims
- `/.well-known/oauth-protected-resource` — PDS OAuth metadata
- `/.well-known/oauth-authorization-server` — Auth server metadata

## Labelers

**Architecture:**

1. Labels = metadata (source DID + subject AT-URI + value string)
2. User Subscription = users subscribe to labelers; clients include in API requests
3. Label Interpretation = per-user config to hide, warn, or ignore content

**Structure:**

```json
{
  "src": "did:plc:labeler",
  "uri": "at://did:user/app.bsky.feed.post/123",
  "val": "spam",
  "cts": "2026-01-01T00:00:00Z"
}
```

## Feeds

**Core Flow**:

1. User requests feed via at-uri of declared feed
2. PDS resolves at-uri → Feed Generator's DID doc
3. PDS sends `getFeedSkeleton` to service endpoint (authenticated by user's JWT)
4. Feed Generator returns skeleton (list of post URIs + cursor)
5. PDS hydrates skeleton with full content (via AppView)
6. Hydrated feed returned to user

## AppView

**Responsibilities**:

1. Record Processing & Indexing - consume firehose, build indices for likes, threads, follows
2. Moderation Enforcement - apply labels from subscribed labelers
3. Query Interface - expose XRPC API (proxied through PDS)
4. Media CDN - fetch/cache blobs from upstream PDSes, generate thumbnails
5. Search & Discovery - full-text search, type-ahead, content ranking

## Patterns from Real AT Protocol Apps

### plyr.fm (Music)

- OAuth 2.1 via `@atproto/oauth-client` library
- Records synced to PDS: tracks, likes, playlists
- Separate moderation service (Rust labeler)

### leaflet.pub (Writing)

- React/Next.js frontend with Supabase + Replicache for sync
- Bluesky integration via dedicated `lexicons/` and `appview/` directories

### wisp.place (Static Sites)

- Stores site files as `place.wisp.fs` records in user's PDS
- Firehose consumer to index and serve sites
- CDN layer caches content from PDS

### Common Patterns

1. Local database for fast queries + PDS for portable, signed records
2. Firehose consumption for discovery/aggregation
3. OAuth 2.1 for production auth (app passwords only for development)
4. Lexicons define the public contract; internal state stays private

## References

- [AT Protocol OAuth Spec](https://atproto.com/specs/oauth)
- [Lexicon Schema Language](https://atproto.com/specs/lexicon)
- [Repository & XRPC](https://atproto.com/specs/xrpc)
- [Feed Generator Starter Kit](https://github.com/bluesky-social/feed-generator)
- [atproto TypeScript SDK](https://github.com/bluesky-social/atproto)
- [Ozone Moderation Service](https://github.com/bluesky-social/ozone)
- [Jetstream Firehose](https://docs.bsky.app/blog/jetstream)
- [Labels and Moderation Guide](https://docs.bsky.app/docs/advanced-guides/moderation)
