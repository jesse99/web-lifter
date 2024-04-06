use crate::{exercise::ExerciseName, pages::SharedState, program::Program};
use anyhow::Context;
use serde::{Deserialize, Serialize};

pub fn get_edit_var_sets_page(
    state: SharedState,
    workout: &str,
    exercise: &str,
) -> Result<String, anyhow::Error> {
    let handlebars = &state.read().unwrap().handlebars;
    let program = &state.read().unwrap().user.program;

    let template = include_str!("../../files/edit_var_sets.html");
    let data = EditVarSetsData::new(program, workout, exercise)?;
    Ok(handlebars
        .render_template(template, &data)
        .context("failed to render template")?)
}

#[derive(Serialize, Deserialize)]
struct EditVarSetsData {
    workout: String,
    exercise: String,
    target: String,
}

impl EditVarSetsData {
    fn new(
        program: &Program,
        workout_name: &str,
        exercise_name: &str,
    ) -> Result<EditVarSetsData, anyhow::Error> {
        let workout = program.find(&workout_name).unwrap();
        let exercise = workout
            .find(&ExerciseName(exercise_name.to_owned()))
            .unwrap();
        let (_, e) = exercise.expect_var_sets();

        let target = format!("{}", e.target());

        Ok(EditVarSetsData {
            workout: workout_name.to_owned(),
            exercise: exercise_name.to_owned(),
            target,
        })
    }
}
