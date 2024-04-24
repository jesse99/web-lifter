use super::{DropItem, EditorBuilder, SharedState};
use crate::exercise::{ExerciseName, SetIndex};
use axum::http::Uri;

pub fn get_edit_durations(state: SharedState, workout: &str, exercise: &str) -> String {
    let post_url = format!("/set-durations/{workout}/{exercise}");
    let cancel_url = format!("/exercise/{workout}/{exercise}");

    let name = ExerciseName(exercise.to_owned());
    let program = &state.read().unwrap().user.program;
    let workout = program.find(workout).unwrap();
    let exercise = workout.find(&name).unwrap();
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
    let items = vec![
        DropItem::new("Secs", "secs"),
        DropItem::new("Mins", "mins"),
        DropItem::new("Hours", "hours"),
    ];
    let javascript = include_str!("../../files/durations.js");

    EditorBuilder::new(&post_url)
        .with_title("Edit Durations")
        .with_text_input(
            "Times",
            &durations,
            Some(r#"\s*\d+(\.\d+)?(\s+\d+(\.\d+)?)*"#),
            "Space separated amount of time to wait for each set.",
        )
        .with_float_input(
            "Target",
            &target,
            "0",
            "0.1",
            "Optional goal for times. Users may switch to a harder version of the exercise when reaching the target.",
        )
        .with_dropdown("Units", items, "Mins", javascript)
        .with_std_buttons(&cancel_url)
        .finalize()
}

pub fn post_set_durations(
    state: SharedState,
    workout_name: &str,
    exercise_name: &str,
    durations: Vec<i32>,
    target: Option<i32>,
) -> Result<Uri, anyhow::Error> {
    let exercise_name = ExerciseName(exercise_name.to_owned());

    {
        let program = &mut state.write().unwrap().user.program;
        let workout = program.find_mut(&workout_name).unwrap();
        let exercise = workout.find_mut(&exercise_name).unwrap();
        let (_, e) = exercise.expect_durations_mut();
        e.try_set_durations(durations)?;
        e.try_set_target(target)?;
    }

    {
        let user = &mut state.write().unwrap().user;
        if let Err(e) = crate::persist::save(user) {
            user.errors.push(format!("{e}")); // not fatal so we don't return an error
        }
    }

    let path = format!("/exercise/{workout_name}/{exercise_name}");
    let uri = url_escape::encode_path(&path);
    let uri = uri.parse()?;
    Ok(uri)
}
