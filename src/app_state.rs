use crate::{history::History, notes::Notes, program::Program, weights::Weights};
use handlebars::Handlebars;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};

/// State associated with a user.
#[derive(Debug, Serialize, Deserialize)]
pub struct UserState {
    pub notes: Notes,
    pub history: History,
    pub weights: Weights,
    pub program: Program,
    pub errors: Vec<String>,
}

/// Global state passed into axum handlers.
pub struct AppState {
    pub handlebars: Handlebars<'static>, // templating engine
    pub user: UserState,
}

/// [`AppState`] is shared across threaded handlers so we need to protect access.
pub type SharedState = Arc<RwLock<AppState>>;
