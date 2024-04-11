use crate::{
    exercise::ExerciseName,
    history::{CompletedSets, History},
    pages::SharedState,
    weights::{self},
};
use anyhow::Context;
use serde::{Deserialize, Serialize};

// TODO should there be a way to delete a record? maybe a button in edit page? or set
// durations/reps to empty?
pub fn get_edit_durs_record_page(
    state: SharedState,
    workout: &str,
    exercise: &str,
    id: u64,
) -> Result<String, anyhow::Error> {
    let handlebars = &state.read().unwrap().handlebars;
    let history = &state.read().unwrap().user.history;

    let template = include_str!("../../files/edit_durs_record.html");
    let data = EditDurationsRecordData::new(history, workout, exercise, id)?;
    Ok(handlebars
        .render_template(template, &data)
        .context("failed to render template")?)
}

#[derive(Serialize, Deserialize)]
struct EditDurationsRecordData {
    workout: String,
    exercise: String,
    durations: String,
    weights: String,
    comment: String,
    id: String,
}

impl EditDurationsRecordData {
    fn new(
        history: &History,
        workout_name: &str,
        exercise_name: &str,
        id: u64,
    ) -> Result<EditDurationsRecordData, anyhow::Error> {
        let name = ExerciseName(exercise_name.to_owned());
        let record = history.find_record(&name, id)?;
        let (durations, weights) = match &record.sets {
            Some(CompletedSets::Durations(r)) => (
                r.iter()
                    .map(|x| format!("{:.2}", x.0 as f32 / 60.0))
                    .collect::<Vec<String>>()
                    .join(" "),
                r.iter()
                    .map(|x| x.1.map_or("".to_owned(), |w| weights::format_weight(w, "")))
                    .collect::<Vec<String>>()
                    .join(" "),
            ),
            Some(_) => panic!("expected durations sets"),
            None => panic!("expected non-empty durations sets"),
        };
        let comment = if let Some(c) = &record.comment {
            c.clone()
        } else {
            "".to_owned()
        };

        Ok(EditDurationsRecordData {
            workout: workout_name.to_owned(),
            exercise: exercise_name.to_owned(),
            durations,
            weights,
            comment,
            id: format!("{id}"),
        })
    }
}
