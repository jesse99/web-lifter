use gear_objects::find_trait;
use paste::paste;

use crate::*;

pub fn get_exercise_page(
    state: SharedState,
    workout_name: &str,
    exercise_name: &str,
) -> Result<String, InternalError> {
    let engine = &state.read().unwrap().engine;
    let program = &state.read().unwrap().program;
    let exercises = &state.read().unwrap().exercises;

    let template = include_str!("../../files/exercise.html");
    let workout = program.find(&workout_name).unwrap();
    let data = ExerciseData::new(
        workout,
        exercises,
        workout_name.to_owned(),
        exercise_name.to_owned(),
    )?;
    Ok(engine.render_template(template, &data).unwrap())
}

#[derive(Serialize, Deserialize)]
struct ExerciseData {
    workout: String,              // "Full Body Exercises"
    exercise: String,             // "RDL"
    exercise_set: String,         // "Set 1 of 3"
    exercise_set_details: String, // "8 reps @ 135 lbs"
}

impl ExerciseData {
    fn new(
        workout: &Workout,
        exercises: &Exercises,
        workout_name: String,
        exercise_name: String,
    ) -> Result<ExerciseData, anyhow::Error> {
        let name = ExerciseName(exercise_name.clone());
        let (exercise_set, index) = if let Some(instance) = workout.find(&name) {
            if let Some(set) = find_trait!(instance, ISetDetails) {
                let details = set.expected();
                (
                    format!("Set {} of {}", details.index + 1, details.count),
                    details.index as usize,
                )
            } else {
                anyhow::bail!("Failed to find ISetDetails for instance '{exercise_name}'")
            }
        } else {
            anyhow::bail!("Failed to find a instance named '{exercise_name}'")
        };

        let exercise_set_details = if let Some(exercise) = exercises.find(&name) {
            if let Some(durations) = find_trait!(exercise, IDurations) {
                format!("{}s", durations.expected()[index])
            } else if let Some(reps) = find_trait!(exercise, IFixedReps) {
                format!("{} reps", reps.expected()[index])
            } else {
                anyhow::bail!("couldn't find sets for {name}")
            }
        } else {
            anyhow::bail!("Failed to find a exercise named '{exercise_name}'")
        };

        Ok(ExerciseData {
            workout: workout_name,
            exercise: exercise_name,
            exercise_set,
            exercise_set_details,
        })
    }
}
