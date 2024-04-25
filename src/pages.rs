use axum::http::Uri;

mod app_state;
mod edit_add_exercise_page;
mod edit_any_weight;
mod edit_current_set;
mod edit_durations;
mod edit_durs_record;
mod edit_exercises;
mod edit_fixed_reps_page;
mod edit_formal_name_page;
mod edit_name_page;
mod edit_note_page;
mod edit_reps_record_page;
mod edit_rest_page;
mod edit_var_reps_page;
mod edit_var_sets_page;
mod edit_weight_page;
mod editor_builder;
mod errors;
mod exercise_page;
mod exercise_post;
mod overview_page;
mod program_page;
mod workout_page;

pub use app_state::*;
pub use edit_add_exercise_page::*;
pub use edit_any_weight::*;
pub use edit_current_set::*;
pub use edit_durations::*;
pub use edit_durs_record::*;
pub use edit_exercises::*;
pub use edit_fixed_reps_page::*;
pub use edit_formal_name_page::*;
pub use edit_name_page::*;
pub use edit_note_page::*;
pub use edit_reps_record_page::*;
pub use edit_rest_page::*;
pub use edit_var_reps_page::*;
pub use edit_var_sets_page::*;
pub use edit_weight_page::*;
pub use editor_builder::*;
pub use errors::*;
pub use exercise_page::*;
pub use exercise_post::*;
pub use overview_page::*;
pub use program_page::*;
pub use workout_page::*;

pub fn post_epilog(state: SharedState, path: &str) -> Result<Uri, anyhow::Error> {
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
