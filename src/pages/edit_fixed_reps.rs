use crate::app_state::SharedState;
use crate::errors::Error;
use crate::exercise::{ExerciseName, FixedReps, SetIndex};
use crate::pages::editor_builder::*;
use axum::http::Uri;

pub fn get_edit_fixed_reps(state: SharedState, workout: &str, exercise: &str) -> String {
    let post_url = format!("/set-fixed-reps/{workout}/{exercise}");
    let cancel_url = format!("/exercise/{workout}/{exercise}");

    let name = ExerciseName(exercise.to_owned());
    let program = &state.read().unwrap().user.program;
    let workout = program.find(workout).unwrap();
    let exercise = workout.find(&name).unwrap();
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

    let widgets: Vec<Box<dyn Widget>> = vec![
        Box::new(Prolog::with_title("Edit Fixed Reps")),
        Box::new(
            TextInput::new(
                "Warmups",
                &warmups,
                "Space separated list of rep/percent, e.g. \"5/70 3/80 1/90\".",
            )
            .with_pattern(r#"\s*(\d+(/\d+)?(\s+\d+(/\d+)?)*)?\s*"#),
        ),
        Box::new(
            TextInput::new(
                "Worksets",
                &worksets,
                "Formatted like warmups except that if percent is missing 100 is used, e.g. \"5 5 5.\".",
            )
            .with_pattern(r#"\s*\d+(/\d+)?(\s+\d+(/\d+)?)*\s*"#)
            .with_required(),
        ),
                        Box::new(StdButtons::new(&cancel_url)),
    ];

    build_editor(&post_url, widgets)
}

pub fn post_set_fixed_reps(
    state: SharedState,
    workout: &str,
    exercise: &str,
    warmups: Vec<FixedReps>,
    worksets: Vec<FixedReps>,
) -> Result<Uri, Error> {
    let path = format!("/exercise/{workout}/{exercise}");
    let exercise = ExerciseName(exercise.to_owned());

    {
        let program = &mut state.write().unwrap().user.program;
        let workout = program.find_mut(&workout).unwrap();
        let exercise = workout.find_mut(&exercise).unwrap();
        let (d, e) = exercise.expect_fixed_reps_mut();
        e.try_set_warmups(warmups)?;
        e.try_set_worksets(worksets)?;

        if !d.finished {
            exercise.reset(exercise.started());
        }
    }

    super::post_epilog(state, &path)
}
