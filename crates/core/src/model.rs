use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub id: String,
    pub owner_did: String,
    pub title: String,
    pub body: String,
    pub tags: Vec<String>,
    pub visibility: Visibility,
    pub published_at: Option<String>,
    #[serde(default)]
    pub links: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Card {
    pub id: String,
    pub owner_did: String,
    pub deck_id: String,
    pub front: String,
    pub back: String,
    pub media_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", content = "content")]
pub enum Visibility {
    Private,
    Unlisted,
    Public,
    SharedWith(Vec<String>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deck {
    pub id: String,
    pub owner_did: String,
    pub title: String,
    pub description: String,
    pub tags: Vec<String>,
    pub visibility: Visibility,
    pub published_at: Option<String>,
    pub fork_of: Option<String>,
}
