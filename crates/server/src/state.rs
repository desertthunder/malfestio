use malfestio_core::model::{Card, Deck, Note};
use std::sync::{Arc, RwLock};

pub type SharedState = Arc<AppState>;

pub struct AppState {
    pub decks: RwLock<Vec<Deck>>,
    pub notes: RwLock<Vec<Note>>,
    pub cards: RwLock<Vec<Card>>,
}

impl AppState {
    pub fn new() -> SharedState {
        Arc::new(Self {
            decks: RwLock::new(Vec::new()),
            notes: RwLock::new(Vec::new()),
            cards: RwLock::new(Vec::new()),
        })
    }
}
