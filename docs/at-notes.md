# AT Protocol Research Notes

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

## Firehose Consumption

For social features (trending, discovery, feeds):

- **WebSocket Connection**: Subscribe to `com.atproto.sync.subscribeRepos` from a Relay
- **CBOR Decoding**: Parse incoming events (or use Jetstream for JSON)
- **Cursor Management**: Track position for reconnection

## AppView Pattern

Index network-wide records to power discovery features:

- Index `app.malfestio.*` records from firehose
- Implement `getFeedSkeleton` for custom algorithmic feeds
- Hydration service combines skeletons with full content from PDSes

## Well-Known Endpoints

- `/.well-known/atproto-did` — Domain verification for handle claims
- `/.well-known/oauth-protected-resource` — PDS OAuth metadata
- `/.well-known/oauth-authorization-server` — Auth server metadata

## Patterns from Real AT Protocol Apps

### plyr.fm (Music)

- OAuth 2.1 via `@atproto/oauth-client` library
- Records synced to PDS: tracks, likes, playlists
- Separate moderation service (Rust labeler)
- Data ownership: "tracks, likes, playlists synced to your PDS as ATProto records"

### leaflet.pub (Writing)

- React/Next.js frontend with Supabase + Replicache for sync
- Bluesky integration via dedicated `lexicons/` and `appview/` directories
- Publications posted to Bluesky

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
