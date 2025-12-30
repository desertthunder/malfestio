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

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum CardType {
    #[default]
    Basic,
    Cloze,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Card {
    pub id: String,
    pub owner_did: String,
    pub deck_id: String,
    pub front: String,
    pub back: String,
    pub media_url: Option<String>,
    #[serde(default)]
    pub card_type: CardType,
    #[serde(default)]
    pub hints: Vec<String>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comment {
    pub id: String,
    pub deck_id: String,
    pub author_did: String,
    pub content: String,
    pub parent_id: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Follow {
    pub follower_did: String,
    pub subject_did: String,
    pub created_at: String,
}
