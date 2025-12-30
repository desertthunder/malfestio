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

## Record Publishing

### XRPC Endpoints

- `com.atproto.repo.putRecord` — Create or update records
- `com.atproto.repo.deleteRecord` — Remove records
- `com.atproto.repo.uploadBlob` — Upload media attachments

### Record Keys

Use TID (timestamp-based identifiers) per Lexicon spec.

### AT-URIs

Format: `at://<did>/<collection>/<rkey>`

Example: `at://did:plc:abc123/app.malfestio.deck/3k5abc123`

## Firehose / Jetstream

### Raw Firehose

- **WebSocket**: Subscribe to `com.atproto.sync.subscribeRepos` from a Relay
- **CBOR Decoding**: Parse incoming events
- **Cursor Management**: Track position for reconnection

### Jetstream (Recommended)

Bluesky's simplified JSON firehose:

- JSON format (no CBOR decoding)
- Reduced bandwidth (zstd compression)
- Collection/repo filtering at source
- Simpler reconnection with cursors

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
