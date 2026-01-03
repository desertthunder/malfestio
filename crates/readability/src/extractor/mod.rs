//! Content extraction using XPath rules and generic algorithms

pub mod generic;
pub mod scoring;
pub mod xpath;

pub use generic::GenericExtractor;
pub use xpath::XPathExtractor;
