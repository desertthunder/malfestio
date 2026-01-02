//! OAuth 2.1 implementation for AT Protocol.
//!
//! This module provides the OAuth 2.1 client flow components required
//! for AT Protocol authentication:
//!
//! - PKCE (Proof Key for Code Exchange)
//! - DPoP (Demonstrating Proof of Possession)
//! - Handle/DID resolution
//! - Token management

pub mod client_metadata;
pub mod dpop;
pub mod flow;
pub mod pkce;
pub mod resolver;

pub use client_metadata::client_metadata_handler;
