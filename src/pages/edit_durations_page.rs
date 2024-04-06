use crate::{
    exercise::{ExerciseName, SetIndex},
    pages::SharedState,
    program::Program,
};
use anyhow::Context;
use serde::{Deserialize, Serialize};

pub fn get_edit_durations_page(
    state: SharedState,
    workout: &str,
    exercise: &str,
) -> Result<String, anyhow::Error> {
    let handlebars = &state.read().unwrap().handlebars;
    let program = &state.read().unwrap().user.program;

    let template = include_str!("../../files/edit_durations.html");
    let data = EditDurationsData::new(program, workout, exercise)?;
    Ok(handlebars
        .render_template(template, &data)
        .context("failed to render template")?)
}

#[derive(Serialize, Deserialize)]
struct EditDurationsData {
    workout: String,
    exercise: String,
    durations: String,
    target: String,
}

impl EditDurationsData {
    fn new(
        program: &Program,
        workout_name: &str,
        exercise_name: &str,
    ) -> Result<EditDurationsData, anyhow::Error> {
        let workout = program.find(&workout_name).unwrap();
        let exercise = workout
            .find(&ExerciseName(exercise_name.to_owned()))
            .unwrap();
        let (_, e) = exercise.expect_durations();

        let durations: Vec<_> = (0..e.num_sets())
            .map(|i| e.set(SetIndex::Workset(i)) as f32 / 60.0)
            .map(|t| format!("{t:.2}"))
            .collect();
        let durations = durations.join(" ");

        let target = if let Some(t) = e.target() {
            format!("{:.2}", t as f32 / 60.0)
        } else {
            "".to_owned()
        };

        Ok(EditDurationsData {
            workout: workout_name.to_owned(),
            exercise: exercise_name.to_owned(),
            durations,
            target,
        })
    }
}
