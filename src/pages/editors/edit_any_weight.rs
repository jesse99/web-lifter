use crate::app_state::SharedState;
use crate::errors::Error;
use crate::exercise::ExerciseName;
use crate::pages::editor_builder::*;
use axum::http::Uri;

pub fn get_edit_any_weight(state: SharedState, workout: &str, exercise: &str) -> String {
    let post_url = format!("/set-any-weight/{workout}/{exercise}");
    let cancel_url = format!("/exercise/{workout}/{exercise}");

    let program = &state.read().unwrap().user.program;
    let workout = program.find(workout).unwrap();
    let exercise = workout.find(&ExerciseName(exercise.to_owned())).unwrap();
    let data = exercise.data();
    let weight = data.weight.unwrap_or(0.0);

    let widgets: Vec<Box<dyn Widget>> = vec![
        Box::new(Prolog::with_title("Edit Weight")),
        Box::new(FloatInput::new(
            "Weight",
            Some(weight),
            "Arbitrary weight (i.e. there isn't a weight set).",
        )),
        Box::new(StdButtons::new(&cancel_url)),
    ];

    build_editor(&post_url, widgets)
}

// Used by set_any_weight and set_weight.
pub fn post_set_weight(
    state: SharedState,
    workout: &str,
    exercise: &str,
    weight: Option<f32>,
) -> Result<Uri, Error> {
    let path = format!("/exercise/{workout}/{exercise}");
    let exercise = ExerciseName(exercise.to_owned());

    {
        let program = &mut state.write().unwrap().user.program;
        let workout = program.find_mut(&workout).unwrap();
        let exercise = workout.find_mut(&exercise).unwrap();
        exercise.try_set_weight(weight)?;
    }

    crate::pages::post_epilog(state, &path)
}
