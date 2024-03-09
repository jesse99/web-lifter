use crate::*;

pub fn get_workout_page(state: SharedState, workout: &str) -> Result<String, InternalError> {
    let engine = &state.read().unwrap().engine;
    let program = &state.read().unwrap().program;
    let exercises = &state.read().unwrap().exercises;

    // Note that MDN recommends against using aria tables, see https://developer.mozilla.org/en-US/docs/Web/Accessibility/ARIA/Roles/table_role
    let template = include_str!("../../files/workout.html");
    let data = WorkoutData::new(program, exercises, workout)?;
    Ok(engine.render_template(template, &data).unwrap())
}

#[derive(Serialize, Deserialize)]
struct WorkoutData {
    workout_name: String,
    exercises: Vec<ExerciseData>,
}

impl WorkoutData {
    fn new(
        program: &Program,
        exercises: &Exercises,
        name: &str,
    ) -> Result<WorkoutData, anyhow::Error> {
        if let Some(workout) = program.find(name) {
            let exercises: Result<Vec<ExerciseData>, anyhow::Error> = workout
                .instances()
                .map(|n| ExerciseData::new(name.to_owned(), n, exercises))
                .collect();
            Ok(WorkoutData {
                workout_name: name.to_owned(),
                exercises: exercises?,
            })
        } else {
            anyhow::bail!("Failed to find a workout named '{name}'");
        }
    }
}

#[derive(Serialize, Deserialize)]
struct ExerciseData {
    workout_name: String,
    name: String,
    summary: String,
}

impl ExerciseData {
    fn new(
        workout_name: String,
        name: &ExerciseName,
        exercises: &Exercises,
    ) -> Result<ExerciseData, anyhow::Error> {
        Ok(ExerciseData {
            workout_name,
            name: name.0.clone(),
            summary: summarize(exercises, name)?,
        })
    }
}

fn summarize(exercises: &Exercises, name: &ExerciseName) -> Result<String, anyhow::Error> {
    if let Some(exercise) = exercises.find(name) {
        let sets = match exercise {
            Exercise::Durations(exercise) => {
                exercise.sets().iter().map(|d| format!("{d}s")).collect()
            } // TODO: convert to a short time, eg secs or mins,
            Exercise::FixedReps(exercise) => exercise
                .sets()
                .iter()
                .map(|d| format!("{d} reps"))
                .collect(),
        };
        Ok(join_labels(sets))
    } else {
        anyhow::bail!("couldn't find execise {name}")
    }
}

/// Takes strings like "10s 10s 30s" and converts them into "2x10s, 30s"
fn join_labels(labels: Vec<String>) -> String {
    let mut parts: Vec<(i32, String)> = Vec::new();

    for label in labels {
        if let Some(last) = parts.last_mut() {
            if last.1 == *label {
                last.0 = last.0 + 1;
            } else {
                parts.push((1, label));
            }
        } else {
            parts.push((1, label));
        }
    }

    let parts: Vec<String> = parts
        .iter()
        .map(|p| {
            if p.0 > 1 {
                format!("{}x{}", p.0, p.1)
            } else {
                p.1.clone()
            }
        })
        .collect();
    parts.join(" ")
}
