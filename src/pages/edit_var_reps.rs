use crate::exercise::{FixedReps, VariableReps};
use crate::pages::editor_builder::*;
use crate::{exercise::ExerciseName, pages::SharedState};
use axum::http::Uri;

pub fn get_edit_var_reps(state: SharedState, workout: &str, exercise: &str) -> String {
    let post_url = format!("/set-var-reps/{workout}/{exercise}");
    let cancel_url = format!("/exercise/{workout}/{exercise}");

    let program = &state.read().unwrap().user.program;
    let workout = program.find(&workout).unwrap();
    let exercise = workout.find(&ExerciseName(exercise.to_owned())).unwrap();
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

    let widgets: Vec<Box<dyn Widget>> = vec![
        Box::new(Prolog::with_title("Edit Variable Reps")),
        Box::new(TextInput::new("Warmups", &warmups, "Space separated list of rep/percent, e.g. \"5/70 3/80 1/90\". If percent is missing 100 is used.").with_pattern(r#"\s*(\d+(/\d+)?(\s+\d+(/\d+)?)*)?\s*"#)),
        Box::new(TextInput::new("Worksets", &worksets, "Formatted like warmups except min reps may be used, e.g. \"8 4-8 3-6\".").with_pattern(r#"\s*((\d+-)?\d+(/\d+)?)(\s+((\d+-)?\d+(/\d+)?))*\s*"#).with_required()),
        Box::new(StdButtons::new(&cancel_url)),
    ];

    build_editor(&post_url, widgets)
}

pub fn post_set_var_reps(
    state: SharedState,
    workout_name: &str,
    exercise_name: &str,
    warmups: Vec<FixedReps>,
    worksets: Vec<VariableReps>,
) -> Result<Uri, anyhow::Error> {
    let exercise_name = ExerciseName(exercise_name.to_owned());

    {
        let program = &mut state.write().unwrap().user.program;
        let workout = program.find_mut(&workout_name).unwrap();
        let exercise = workout.find_mut(&exercise_name).unwrap();
        let (d, e) = exercise.expect_var_reps_mut();
        e.try_set_warmups(warmups)?;
        e.try_set_worksets(worksets)?;

        if !d.finished {
            exercise.reset(exercise.started());
        }
    }

    let path = format!("/exercise/{workout_name}/{exercise_name}");
    super::post_epilog(state, &path)
}
