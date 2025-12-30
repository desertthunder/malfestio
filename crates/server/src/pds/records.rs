//! Record serialization for AT Protocol Lexicons.
//!
//! Converts internal models to AT Protocol record format.

use chrono::Utc;
use malfestio_core::at_uri::AtUri;
use malfestio_core::model::{Card, Deck, Note, Visibility};
use malfestio_core::tid::generate_tid;
use serde::Serialize;
use serde_json::Value;

/// A deck record in Lexicon format.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeckRecord {
    #[serde(rename = "$type")]
    pub record_type: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub tags: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub card_refs: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub source_refs: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
    pub created_at: String,
}

/// A card record in Lexicon format.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CardRecord {
    #[serde(rename = "$type")]
    pub record_type: String,
    pub deck_ref: String,
    pub front: String,
    pub back: String,
    pub card_type: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub hints: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media: Option<CardMedia>,
    pub created_at: String,
}

/// Media attachment for a card.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CardMedia {
    pub image_ref: Option<String>,
    pub audio_ref: Option<String>,
}

/// A note record in Lexicon format.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteRecord {
    #[serde(rename = "$type")]
    pub record_type: String,
    pub title: String,
    pub body: String,
    pub tags: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub links: Vec<String>,
    pub visibility: String,
    pub created_at: String,
}

/// Result of preparing a record for publishing.
pub struct PreparedRecord {
    /// The record key (TID)
    pub rkey: String,
    /// The NSID collection
    pub collection: String,
    /// The serialized record
    pub record: Value,
}

impl DeckRecord {
    /// Create a DeckRecord from an internal Deck model.
    pub fn from_deck(deck: &Deck, card_at_uris: Vec<String>) -> Self {
        Self {
            record_type: "app.malfestio.deck".to_string(),
            title: deck.title.clone(),
            description: if deck.description.is_empty() { None } else { Some(deck.description.clone()) },
            tags: deck.tags.clone(),
            card_refs: card_at_uris,
            source_refs: vec![],
            license: None,
            created_at: Utc::now().to_rfc3339(),
        }
    }
}

impl CardRecord {
    /// Create a CardRecord from an internal Card model.
    pub fn from_card(card: &Card, deck_at_uri: &str) -> Self {
        Self {
            record_type: "app.malfestio.card".to_string(),
            deck_ref: deck_at_uri.to_string(),
            front: card.front.clone(),
            back: card.back.clone(),
            card_type: "basic".to_string(),
            hints: vec![],
            media: card
                .media_url
                .as_ref()
                .map(|url| CardMedia { image_ref: Some(url.clone()), audio_ref: None }),
            created_at: Utc::now().to_rfc3339(),
        }
    }
}

impl NoteRecord {
    /// Create a NoteRecord from an internal Note model.
    pub fn from_note(note: &Note) -> Self {
        Self {
            record_type: "app.malfestio.note".to_string(),
            title: note.title.clone(),
            body: note.body.clone(),
            tags: note.tags.clone(),
            links: note.links.clone(),
            visibility: visibility_to_string(&note.visibility),
            created_at: Utc::now().to_rfc3339(),
        }
    }
}

/// Convert visibility enum to string for Lexicon.
fn visibility_to_string(visibility: &Visibility) -> String {
    match visibility {
        Visibility::Private => "private".to_string(),
        Visibility::Unlisted => "unlisted".to_string(),
        Visibility::Public => "public".to_string(),
        Visibility::SharedWith(_) => "shared".to_string(),
    }
}

/// Prepare a deck for publishing to PDS.
pub fn prepare_deck_record(deck: &Deck, card_at_uris: Vec<String>) -> PreparedRecord {
    let record = DeckRecord::from_deck(deck, card_at_uris);
    PreparedRecord {
        rkey: generate_tid(),
        collection: "app.malfestio.deck".to_string(),
        record: serde_json::to_value(record).expect("Failed to serialize deck record"),
    }
}

/// Prepare a card for publishing to PDS.
pub fn prepare_card_record(card: &Card, deck_at_uri: &str) -> PreparedRecord {
    let record = CardRecord::from_card(card, deck_at_uri);
    PreparedRecord {
        rkey: generate_tid(),
        collection: "app.malfestio.card".to_string(),
        record: serde_json::to_value(record).expect("Failed to serialize card record"),
    }
}

/// Prepare a note for publishing to PDS.
pub fn prepare_note_record(note: &Note) -> PreparedRecord {
    let record = NoteRecord::from_note(note);
    PreparedRecord {
        rkey: generate_tid(),
        collection: "app.malfestio.note".to_string(),
        record: serde_json::to_value(record).expect("Failed to serialize note record"),
    }
}

