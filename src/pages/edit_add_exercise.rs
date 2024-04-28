use crate::exercise::Exercise;
use crate::pages::editor_builder::*;
use crate::pages::SharedState;
use axum::http::Uri;

pub fn get_add_exercise(workout: &str) -> String {
    let post_url = format!("/append-exercise/{workout}");
    let cancel_url = format!("/workout/{workout}");

    let items = vec![
        ("Durations: 3 sets of 30s", "durations"),
        ("Fixed Reps: 3 sets of 12 reps", "fixed"),
        ("Variable Reps: 3 sets of 4-8 reps", "var-reps"),
        ("Variable Sets: 3+ sets of 24 total reps", "var-sets"),
    ];
    let items = items
        .iter()
        .map(|(l, v)| (l.to_string(), v.to_string()))
        .collect();

    let widgets: Vec<Box<dyn Widget>> = vec![
        Box::new(Prolog::with_title("Add Exercise")),
        Box::new(TextInput::new("Name", "", "Must be unique within the workout.").with_required()),
        Box::new(
            Radio::new(
                "types",
                items,
                "All exercise types may include a target weight, weight set (e.g. dumbbells or plates), and an amount of time to rest after each set.",
            )
            .with_checked("var-reps"),
        ),
        Box::new(StdButtons::new(&cancel_url)),
    ];

    build_editor(&post_url, widgets)
}

pub fn post_append_exercise(
    state: SharedState,
    workout_name: &str,
    exercise: Exercise,
) -> Result<Uri, anyhow::Error> {
    {
        let program = &mut state.write().unwrap().user.program;
        let workout = program.find_mut(&workout_name).unwrap();
        workout.try_add_exercise(exercise)?;
    }

    let path = format!("/workout/{workout_name}");
    super::post_epilog(state, &path)
}
