use crate::*;

/// State associated with a user.
pub struct UserState {
    pub notes: Notes,
    pub history: History,
    pub weights: Weights,
    pub program: Program,
}

/// Global state passed into axum handlers.
pub struct AppState {
    pub handlebars: Handlebars<'static>, // templating engine
    pub user: UserState,
}

/// [`AppState`] is shared across threaded handlers so we need to protect access.
pub type SharedState = Arc<RwLock<AppState>>;