/// Generate an AT-URI for a record.
pub fn make_at_uri(did: &str, collection: &str, rkey: &str) -> AtUri {
    AtUri::new(did, collection, rkey)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_deck() -> Deck {
        Deck {
            id: "deck-123".to_string(),
            owner_did: "did:plc:abc123".to_string(),
            title: "Test Deck".to_string(),
            description: "A test deck".to_string(),
            tags: vec!["test".to_string(), "sample".to_string()],
            visibility: Visibility::Public,
            published_at: None,
            fork_of: None,
        }
    }

    fn sample_card() -> Card {
        Card {
            id: "card-123".to_string(),
            owner_did: "did:plc:abc123".to_string(),
            deck_id: "deck-123".to_string(),
            front: "What is the capital of France?".to_string(),
            back: "Paris".to_string(),
            media_url: None,
            card_type: malfestio_core::model::CardType::default(),
            hints: vec![],
        }
    }

    fn sample_note() -> Note {
        Note {
            id: "note-123".to_string(),
            owner_did: "did:plc:abc123".to_string(),
            title: "Test Note".to_string(),
            body: "This is a test note with **markdown**.".to_string(),
            tags: vec!["notes".to_string()],
            visibility: Visibility::Public,
            published_at: None,
            links: vec![],
        }
    }

    #[test]
    fn test_deck_record_from_deck() {
        let deck = sample_deck();
        let record = DeckRecord::from_deck(&deck, vec![]);

        assert_eq!(record.record_type, "app.malfestio.deck");
        assert_eq!(record.title, "Test Deck");
        assert_eq!(record.description, Some("A test deck".to_string()));
        assert_eq!(record.tags.len(), 2);
    }

    #[test]
    fn test_deck_record_serialization() {
        let deck = sample_deck();
        let record = DeckRecord::from_deck(&deck, vec!["at://did:plc:abc/app.malfestio.card/tid1".to_string()]);

        let json = serde_json::to_string(&record).unwrap();
        assert!(json.contains("\"$type\":\"app.malfestio.deck\""));
        assert!(json.contains("\"title\":\"Test Deck\""));
        assert!(json.contains("cardRefs"));
    }

    #[test]
    fn test_card_record_from_card() {
        let card = sample_card();
        let deck_uri = "at://did:plc:abc123/app.malfestio.deck/tid123";
        let record = CardRecord::from_card(&card, deck_uri);

        assert_eq!(record.record_type, "app.malfestio.card");
        assert_eq!(record.deck_ref, deck_uri);
        assert_eq!(record.front, "What is the capital of France?");
        assert_eq!(record.back, "Paris");
    }

    #[test]
    fn test_note_record_from_note() {
        let note = sample_note();
        let record = NoteRecord::from_note(&note);

        assert_eq!(record.record_type, "app.malfestio.note");
        assert_eq!(record.title, "Test Note");
        assert_eq!(record.visibility, "public");
    }

    #[test]
    fn test_prepare_deck_record() {
        let deck = sample_deck();
        let prepared = prepare_deck_record(&deck, vec![]);

        assert_eq!(prepared.collection, "app.malfestio.deck");
        assert_eq!(prepared.rkey.len(), 13); // TID length
        assert!(prepared.record.is_object());
    }

    #[test]
    fn test_prepare_card_record() {
        let card = sample_card();
        let prepared = prepare_card_record(&card, "at://did:plc:abc/app.malfestio.deck/tid");

        assert_eq!(prepared.collection, "app.malfestio.card");
        assert_eq!(prepared.rkey.len(), 13);
    }

    #[test]
    fn test_prepare_note_record() {
        let note = sample_note();
        let prepared = prepare_note_record(&note);

        assert_eq!(prepared.collection, "app.malfestio.note");
        assert_eq!(prepared.rkey.len(), 13);
    }

    #[test]
    fn test_make_at_uri() {
        let uri = make_at_uri("did:plc:abc123", "app.malfestio.deck", "3k5abc123");
        assert_eq!(uri.to_string(), "at://did:plc:abc123/app.malfestio.deck/3k5abc123");
    }

    #[test]
    fn test_visibility_to_string() {
        assert_eq!(visibility_to_string(&Visibility::Private), "private");
        assert_eq!(visibility_to_string(&Visibility::Public), "public");
        assert_eq!(visibility_to_string(&Visibility::Unlisted), "unlisted");
        assert_eq!(visibility_to_string(&Visibility::SharedWith(vec![])), "shared");
    }
}
