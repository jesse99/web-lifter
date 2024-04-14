use crate::{exercise::ExerciseName, notes::Notes, pages::SharedState, program::Program};
use anyhow::Context;
use serde::{Deserialize, Serialize};

pub fn get_edit_formal_name_page(
    state: SharedState,
    workout: &str,
    exercise: &str,
) -> Result<String, anyhow::Error> {
    let handlebars = &state.read().unwrap().handlebars;
    let program = &state.read().unwrap().user.program;
    let notes = &state.read().unwrap().user.notes;

    let template = include_str!("../../files/edit_formal_name.html");
    let data = EditFormalNameData::new(notes, program, workout, exercise)?;
    Ok(handlebars
        .render_template(template, &data)
        .context("failed to render template")?)
}

#[derive(Serialize, Deserialize)]
struct EditFormalNameData<'a> {
    workout: String,
    exercise: String,
    name: String,

    #[serde(borrow)]
    names: Vec<FormalNameEntry<'a>>,
}

#[derive(Serialize, Deserialize)]
struct FormalNameEntry<'a> {
    name: &'a str,
}

impl<'a> EditFormalNameData<'a> {
    fn new(
        notes: &'a Notes,
        program: &Program,
        workout: &str,
        exercise: &str,
    ) -> Result<EditFormalNameData<'a>, anyhow::Error> {
        let workout = program.find(&workout).unwrap();
        let exercise = workout.find(&ExerciseName(exercise.to_owned())).unwrap();
        let d = exercise.data();

        let workout = workout.name.clone();
        let exercise = d.name.0.clone();
        let name = d.formal_name.0.to_owned();
        let names = notes
            .names()
            .iter()
            .map(|n| FormalNameEntry { name: &n.0 })
            .collect();

        Ok(EditFormalNameData {
            workout,
            exercise,
            name,
            names,
        })
    }
}
