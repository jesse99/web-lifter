use axum::http::Uri;

mod app_state;
mod edit_add_exercise;
mod edit_any_weight;
mod edit_block;
mod edit_blocks;
mod edit_current_set;
mod edit_discrete_set;
mod edit_durations;
mod edit_durs_record;
mod edit_exercises;
mod edit_fixed_reps;
mod edit_formal_name;
mod edit_name;
mod edit_note;
mod edit_plate_set;
mod edit_reps_record;
mod edit_rest;
mod edit_schedule;
mod edit_set_week;
mod edit_var_reps;
mod edit_var_sets;
mod edit_weight;
mod edit_weight_sets;
mod edit_workouts;
mod editor_builder;
mod errors;
mod exercise_page;
mod exercise_post;
mod overview_page;
mod program_page;
mod workout_page;

pub use app_state::*;
pub use edit_add_exercise::*;
pub use edit_any_weight::*;
pub use edit_block::*;
pub use edit_blocks::*;
pub use edit_current_set::*;
pub use edit_discrete_set::*;
pub use edit_durations::*;
pub use edit_durs_record::*;
pub use edit_exercises::*;
pub use edit_fixed_reps::*;
pub use edit_formal_name::*;
pub use edit_name::*;
pub use edit_note::*;
pub use edit_plate_set::*;
pub use edit_reps_record::*;
pub use edit_rest::*;
pub use edit_schedule::*;
pub use edit_set_week::*;
pub use edit_var_reps::*;
pub use edit_var_sets::*;
pub use edit_weight::*;
pub use edit_weight_sets::*;
pub use edit_workouts::*;
pub use errors::*;
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
