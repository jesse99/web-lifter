use super::{DropItem, EditorBuilder, SharedState};
use crate::{exercise::ExerciseName, history::CompletedSets, weights};
use axum::http::Uri;

pub fn get_edit_durs_record(
    state: SharedState,
    workout: &str,
    exercise: &str,
    id: u64,
) -> Result<String, anyhow::Error> {
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
    let items = vec![
        DropItem::new("Secs", "secs"),
        DropItem::new("Mins", "mins"),
        DropItem::new("Hours", "hours"),
    ];
    let javascript = include_str!("../../files/durations.js");

    Ok(EditorBuilder::new(&post_url)
        .with_title("Edit Durations Record")
        .with_text_input(
            "Times",
            &durations,
            Some(r#"\s*\d+(\.\d+)?(\s+\d+(\.\d+)?)*"#),
            "Space separated list of times that were done for each set.",
        )
        .with_text_input(
            "Weights",
            &weights,
            Some(r#"\s*(\d+(\.\d+)?(\s+\d+(\.\d+)?)*\s*)?"#),
            "Space separated list of weights that were used for each set.",
        )
        .with_text_input(
            "Comment",
            &comment,
            None,
            "Optional comment, e.g. for exercise difficulty.",
        )
        .with_dropdown("Units", items, "Mins", javascript)
        .with_std_buttons(&cancel_url)
        .finalize())
}

pub fn post_set_durs_record(
    state: SharedState,
    workout_name: &str,
    exercise_name: &str,
    durations: Vec<i32>,
    weights: Vec<f32>,
    comment: String,
    id: u64,
) -> Result<Uri, anyhow::Error> {
    let exercise_name = ExerciseName(exercise_name.to_owned());

    {
        let history = &mut state.write().unwrap().user.history;
        let record = history.find_record_mut(&exercise_name, id)?;
        let sets = if durations.len() == weights.len() {
            durations
                .iter()
                .copied()
                .zip(weights.iter().map(|w| Some(*w)))
                .collect()
        } else if weights.is_empty() {
            durations.iter().map(|r| (*r, None)).collect()
        } else {
            return Err(anyhow::Error::msg(
                "Weights must be empty or match durations",
            ));
        };
        record.sets = Some(CompletedSets::Durations(sets));
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
