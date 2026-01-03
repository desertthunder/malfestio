# Local Development

## Prerequisites

### Required Tools

- Rust (latest stable)
- Node.js 18+ and pnpm
- PostgreSQL 14+
- Docker (optional, for containerized Postgres)

### Bluesky Account Setup

1. Create a Bluesky account at <https://bsky.app> (you'll use this for OAuth testing)

### Environment Configuration

Copy the template and configure for your environment:

```bash
cp .env.example .env
```

For local development, the defaults in `.env.example` work out of the box. You only need to ensure your PostgreSQL connection string is correct.

## Testing OAuth Flow

### Step-by-Step

1. **Start PostgreSQL**

    ```bash
    # Using Docker
    docker run -d -p 5432:5432 -e POSTGRES_PASSWORD=postgres postgres:14

    # Or use your local PostgreSQL installation
    ```

2. **Run migrations**

    ```bash
    just migrate
    ```

3. **Start backend**

    ```bash
    just start
    ```

    Server runs on <http://localhost:8080>

4. **Start frontend**

    ```bash
    just web-dev
    ```

    Frontend runs on <http://localhost:3000>

5. **Test OAuth login**
    - Navigate to <http://localhost:3000/login>
    - Enter your Bluesky handle (e.g., `thunderbot.bsky.social`)
    - Authorize the application on bsky.social
    - Verify redirect back to app with successful login

### OAuth Flow Details

When you enter a handle like `thunderbot.bsky.social`, the system:

1. **Handle Resolution**: DNS TXT lookup at `_atproto.thunderbot.bsky.social` or HTTP `https://thunderbot.bsky.social/.well-known/atproto-did`
2. **DID Resolution**: Resolved DID (e.g., `did:plc:...`) queries `https://plc.directory` for PDS endpoint
3. **OAuth Discovery**: `https://bsky.social/.well-known/oauth-authorization-server` fetched for endpoints
4. **Authorization**: User redirected to PDS authorization page with PKCE challenge
5. **Token Exchange**: Authorization code exchanged for access/refresh tokens with DPoP binding
6. **Storage**: Tokens stored in database with encrypted DPoP keypair

## Testing Record Publishing

After successful OAuth login:

1. Create a deck or note in the UI
2. Click "Publish" to publish to your PDS
3. Check your Bluesky profile at <https://bsky.app> to see the published record
4. Verify record appears in your AT Protocol repository

## Testing Sync Flow

The app supports offline-first editing with automatic sync when online.

### Local Storage (IndexedDB)

All decks, notes, and cards are stored locally in IndexedDB via Dexie.js. You can view this data:

1. Navigate to Settings page
2. Scroll to "Local Sync Data" section
3. Use the Records/Queue tabs to view stored data

### Testing Offline Mode

1. Create or edit a deck while online → syncs immediately
2. Disconnect network (DevTools → Network → Offline)
3. Create/edit content → stored locally with "local_only" or "pending_push" status
4. Reconnect → content auto-syncs when online status changes

### Sync Statuses

| Status         | Meaning                          |
| -------------- | -------------------------------- |
| `local_only`   | New content, never synced        |
| `synced`       | Content matches PDS              |
| `pending_push` | Local changes waiting to sync    |
| `conflict`     | Local and remote versions differ |

### Conflict Resolution

When conflicts occur:

1. Settings → Local Sync Data shows records with "conflict" status
2. Click "Keep Local" to overwrite remote with local version
3. Or use the API: `POST /api/sync/resolve/:type/:id` with strategy

## Verifying Your Setup

### Check OAuth Tokens

After successful login, verify tokens were stored:

```sql
SELECT
  did,
  pds_url,
  LEFT(access_token, 20) || '...' as token_preview,
  created_at,
  updated_at
FROM oauth_tokens
WHERE did = 'your-did-here';
```

Replace `'your-did-here'` with the DID from your login success page.

### Check Indexed Records

After publishing content, verify firehose indexing:

```sql
-- Check indexed decks
SELECT at_uri, title, indexed_at
FROM indexed_decks
WHERE did = 'your-did-here'
ORDER BY indexed_at DESC
LIMIT 10;

-- Check indexed cards
SELECT at_uri, front_content, indexed_at
FROM indexed_cards
WHERE did = 'your-did-here'
ORDER BY indexed_at DESC
LIMIT 10;
```

Note: Indexing may take 5-10 seconds after publishing.

### Diagnostic Command

Run this command to check handle resolution and database state:

```bash
just verify your-handle.bsky.social
```

This will verify:

- Database connection
- Handle → DID resolution
- DID → PDS URL resolution
- OAuth token status
- Indexed content count

## Environment Variables Reference

### Required

```bash
DB_URL="postgres://postgres:postgres@localhost:5432/malfestio_dev?sslmode=disable"
```

### Optional

```bash
# OAuth Client Configuration
APP_URL=http://localhost:3000        # OAuth callback URL
APP_NAME=Malfestio                   # App display name

# Server Configuration
SERVER_HOST=127.0.0.1
SERVER_PORT=8080

# Frontend Configuration
VITE_API_URL=http://localhost:8080

# Logging
RUST_LOG=info,malfestio_server=debug
```

See `.env.example` for a complete template.

## Additional Resources

- [AT Protocol OAuth Guide](https://docs.bsky.app/blog/oauth-atproto)
- [OAuth Client Implementation](https://docs.bsky.app/docs/advanced-guides/oauth-client)
- [PDS Self-Hosting](https://atproto.com/guides/self-hosting)
- [AT Protocol Specifications](https://atproto.com)
