use crate::app_state::SharedState;
use crate::errors::Error;
use crate::exercise::ExerciseName;
use crate::pages::editor_builder::*;
use axum::http::Uri;

pub fn get_edit_note(state: SharedState, workout: &str, exercise: &str) -> String {
    let post_url = format!("/set-note/{workout}/{exercise}");
    let revert_url = format!("/revert-note/{workout}/{exercise}");
    let cancel_url = format!("/exercise/{workout}/{exercise}");

    let program = &state.read().unwrap().user.program;
    let notes = &state.read().unwrap().user.notes;

    let workout = program.find(&workout).unwrap();
    let exercise = workout.find(&ExerciseName(exercise.to_owned())).unwrap();
    let d = exercise.data();
    let note = notes.markdown(&d.formal_name);
    let custom = ("Revert", "btn-danger", revert_url.as_ref());

    let widgets: Vec<Box<dyn Widget>> = vec![
        Box::new(Prolog::with_title("Edit Note")),
        Box::new(
            TextArea::new("note", 15, 60, "Text and links that describe how to perform the exercise in <a href=\"https://www.markdownguide.org/cheat-sheet/\">markdown</a> format")
            .with_spellcheck()
            .with_autocapitalize("sentences")
            .with_body(&note)),
        Box::new(StdButtons::with_custom(&cancel_url, custom)),
    ];

    build_editor(&post_url, widgets)
}

pub fn post_set_note(
    state: SharedState,
    workout_name: &str,
    exercise_name: &str,
    note: String,
) -> Result<Uri, Error> {
    let exercise_name = ExerciseName(exercise_name.to_owned());

    {
        let formal_name = {
            let program = &state.read().unwrap().user.program;
            let workout = program.find(&workout_name).unwrap();
            let exercise = workout.find(&exercise_name).unwrap();
            let d = exercise.data();
            d.formal_name.clone()
        };

        let notes = &mut state.write().unwrap().user.notes;
        notes.set_markdown(formal_name, note);
    }

    let path = format!("/exercise/{workout_name}/{exercise_name}");
    crate::pages::post_epilog(state, &path)
}

pub fn post_revert_note(
    state: SharedState,
    workout_name: &str,
    exercise_name: &str,
) -> Result<Uri, Error> {
    let exercise_name = ExerciseName(exercise_name.to_owned());

    {
        let formal_name = {
            let program = &state.read().unwrap().user.program;
            let workout = program.find(&workout_name).unwrap();
            let exercise = workout.find(&exercise_name).unwrap();
            let d = exercise.data();
            d.formal_name.clone()
        };

        let notes = &mut state.write().unwrap().user.notes;
        notes.revert_markdown(formal_name);
    }

    let path = format!("/exercise/{workout_name}/{exercise_name}");
    crate::pages::post_epilog(state, &path)
}
