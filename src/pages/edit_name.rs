use super::SharedState;
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

pub fn post_set_workout_name(
    state: SharedState,
    old_name: &str,
    new_name: &str,
) -> Result<Uri, anyhow::Error> {
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
) -> Result<Uri, anyhow::Error> {
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
