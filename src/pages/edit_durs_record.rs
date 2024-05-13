use crate::app_state::SharedState;
use crate::pages::editor_builder::*;
use crate::pages::Error;
use crate::validation_err;
use crate::{exercise::ExerciseName, history::CompletedSets, weights};
use axum::http::Uri;

pub fn get_edit_durs_record(
    state: SharedState,
    workout: &str,
    exercise: &str,
    id: u64,
) -> Result<String, Error> {
    let post_url = format!("/set-durs-record/{workout}/{exercise}/{id}");
    let cancel_url = format!("/exercise/{workout}/{exercise}");

    let name = ExerciseName(exercise.to_owned());
    let history = &state.read().unwrap().user.history;

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
    let items = [("Secs", "secs"), ("Mins", "mins"), ("Hours", "hours")];
    let javascript = include_str!("../../files/durations.js");

    let widgets: Vec<Box<dyn Widget>> = vec![
        Box::new(Prolog::with_title("Edit Durations Record")),
        Box::new(
            TextInput::new(
                "Times",
                &durations,
                "Space separated list of times that were done for each set.",
            )
            .with_pattern(r#"\s*\d+(\.\d+)?(\s+\d+(\.\d+)?)*"#)
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
        Box::new(Dropdown::new("Units", &items, javascript).with_active("Mins")),
        Box::new(StdButtons::new(&cancel_url)),
    ];

    Ok(build_editor(&post_url, widgets))
}

pub fn post_set_durs_record(
    state: SharedState,
    workout: &str,
    exercise: &str,
    durations: Vec<i32>,
    weights: Vec<f32>,
    comment: String,
    id: u64,
) -> Result<Uri, Error> {
    let path = format!("/exercise/{workout}/{exercise}");
    let exercise = ExerciseName(exercise.to_owned());

    {
        let history = &mut state.write().unwrap().user.history;
        let record = history.find_record_mut(&exercise, id)?;
        let sets = if durations.len() == weights.len() {
            durations
                .iter()
                .copied()
                .zip(weights.iter().map(|w| Some(*w)))
                .collect()
        } else if weights.is_empty() {
            durations.iter().map(|r| (*r, None)).collect()
        } else {
            return validation_err!("Weights must be empty or match durations");
        };
        record.sets = Some(CompletedSets::Durations(sets));
        if !comment.is_empty() {
            record.comment = Some(comment);
        } else {
            record.comment = None;
        }
    }

    let uri = url_escape::encode_path(&path);
    let uri = uri.parse()?;
    Ok(uri)
}
