# AGENTS.md

This file is the canonical source of truth for AI agents working on this project. It also serves as
a reference for contributors to the project.

## Project Overview

Malfestio is a learning OS combining flashcards, notes, lectures, and articles for daily study built on top of the
AT Protocol. It implements a local-first approach with social features for publishing, sharing, and remixing learning artifacts.

## Development Commands

### Rust Backend

```bash
# Build the workspace
cargo build

# Run the server via CLI
cargo run --bin malfestio-cli start

# Run tests
cargo test

# Run tests for specific crate
cargo test -p malfestio-server
cargo test -p malfestio-core

# Check without building
cargo check

# Run clippy lints
cargo clippy
```

### Frontend (SolidJS)

```bash
# Install dependencies
cd web && pnpm install

# Run development server
pnpm dev

# Build for production
pnpm build

# Run tests
pnpm test

# Type check without building
pnpm check

# Lint
pnpm lint
```

## Project Structure

```sh
# tree
.
├── crates
│   ├── core
│   ├── server
│   └── cli
└── web
```

## Rules & Workflows

- *todo*
