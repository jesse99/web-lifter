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

pub fn get_next_exercise_page(
    mut state: SharedState,
    workout_name: &str,
    exercise_name: &str,
) -> Result<String, InternalError> {
    let current = {
        let program = &state.read().unwrap().program;
        let workout = program
            .find(&workout_name)
            .context("failed to find workout")?;
        let exercise = workout
            .find(&ExerciseName(exercise_name.to_owned()))
            .context("failed to find exercise")?;
        exercise.current_set()
    };
    if let Some((current_set, num_sets)) = current {
        let next_set = current_set + 1;
        if next_set < num_sets {
            advance_set(&mut state, workout_name, exercise_name);
            get_exercise_page(state, workout_name, exercise_name)
        } else {
            // TODO save history
            get_workout_page(state, workout_name)
        }
    } else {
        // TODO save history
        get_workout_page(state, workout_name)
    }
}

fn advance_set(state: &mut SharedState, workout_name: &str, exercise_name: &str) {
    let program = &mut state.write().unwrap().program;
    let workout = program.find_mut(&workout_name).unwrap();
    let exercise = workout
        .find(&ExerciseName(exercise_name.to_owned()))
        .unwrap();
    match exercise {
        Exercise::Durations(n, f, e, s) => {
            let mut s = s.clone(); // TODO kinda inefficient, maybe we can do something with swap?
            s.current_set += 1;
            workout.update(Exercise::Durations(n.clone(), f.clone(), e.clone(), s));
        }
        Exercise::FixedReps(n, f, e, s) => {
            let mut s = s.clone();
            s.current_set += 1;
            workout.update(Exercise::FixedReps(n.clone(), f.clone(), e.clone(), s));
        }
    }
}

#[derive(Serialize, Deserialize)]
struct ExerciseData {
    workout: String,              // "Full Body Exercises"
    exercise: String,             // "RDL"
    exercise_set: String,         // "Set 1 of 3"
    exercise_set_details: String, // "8 reps @ 135 lbs"
    rest: String,                 // "" or "30" (seconds)
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
        let rest = e.rest().map_or("".to_owned(), |r| format!("{r}"));

        ExerciseData {
            workout,
            exercise,
            exercise_set,
            exercise_set_details,
            rest,
        }
    }
}
