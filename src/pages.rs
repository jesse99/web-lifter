use crate::app_state::SharedState;
use crate::errors::Error;
use axum::http::Uri;

mod editor_builder;
mod editors;
mod exercise_page;
mod exercise_post;
mod overview_page;
mod program_page;
mod workout_page;

pub use editors::*;
pub use exercise_page::*;
pub use exercise_post::*;
pub use overview_page::*;
pub use program_page::*;
pub use workout_page::*;

pub fn post_epilog(state: SharedState, path: &str) -> Result<Uri, Error> {
    {
        let user = &mut state.write().unwrap().user;
        if let Err(e) = crate::persist::save(user) {
            user.errors.push(format!("{e}")); // not fatal so we don't return an error
        }
    }

    let uri = url_escape::encode_path(path);
    let uri = uri.parse()?;
    Ok(uri)
}
