use crate::{
    exercise::{self, ExerciseName},
    pages::SharedState,
    program::Program,
    weights::{self, Weights},
};
use anyhow::Context;
use serde::{Deserialize, Serialize};

pub fn get_edit_exercises_page(state: SharedState, workout: &str) -> Result<String, anyhow::Error> {
    let handlebars = &state.read().unwrap().handlebars;
    let program = &state.read().unwrap().user.program;

    let template = include_str!("../../files/edit_exercises.html");
    let data = EditExercisesData::new(program, workout)?;
    Ok(handlebars
        .render_template(template, &data)
        .context("failed to render template")?)
}

#[derive(Serialize, Deserialize)]
struct EditExercisesData {
    workout: String,
    exercises: Vec<ExercisesData>,
}

impl EditExercisesData {
    fn new(program: &Program, workout_name: &str) -> Result<EditExercisesData, anyhow::Error> {
        let workout = program.find(&workout_name).unwrap();

        let exercises = workout
            .exercises()
            .enumerate()
            .map(|(i, e)| ExercisesData::new(i == 0, &e.name().0))
            .collect();

        Ok(EditExercisesData {
            workout: workout_name.to_owned(),
            exercises,
        })
    }
}

#[derive(Serialize, Deserialize)]
struct ExercisesData {
    active: String,  // "active" or ""
    current: String, // "true" or "false"
    exercise: String,
}

impl ExercisesData {
    fn new(is_active: bool, exercise: &str) -> ExercisesData {
        let active = if is_active {
            "active".to_owned()
        } else {
            "".to_owned()
        };
        let current = if is_active {
            "true".to_owned()
        } else {
            "false".to_owned()
        };
        let exercise = exercise.to_owned();
        ExercisesData {
            active,
            current,
            exercise,
        }
    }
}
