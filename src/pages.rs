use crate::app_state::SharedState;
use crate::errors::Error;
use axum::http::Uri;

mod editor_builder;
mod editors;
mod views;

pub use editors::*;
pub use views::*;

pub fn post_epilog(state: SharedState, path: &str) -> Result<Uri, Error> {
    {
        let state = &mut state.write().unwrap();
        let name = state.name.clone();
        let user = &mut state.user;
        if let Err(e) = crate::persist::save(&name, user) {
            user.errors.push(format!("{e}")); // not fatal so we don't return an error
        }
    }

    let uri = url_escape::encode_path(path);
    let uri = uri.parse()?;
    Ok(uri)
}
