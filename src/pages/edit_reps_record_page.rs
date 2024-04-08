use crate::{
    exercise::ExerciseName,
    history::{CompletedSets, History},
    pages::SharedState,
    weights::{self},
};
use anyhow::Context;
use serde::{Deserialize, Serialize};

pub fn get_edit_reps_record_page(
    state: SharedState,
    workout: &str,
    exercise: &str,
    id: u64,
) -> Result<String, anyhow::Error> {
    let handlebars = &state.read().unwrap().handlebars;
    let history = &state.read().unwrap().user.history;

    let template = include_str!("../../files/edit_reps_record.html");
    let data = EditRepsRecordData::new(history, workout, exercise, id)?;
    Ok(handlebars
        .render_template(template, &data)
        .context("failed to render template")?)
}

#[derive(Serialize, Deserialize)]
struct EditRepsRecordData {
    workout: String,
    exercise: String,
    reps: String,
    weights: String,
    comment: String,
    id: String,
}

impl EditRepsRecordData {
    fn new(
        history: &History,
        workout_name: &str,
        exercise_name: &str,
        id: u64,
    ) -> Result<EditRepsRecordData, anyhow::Error> {
        let name = ExerciseName(exercise_name.to_owned());
        let record = history.find_record(&name, id)?;
        let (reps, weights) = match &record.sets {
            Some(CompletedSets::Reps(r)) => (
                r.iter()
                    .map(|x| format!("{}", x.0))
                    .collect::<Vec<String>>()
                    .join(" "),
                r.iter()
                    .map(|x| x.1.map_or("".to_owned(), |w| weights::format_weight(w, "")))
                    .collect::<Vec<String>>()
                    .join(" "),
            ),
            Some(_) => panic!("expected reps sets"),
            None => panic!("expected non-empty reps sets"),
        };
        let comment = if let Some(c) = &record.comment {
            c.clone()
        } else {
            "".to_owned()
        };

        Ok(EditRepsRecordData {
            workout: workout_name.to_owned(),
            exercise: exercise_name.to_owned(),
            reps,
            weights,
            comment,
            id: format!("{id}"),
        })
    }
}
