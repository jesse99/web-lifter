use crate::*;

/// Global state passed into axum handlers.
pub struct AppState {
    pub engine: Handlebars<'static>,
    pub history: History,
    pub program: Program,
}

/// [`State`] is shared across threaded handlers so we need to protect access.
pub type SharedState = Arc<RwLock<AppState>>;
