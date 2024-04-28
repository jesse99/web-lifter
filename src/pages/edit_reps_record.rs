use crate::pages::editor_builder::*;
use crate::{
    exercise::ExerciseName,
    history::CompletedSets,
    pages::SharedState,
    weights::{self},
};
use axum::http::Uri;

pub fn get_edit_reps_record(
    state: SharedState,
    workout: &str,
    exercise: &str,
    id: u64,
) -> Result<String, anyhow::Error> {
    let post_url = format!("/set-reps-record/{workout}/{exercise}/{id}");
    let cancel_url = format!("/exercise/{workout}/{exercise}");

    let history = &state.read().unwrap().user.history;
    let name = ExerciseName(exercise.to_owned());
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

    let widgets: Vec<Box<dyn Widget>> = vec![
        Box::new(Prolog::with_title("Edit Reps Record")),
        Box::new(
            TextInput::new(
                "Reps",
                &reps,
                "Space separated list of reps that were done for each set.",
            )
            .with_pattern(r#"\s*\d+(\s+\d+)*\s*"#)
            .with_required(),
        ),
        Box::new(
            TextInput::new(
                "Weights",
                &weights,
                "Space separated list of weights that were used for each set.",
            )
            .with_pattern(r#"\s*(\d+(\.\d+)?(\s+\d+(\.\d+)?)*\s*)?"#),
        ),
        Box::new(TextInput::new(
            "Comment",
            &comment,
            "Optional comment, e.g. for exercise difficulty.",
        )),
        Box::new(StdButtons::new(&cancel_url)),
    ];

    Ok(build_editor(&post_url, widgets))
}

pub fn post_set_reps_record(
    state: SharedState,
    workout_name: &str,
    exercise_name: &str,
    reps: Vec<i32>,
    weights: Vec<f32>,
    comment: String,
    id: u64,
) -> Result<Uri, anyhow::Error> {
    let exercise_name = ExerciseName(exercise_name.to_owned());

    {
        let history = &mut state.write().unwrap().user.history;
        let record = history.find_record_mut(&exercise_name, id)?;
        let sets = if reps.len() == weights.len() {
            reps.iter()
                .copied()
                .zip(weights.iter().map(|w| Some(*w)))
                .collect()
        } else if weights.is_empty() {
            reps.iter().map(|r| (*r, None)).collect()
        } else {
            return Err(anyhow::Error::msg("Weights must be empty or match reps"));
        };
        record.sets = Some(CompletedSets::Reps(sets));
        if !comment.is_empty() {
            record.comment = Some(comment);
        } else {
            record.comment = None;
        }
    }

    let path = format!("/exercise/{workout_name}/{exercise_name}");
    let uri = url_escape::encode_path(&path);
    let uri = uri.parse()?;
    Ok(uri)
}
