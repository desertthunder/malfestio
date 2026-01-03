//! Configuration file parsing and loading for site-specific extraction rules

pub mod embedded_rules;
pub mod loader;
pub mod parser;

pub use loader::ConfigLoader;
pub use parser::{SiteConfig, parse_config};
