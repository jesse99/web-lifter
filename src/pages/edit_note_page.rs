use crate::{exercise::ExerciseName, notes::Notes, pages::SharedState, program::Program};
use anyhow::Context;
use serde::{Deserialize, Serialize};

pub fn get_edit_note_page(
    state: SharedState,
    workout: &str,
    exercise: &str,
) -> Result<String, anyhow::Error> {
    let handlebars = &state.read().unwrap().handlebars;
    let program = &state.read().unwrap().user.program;
    let notes = &state.read().unwrap().user.notes;

    let template = include_str!("../../files/edit_note.html");
    let data = EditNoteData::new(notes, program, workout, exercise)?;
    Ok(handlebars
        .render_template(template, &data)
        .context("failed to render template")?)
}

#[derive(Serialize, Deserialize)]
struct EditNoteData {
    workout: String,
    exercise: String,
    note: String,
}

impl EditNoteData {
    fn new(
        notes: &Notes,
        program: &Program,
        workout_name: &str,
        exercise_name: &str,
    ) -> Result<EditNoteData, anyhow::Error> {
        let workout = program.find(&workout_name).unwrap();
        let exercise = workout
            .find(&ExerciseName(exercise_name.to_owned()))
            .unwrap();
        let d = exercise.data();
        let note = notes.markdown(&d.formal_name);

        Ok(EditNoteData {
            workout: workout_name.to_owned(),
            exercise: exercise_name.to_owned(),
            note,
        })
    }
}
