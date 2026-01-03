//! Content extraction using XPath rules and generic algorithms

pub mod generic;
pub mod scoring;
pub mod xpath;

pub use generic::{ExtractedContent, GenericExtractor};
pub use scoring::{
    ContentScore, calculate_class_weight, calculate_link_density, is_unlikely_candidate, is_viable_candidate,
};
pub use xpath::XPathExtractor;
