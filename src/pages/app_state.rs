use crate::*;

/// Global state passed into axum handlers.
pub struct AppState {
    pub major_version: i32, // file version (just in case we need it later)
    pub minor_version: i32,
    pub engine: Handlebars<'static>,
    pub notes: Notes,
    pub history: History,
    pub weights: Weights,
    pub program: Program,
}

/// [`State`] is shared across threaded handlers so we need to protect access.
pub type SharedState = Arc<RwLock<AppState>>;
