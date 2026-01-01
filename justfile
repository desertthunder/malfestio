# Malfestio

# Build all Rust crates
build:
    cargo build

# Build for release
build-release:
    cargo build --release

# Run the server via CLI
start:
    cargo run --bin malfestio-cli start

# Run all tests
test:
    cargo test --quiet

# Check code without building
check:
    cargo check

# Run clippy lints
lint:
    cargo clippy --fix --allow-dirty

# Format code
fmt:
    cargo fmt

# Install frontend dependencies
web-install:
    cd web && pnpm install

# Run development server
web-dev:
    cd web && pnpm dev

# Build frontend for production
web-build:
    cd web && pnpm build

# Run frontend tests
web-test:
    cd web && pnpm test

# Type check frontend
web-check:
    cd web && pnpm check

# Lint frontend
web-lint:
    cd web && pnpm lint

# Start both backend and frontend (in separate terminals recommended)
dev:
    @echo "Start backend: just start"
    @echo "Start frontend: just web-dev"

# Run all tests (backend + frontend)
test-all: test web-test

# Run database migrations
migrate:
    cargo run --bin malfestio-cli migrate

# Setup and test OAuth flow with real Bluesky account
test-oauth:
    @echo "Testing OAuth with Bluesky account..."
    @echo "1. Ensure PostgreSQL is running"
    @echo "2. Running migrations..."
    @just migrate
    @echo "3. Start backend with: just start"
    @echo "4. Start frontend with: just web-dev"
    @echo "5. Navigate to http://localhost:3000/login"
    @echo "6. Enter your Bluesky handle from .env"

# Clean build artifacts
clean:
    cargo clean
    cd web && rm -rf dist node_modules/.vite
