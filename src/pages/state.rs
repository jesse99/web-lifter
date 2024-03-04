use crate::*;

/// Global state passed into axum handlers.
pub struct State {
    pub engine: Handlebars<'static>,
    pub program: Program,
    pub exercises: Exercises,
}

/// [`State`] is shared across threaded handlers so we need to protect access.
pub type SharedState = Arc<RwLock<State>>;
