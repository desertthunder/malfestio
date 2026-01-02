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

# Test handle and DID resolution for a Bluesky account
verify HANDLE:
    cargo run --bin malfestio-cli check {{HANDLE}}

# Clean build artifacts
clean:
    cargo clean
    cd web && rm -rf dist node_modules/.vite

push:
    git push origin main && git push alpha main
