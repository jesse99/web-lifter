use super::SharedState;
use crate::exercise::{ExerciseName, SetIndex};
use crate::pages::editor_builder::*;
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

    let target = e.target().map(|t| t as f32 / 60.0);
    let items = [("Secs", "secs"), ("Mins", "mins"), ("Hours", "hours")];
    let javascript = include_str!("../../files/durations.js");

    let widgets: Vec<Box<dyn Widget>> = vec![
        Box::new(Prolog::with_title("Edit Durations")),
        Box::new(
            TextInput::new(
                "Times",
                &durations,
                "Space separated amount of time to wait for each set.",
            )
            .with_pattern(r#"\s*\d+(\.\d+)?(\s+\d+(\.\d+)?)*"#)
            .with_required(),
        ),
        Box::new(
            FloatInput::new(
                "Target",
                target,
                "Optional goal for times. Users may switch to a harder version of the exercise when reaching the target.",
            )
        ),
        Box::new(
            Dropdown::new("Units", &items, javascript)
            .with_active("Mins"),
        ),
        Box::new(StdButtons::new(&cancel_url)),
    ];

    build_editor(&post_url, widgets)
}

pub fn post_set_durations(
    state: SharedState,
    workout: &str,
    exercise: &str,
    durations: Vec<i32>,
    target: Option<i32>,
) -> Result<Uri, anyhow::Error> {
    let path = format!("/exercise/{workout}/{exercise}");
    let exercise = ExerciseName(exercise.to_owned());

    {
        let program = &mut state.write().unwrap().user.program;
        let workout = program.find_mut(&workout).unwrap();
        let exercise = workout.find_mut(&exercise).unwrap();
        let (_, e) = exercise.expect_durations_mut();
        e.try_set_durations(durations)?;
        e.try_set_target(target)?;
    }

    super::post_epilog(state, &path)
}
