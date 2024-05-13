use crate::app_state::SharedState;
use crate::exercise::ExerciseName;
use crate::pages::editor_builder::*;
use crate::pages::Error;
use axum::http::Uri;

pub fn get_edit_rest(state: SharedState, workout: &str, exercise: &str) -> String {
    let post_url = format!("/set-rest/{workout}/{exercise}");
    let cancel_url = format!("/exercise/{workout}/{exercise}");

    let program = &state.read().unwrap().user.program;
    let workout = program.find(&workout).unwrap();
    let exercise = workout.find(&ExerciseName(exercise.to_owned())).unwrap();
    let d = exercise.data();
    let rest = d.rest.map(|r| r as f32 / 60.0);
    let last_rest = d.last_rest.map(|r| r as f32 / 60.0);

    let items = [("Secs", "secs"), ("Mins", "mins"), ("Hours", "hours")];
    let javascript = include_str!("../../files/rest.js");

    let widgets: Vec<Box<dyn Widget>> = vec![
        Box::new(Prolog::with_title("Edit Rest")),
        Box::new(FloatInput::new(
            "Rest",
            rest,
            "Amount of time to rest after each work set.",
        )),
        Box::new(FloatInput::new(
            "Last Rest",
            last_rest,
            "If set then overrides the rest for the last work set.",
        )),
        Box::new(Dropdown::new("Units", &items, javascript).with_active("Mins")),
        Box::new(StdButtons::new(&cancel_url)),
    ];

    build_editor(&post_url, widgets)
}

pub fn post_set_rest(
    state: SharedState,
    workout_name: &str,
    exercise_name: &str,
    rest: Option<i32>,
    last_rest: Option<i32>,
) -> Result<Uri, Error> {
    let exercise_name = ExerciseName(exercise_name.to_owned());

    {
        let program = &mut state.write().unwrap().user.program;
        let workout = program.find_mut(&workout_name).unwrap();
        let exercise = workout.find_mut(&exercise_name).unwrap();
        exercise.try_set_rest(rest)?;
        exercise.try_set_last_rest(last_rest)?;
    }

    let path = format!("/exercise/{workout_name}/{exercise_name}");
    super::post_epilog(state, &path)
}
