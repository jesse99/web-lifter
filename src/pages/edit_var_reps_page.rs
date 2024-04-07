use crate::{exercise::ExerciseName, pages::SharedState, program::Program};
use anyhow::Context;
use serde::{Deserialize, Serialize};

pub fn get_edit_var_reps_page(
    state: SharedState,
    workout: &str,
    exercise: &str,
) -> Result<String, anyhow::Error> {
    let handlebars = &state.read().unwrap().handlebars;
    let program = &state.read().unwrap().user.program;

    let template = include_str!("../../files/edit_var_reps.html");
    let data = EditVarRepsData::new(program, workout, exercise)?;
    Ok(handlebars
        .render_template(template, &data)
        .context("failed to render template")?)
}

#[derive(Serialize, Deserialize)]
struct EditVarRepsData {
    workout: String,
    exercise: String,
    warmups: String,
    worksets: String,
}

impl EditVarRepsData {
    fn new(
        program: &Program,
        workout_name: &str,
        exercise_name: &str,
    ) -> Result<EditVarRepsData, anyhow::Error> {
        let workout = program.find(&workout_name).unwrap();
        let exercise = workout
            .find(&ExerciseName(exercise_name.to_owned()))
            .unwrap();
        let (_, e) = exercise.expect_var_reps();

        let warmups: Vec<_> = (0..e.num_warmups())
            .map(|i| e.warmup(i))
            .map(|r| {
                if r.percent == 100 {
                    format!("{}", r.reps)
                } else {
                    format!("{}/{}", r.reps, r.percent)
                }
            })
            .collect();
        let warmups = warmups.join(" ");

        let worksets: Vec<_> = (0..e.num_worksets())
            .map(|i| e.workset(i))
            .map(|r| {
                let prefix = if r.min == r.max {
                    format!("{}", r.min)
                } else {
                    format!("{}-{}", r.min, r.max)
                };
                let suffix = if r.percent == 100 {
                    "".to_owned()
                } else {
                    format!("/{}", r.percent)
                };
                format!("{prefix}{suffix}")
            })
            .collect();
        let worksets = worksets.join(" ");

        Ok(EditVarRepsData {
            workout: workout_name.to_owned(),
            exercise: exercise_name.to_owned(),
            warmups,
            worksets,
        })
    }
}
