use crate::*;
use anyhow::Context;

pub fn get_workout_page(state: SharedState, workout: &str) -> Result<String, InternalError> {
    let engine = &state.read().unwrap().engine;
    let program = &state.read().unwrap().program;

    // Note that MDN recommends against using aria tables, see https://developer.mozilla.org/en-US/docs/Web/Accessibility/ARIA/Roles/table_role
    let template = include_str!("../../files/workout.html");
    let data = WorkoutData::new(program, workout)?;
    Ok(engine
        .render_template(template, &data)
        .context("failed to render template")?)
}

#[derive(Serialize, Deserialize)]
struct WorkoutData {
    workout_name: String,
    exercises: Vec<ExerciseData>,
}

impl WorkoutData {
    fn new(program: &Program, name: &str) -> Result<WorkoutData, anyhow::Error> {
        if let Some(workout) = program.find(name) {
            let exercises: Vec<ExerciseData> = workout
                .exercises()
                .map(|e| ExerciseData::new(workout, e))
                .collect();
            Ok(WorkoutData {
                workout_name: name.to_owned(),
                exercises: exercises,
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
    fn new(workout: &Workout, exercise: &Exercise) -> ExerciseData {
        ExerciseData {
            workout_name: workout.name.clone(),
            name: exercise.name().0.clone(),
            summary: summarize(exercise),
        }
    }
}

fn summarize(exercise: &Exercise) -> String {
    let sets = match exercise {
        // TODO: convert to a short time, eg secs or mins
        Exercise::Durations(_, _, exercise, _) => {
            exercise.sets().iter().map(|d| format!("{d}s")).collect()
        }
        Exercise::FixedReps(_, _, exercise, _) => exercise
            .sets()
            .iter()
            .map(|d| format!("{d} reps"))
            .collect(),
    };
    join_labels(sets)
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
