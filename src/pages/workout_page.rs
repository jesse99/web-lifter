use crate::*;
use anyhow::Context;

pub fn get_workout_page(
    state: SharedState,
    workout: &str,
    error: String,
) -> Result<String, InternalError> {
    let handlebars = &state.read().unwrap().handlebars;
    let weights = &state.read().unwrap().user.weights;
    let program = &state.read().unwrap().user.program;

    // Note that MDN recommends against using aria tables, see https://developer.mozilla.org/en-US/docs/Web/Accessibility/ARIA/Roles/table_role
    let template = include_str!("../../files/workout.html");
    let data = WorkoutData::new(weights, program, workout, error)?;
    Ok(handlebars
        .render_template(template, &data)
        .context("failed to render template")?)
}

#[derive(Serialize, Deserialize)]
struct WorkoutData {
    workout_name: String,
    exercises: Vec<ExerciseData>,
    error: String,
}

impl WorkoutData {
    fn new(
        weights: &Weights,
        program: &Program,
        name: &str,
        error: String,
    ) -> Result<WorkoutData, anyhow::Error> {
        if let Some(workout) = program.find(name) {
            let exercises: Vec<ExerciseData> = workout
                .exercises()
                .map(|e| ExerciseData::new(weights, workout, e))
                .collect();
            Ok(WorkoutData {
                workout_name: name.to_owned(),
                exercises: exercises,
                error,
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
    fn new(weights: &Weights, workout: &Workout, exercise: &Exercise) -> ExerciseData {
        ExerciseData {
            workout_name: workout.name.clone(),
            name: exercise.name().0.clone(),
            summary: summarize(weights, exercise),
        }
    }
}

fn summarize(weights: &Weights, exercise: &Exercise) -> String {
    let sets = match exercise {
        // TODO: convert to a short time, eg secs or mins
        Exercise::Durations(_, e) => (0..e.num_sets())
            .map(|i| {
                let index = SetIndex::Workset(i);
                let d = e.set(index);
                let w = exercise.lower_weight(weights, index);
                let suffix = w.map_or("".to_owned(), |w| format!(" @ {}", w.text()));
                format!("{d}s{suffix}")
            })
            .collect(),
        Exercise::FixedReps(_, e) => (0..e.num_worksets())
            .map(|i| {
                let index = SetIndex::Workset(i); // workout page only shows work sets
                let r = e.set(index).reps;
                let w = exercise.lower_weight(weights, index);
                let suffix = w.map_or("".to_owned(), |w| format!(" @ {}", w.text()));
                format!("{r} reps{suffix}")
            })
            .collect(),
        Exercise::VariableReps(_, e) => (0..e.num_worksets())
            .map(|i| {
                let index = SetIndex::Workset(i);
                let r = e.expected_range(index);
                let w = exercise.lower_weight(weights, index);
                let suffix = w.map_or("".to_owned(), |w| format!(" @ {}", w.text()));
                if r.min < r.max {
                    format!("{}-{} reps{suffix}", r.min, r.max)
                } else {
                    format!("{} reps{suffix}", r.max)
                }
            })
            .collect(),
        Exercise::VariableSets(_, e) => {
            let previous = e.get_previous().iter().sum();
            vec![if previous == 0 {
                format!("{} reps over 1+ sets", e.target())
            } else if e.target() == previous {
                format!("{} reps over {} sets", e.target(), e.get_previous().len())
            } else if e.target() > previous {
                format!("{} reps over {}+ sets", e.target(), e.get_previous().len())
            } else {
                format!("{} reps over {} sets", e.target(), e.get_previous().len())
            }]
        }
    };
    join_labels(sets)
}

/// Takes strings like "10s 10s 30s" and converts them into "2x10s, 30s"
pub fn join_labels(labels: Vec<String>) -> String {
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
    parts.join(", ")
}
