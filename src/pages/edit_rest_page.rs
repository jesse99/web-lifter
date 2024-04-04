use crate::{exercise::ExerciseName, pages::SharedState, program::Program};
use anyhow::Context;
use serde::{Deserialize, Serialize};

pub fn get_edit_rest_page(
    state: SharedState,
    workout: &str,
    exercise: &str,
) -> Result<String, anyhow::Error> {
    let handlebars = &state.read().unwrap().handlebars;
    let program = &state.read().unwrap().user.program;

    let template = include_str!("../../files/edit_rest.html");
    let data = EditRestData::new(program, workout, exercise)?;
    Ok(handlebars
        .render_template(template, &data)
        .context("failed to render template")?)
}

#[derive(Serialize, Deserialize)]
struct EditRestData {
    workout: String,
    exercise: String,
    rest: String,
    last_rest: String,
}

impl EditRestData {
    fn new(
        program: &Program,
        workout_name: &str,
        exercise_name: &str,
    ) -> Result<EditRestData, anyhow::Error> {
        let workout = program.find(&workout_name).unwrap();
        let exercise = workout
            .find(&ExerciseName(exercise_name.to_owned()))
            .unwrap();
        let d = exercise.data();

        let rest = if let Some(r) = d.rest {
            format!("{}", r)
        } else {
            "".to_owned()
        };

        let last_rest = if let Some(r) = d.last_rest {
            format!("{}", r)
        } else {
            "".to_owned()
        };

        Ok(EditRestData {
            workout: workout_name.to_owned(),
            exercise: exercise_name.to_owned(),
            rest,
            last_rest,
        })
    }
}
