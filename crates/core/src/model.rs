use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub title: String,
    pub body: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Card {
    pub front: String,
    pub back: String,
    pub deck_ref: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deck {
    pub title: String,
    pub description: String,
    pub tags: Vec<String>,
}
