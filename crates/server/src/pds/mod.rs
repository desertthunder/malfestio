//! PDS (Personal Data Server) client for AT Protocol.
//!
//! Provides record publishing operations:
//! - putRecord - Create or update records
//! - deleteRecord - Remove records
//! - uploadBlob - Upload media attachments

pub mod client;
pub mod publish;
pub mod records;
