use crate::db::DbPool;
use std::sync::Arc;

pub type SharedState = Arc<AppState>;

pub struct AppState {
    pub pool: DbPool,
}

impl AppState {
    pub fn new(pool: DbPool) -> SharedState {
        Arc::new(Self { pool })
    }
}
