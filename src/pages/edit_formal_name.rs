use crate::app_state::SharedState;
use crate::exercise::ExerciseName;
use crate::pages::editor_builder::*;
use crate::pages::Error;
use axum::http::Uri;

pub fn get_formal_name(state: SharedState, workout: &str, exercise: &str) -> String {
    let post_url = format!("/set-formal-name/{workout}/{exercise}");
    let cancel_url = format!("/exercise/{workout}/{exercise}");

    let notes = &state.read().unwrap().user.notes;
    let program = &state.read().unwrap().user.program;
    let workout = program.find(workout).unwrap();
    let exercise = workout.find(&ExerciseName(exercise.to_owned())).unwrap();
    let d = exercise.data();

    let name = d.formal_name.0.to_owned();
    let mut items: Vec<_> = notes.names().iter().map(|n| n.0.clone()).collect();
    items.push("…".to_owned());
    let javascript = include_str!("../../files/formal_name.js");

    let widgets: Vec<Box<dyn Widget>> = vec![
        Box::new(Prolog::with_title("Edit Formal Name")),
        Box::new(TextInput::new(
            "Name",
            &name,
            "The formal name is used to select the notes to show.",
        )),
        Box::new(List::with_names("names", items, "").with_custom_js(javascript)),
        Box::new(StdButtons::new(&cancel_url)),
    ];

    build_editor(&post_url, widgets)
}

pub fn post_set_formal_name(
    state: SharedState,
    workout: &str,
    exercise: &str,
    name: &str,
) -> Result<Uri, Error> {
    let path = format!("/exercise/{workout}/{exercise}");
    let exercise = ExerciseName(exercise.to_owned());

    {
        let program = &mut state.write().unwrap().user.program;
        let workout = program.find_mut(&workout).unwrap();
        let exercise = workout.find_mut(&exercise).unwrap();
        exercise.try_set_formal_name(name)?;
    }

    super::post_epilog(state, &path)
}
