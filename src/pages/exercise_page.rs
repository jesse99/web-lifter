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
            let set = instance.current_set();
            (
                format!("Set {} of {}", set.current_set + 1, set.num_sets),
                set.current_set as usize,
            )
        } else {
            anyhow::bail!("Failed to find a instance named '{exercise_name}'")
        };

        let exercise_set_details = if let Some(exercise) = exercises.find(&name) {
            match exercise {
                Exercise::Durations(exercise) => format!("{}s", exercise.sets()[index]),
                Exercise::FixedReps(exercise) => format!("{} reps", exercise.sets()[index]),
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
