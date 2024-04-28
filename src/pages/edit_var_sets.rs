use crate::pages::editor_builder::*;
use crate::{exercise::ExerciseName, pages::SharedState};
use axum::http::Uri;

pub fn get_edit_var_sets(state: SharedState, workout: &str, exercise: &str) -> String {
    let post_url = format!("/set-var-sets/{workout}/{exercise}");
    let cancel_url = format!("/exercise/{workout}/{exercise}");

    let program = &state.read().unwrap().user.program;
    let workout = program.find(&workout).unwrap();
    let exercise = workout.find(&ExerciseName(exercise.to_owned())).unwrap();
    let (_, e) = exercise.expect_var_sets();

    let target = e.target() as f32;

    let widgets: Vec<Box<dyn Widget>> = vec![
        Box::new(Prolog::with_title("Edit Variable Sets")),
        Box::new(
            FloatInput::new(
                "Target",
                Some(target),
                "Number of reps to do over an arbitrary number of sets.",
            )
            .with_min(1.0)
            .with_step(1.0)
            .with_required(),
        ),
        Box::new(StdButtons::new(&cancel_url)),
    ];

    build_editor(&post_url, widgets)
}

pub fn post_set_var_sets(
    state: SharedState,
    workout_name: &str,
    exercise_name: &str,
    target: i32,
) -> Result<Uri, anyhow::Error> {
    let exercise_name = ExerciseName(exercise_name.to_owned());

    {
        let program = &mut state.write().unwrap().user.program;
        let workout = program.find_mut(&workout_name).unwrap();
        let exercise = workout.find_mut(&exercise_name).unwrap();
        let (_, e) = exercise.expect_var_sets_mut();
        e.try_set_target(target)?;
    }

    let path = format!("/exercise/{workout_name}/{exercise_name}");
    super::post_epilog(state, &path)
}
