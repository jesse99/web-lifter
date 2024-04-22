use super::{EditorBuilder, SharedState};
use crate::{exercise::ExerciseName, weights};
use axum::http::Uri;

pub fn get_edit_any_weight(state: SharedState, workout: &str, exercise: &str) -> String {
    let post_url = format!("/set-any-weight/{workout}/{exercise}");
    let cancel_url = format!("/exercise/{workout}/{exercise}");

    let program = &state.read().unwrap().user.program;
    let workout = program.find(workout).unwrap();
    let exercise = workout.find(&ExerciseName(exercise.to_owned())).unwrap();
    let data = exercise.data();

    let weight = if let Some(current) = data.weight {
        weights::format_weight(current, "")
    } else {
        "0.0".to_owned()
    };
    EditorBuilder::new(&post_url)
        .with_title("Edit Weight")
        .with_float_input(
            "Weight",
            &weight,
            "0",
            "0.01",
            "Arbitrary weight (i.e. there isn't a weight set).",
        )
        .with_std_buttons(&cancel_url)
        .finalize()
}

// Used by set_any_weight and set_weight.
pub fn post_set_weight(
    state: SharedState,
    workout: &str,
    exercise: &str,
    weight: Option<f32>,
) -> Result<Uri, anyhow::Error> {
    let path = format!("/exercise/{workout}/{exercise}");
    let exercise = ExerciseName(exercise.to_owned());

    {
        let program = &mut state.write().unwrap().user.program;
        let workout = program.find_mut(&workout).unwrap();
        let exercise = workout.find_mut(&exercise).unwrap();
        exercise.try_set_weight(weight)?;
    }

    super::post_epilog(state, &path)
}
