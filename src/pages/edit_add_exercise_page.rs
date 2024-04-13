use crate::pages::SharedState;
use anyhow::Context;
use serde::{Deserialize, Serialize};

pub fn get_add_exercise_page(state: SharedState, workout: &str) -> Result<String, anyhow::Error> {
    let handlebars = &state.read().unwrap().handlebars;

    let template = include_str!("../../files/add_exercise.html");
    let data = AddExerciseData::new(workout)?;
    Ok(handlebars
        .render_template(template, &data)
        .context("failed to render template")?)
}

#[derive(Serialize, Deserialize)]
struct AddExerciseData {
    workout: String,
}

impl AddExerciseData {
    fn new(workout_name: &str) -> Result<AddExerciseData, anyhow::Error> {
        Ok(AddExerciseData {
            workout: workout_name.to_owned(),
        })
    }
}
