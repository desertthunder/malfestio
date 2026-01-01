# Local Development

## Prerequisites

### Required Tools

- Rust (latest stable)
- Node.js 18+ and pnpm
- PostgreSQL 14+
- Docker (optional, for containerized Postgres)

### Bluesky Account Setup

1. Create a Bluesky account at <https://bsky.app>
2. Generate an App Password (Settings → App Passwords)
3. Configure `.env` with your credentials:

```bash
APP_USERNAME=your-handle.bsky.social
APP_PASSWORD=your-app-password-here
DB_URL="postgres://postgres:postgres@localhost:5432/malfestio_dev?sslmode=disable"
```

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

## Environment Variables

### Required

```bash
APP_USERNAME=your-handle.bsky.social
APP_PASSWORD=your-app-password
DB_URL="postgres://postgres:postgres@localhost:5432/malfestio_dev?sslmode=disable"
```

### Optional

```bash
# Server configuration
SERVER_HOST=127.0.0.1
SERVER_PORT=8080

# Frontend proxy
VITE_API_URL=http://localhost:8080

# Logging
RUST_LOG=info,malfestio_server=debug
```

## Additional Resources

- [AT Protocol OAuth Guide](https://docs.bsky.app/blog/oauth-atproto)
- [OAuth Client Implementation](https://docs.bsky.app/docs/advanced-guides/oauth-client)
- [PDS Self-Hosting](https://atproto.com/guides/self-hosting)
- [AT Protocol Specifications](https://atproto.com)
