use crate::{
    exercise::{ExerciseName, SetIndex},
    pages::SharedState,
    program::Program,
};
use anyhow::Context;
use serde::{Deserialize, Serialize};

pub fn get_edit_fixed_reps_page(
    state: SharedState,
    workout: &str,
    exercise: &str,
) -> Result<String, anyhow::Error> {
    let handlebars = &state.read().unwrap().handlebars;
    let program = &state.read().unwrap().user.program;

    let template = include_str!("../../files/edit_fixed_reps.html");
    let data = EditFixedRepsData::new(program, workout, exercise)?;
    Ok(handlebars
        .render_template(template, &data)
        .context("failed to render template")?)
}

#[derive(Serialize, Deserialize)]
struct EditFixedRepsData {
    workout: String,
    exercise: String,
    warmups: String,
    worksets: String,
}

impl EditFixedRepsData {
    fn new(
        program: &Program,
        workout_name: &str,
        exercise_name: &str,
    ) -> Result<EditFixedRepsData, anyhow::Error> {
        let workout = program.find(&workout_name).unwrap();
        let exercise = workout
            .find(&ExerciseName(exercise_name.to_owned()))
            .unwrap();
        let (_, e) = exercise.expect_fixed_reps();

        let warmups: Vec<_> = (0..e.num_warmups())
            .map(|i| e.set(SetIndex::Warmup(i)))
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
            .map(|i| e.set(SetIndex::Workset(i)))
            .map(|r| {
                if r.percent == 100 {
                    format!("{}", r.reps)
                } else {
                    format!("{}/{}", r.reps, r.percent)
                }
            })
            .collect();
        let worksets = worksets.join(" ");

        Ok(EditFixedRepsData {
            workout: workout_name.to_owned(),
            exercise: exercise_name.to_owned(),
            warmups,
            worksets,
        })
    }
}
