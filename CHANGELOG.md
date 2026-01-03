# Changelog

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## Added

### 2026-01-03

- OAuth login directly to user's PDS with DPoP token binding
    - Handle resolution via DNS TXT or `/.well-known/atproto-did`
- Bi-directional sync infrastructure with conflict resolution
- Implemented `malfestio-readability` crate
    - Custom, rule-based content extraction engine replacing `dom_smoothie`, featuring XPath support (ftr-site-config compatible) and a Mozilla Readability-based generic fallback.

### 2026-01-02

- Published AT Protocol Lexicons for all core types (`org.stormlightlabs.malfestio.*`)

### 2025-12-*

- *TODO*
