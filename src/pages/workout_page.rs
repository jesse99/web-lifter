use crate::*;

pub fn get_workout_page(state: SharedState, workout: &str) -> String {
    let engine = &state.read().unwrap().engine;
    let program = &state.read().unwrap().program;
    let exercises = &state.read().unwrap().exercises;

    // Note that MDN recommends against using aria tables, see https://developer.mozilla.org/en-US/docs/Web/Accessibility/ARIA/Roles/table_role
    let template = include_str!("../../files/workout.html");
    let data = WorkoutData::new(program, exercises, workout);
    engine.render_template(template, &data).unwrap()
}

#[derive(Serialize, Deserialize)]
struct WorkoutData {
    name: String,
    exercises: Vec<ExerciseData>,
}

impl WorkoutData {
    fn new(program: &Program, exercises: &Exercises, name: &str) -> WorkoutData {
        let workout = program.find(name).unwrap(); // TODO: don't use unwrap
        let exercises = workout
            .exercises()
            .map(|n| ExerciseData::new(n, exercises))
            .collect();
        WorkoutData {
            name: name.to_owned(),
            exercises,
        }
    }
}

#[derive(Serialize, Deserialize)]
struct ExerciseData {
    name: String,
    summary: String,
}

impl ExerciseData {
    fn new(name: &ExerciseName, exercises: &Exercises) -> ExerciseData {
        let exercise = exercises.find(name).unwrap(); // this unwrap should be OK
        ExerciseData {
            name: name.0.clone(),
            summary: exercise.summary().clone(),
        }
    }
}
