use crate::app_state::SharedState;
use crate::errors::Error;
use crate::exercise::ExerciseName;
use crate::pages::editor_builder::*;
use axum::http::Uri;

pub fn get_edit_name(value: &str, help: &str, post_url: &str, cancel_url: &str) -> String {
    let widgets: Vec<Box<dyn Widget>> = vec![
        Box::new(Prolog::with_title("Edit Name")),
        Box::new(TextInput::new("Name", &value, help).with_required()),
        Box::new(StdButtons::new(&cancel_url)),
    ];

    build_editor(&post_url, widgets)
}

pub fn post_set_add_workout(state: SharedState, name: &str) -> Result<Uri, Error> {
    {
        let program = &mut state.write().unwrap().user.program;
        program.try_add_workout(name)?;
    }

    super::post_epilog(state, "/")
}

pub fn post_set_program_name(state: SharedState, new_name: &str) -> Result<Uri, Error> {
    let path = "/";

    {
        let program = &mut state.write().unwrap().user.program;
        if program.name != new_name {
            program.name = new_name.to_string(); // TODO need some validation once we support multiple programs
        }
    }

    super::post_epilog(state, &path)
}

pub fn post_set_workout_name(
    state: SharedState,
    old_name: &str,
    new_name: &str,
) -> Result<Uri, Error> {
    let path = format!("/workout/{new_name}");

    if old_name != new_name {
        let program = &mut state.write().unwrap().user.program;
        program.try_change_workout_name(&old_name, new_name)?;
    }

    super::post_epilog(state, &path)
}

pub fn post_set_exercise_name(
    state: SharedState,
    workout: &str,
    old_name: &str,
    new_name: &str,
) -> Result<Uri, Error> {
    let path = format!("/exercise/{workout}/{new_name}");

    if old_name != new_name {
        let old_name = ExerciseName(old_name.to_owned());
        {
            let program = &mut state.write().unwrap().user.program;
            let workout = program.find_mut(&workout).unwrap();
            workout.try_change_exercise_name(&old_name, new_name)?;
        }
    }

    super::post_epilog(state, &path)
}
