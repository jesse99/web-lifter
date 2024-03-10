use crate::*;
use anyhow::Context;

pub fn get_exercise_page(
    state: SharedState,
    workout_name: &str,
    exercise_name: &str,
) -> Result<String, InternalError> {
    let engine = &state.read().unwrap().engine;
    let program = &state.read().unwrap().program;

    let template = include_str!("../../files/exercise.html");
    let workout = program
        .find(&workout_name)
        .context("failed to find workout")?;
    let exercise = workout
        .find(&ExerciseName(exercise_name.to_owned()))
        .context("failed to find exercise")?;
    let data = ExerciseData::new(workout, exercise);
    Ok(engine
        .render_template(template, &data)
        .context("failed to render template")?)
}

#[derive(Serialize, Deserialize)]
struct ExerciseData {
    workout: String,              // "Full Body Exercises"
    exercise: String,             // "RDL"
    exercise_set: String,         // "Set 1 of 3"
    exercise_set_details: String, // "8 reps @ 135 lbs"
}

impl ExerciseData {
    fn new(w: &Workout, e: &Exercise) -> ExerciseData {
        let workout = w.name.clone();
        let exercise = e.name().0.clone();
        let (exercise_set, exercise_set_details) =
            if let Some((current_set, num_sets)) = e.current_set() {
                let details = match e {
                    Exercise::Durations(_, _, exercise, _) => {
                        format!("{}s", exercise.sets()[current_set as usize])
                    }
                    Exercise::FixedReps(_, _, exercise, _) => {
                        format!("{} reps", exercise.sets()[current_set as usize])
                    }
                };
                (format!("Set {} of {}", current_set + 1, num_sets), details)
            } else {
                ("".to_owned(), "".to_owned())
            };

        ExerciseData {
            workout,
            exercise,
            exercise_set,
            exercise_set_details,
        }
    }
}
