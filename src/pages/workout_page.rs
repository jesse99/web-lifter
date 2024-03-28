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
    let history = &state.read().unwrap().user.history;

    // Note that MDN recommends against using aria tables, see https://developer.mozilla.org/en-US/docs/Web/Accessibility/ARIA/Roles/table_role
    let template = include_str!("../../files/workout.html");
    let data = WorkoutData::new(history, weights, program, workout, error)?;
    Ok(handlebars
        .render_template(template, &data)
        .context("failed to render template")?)
}

#[derive(Serialize, Deserialize)]
struct WorkoutData {
    workout_name: String,
    exercises: Vec<ExerciseData>,
    total_duration: String,
    error: String,
}

impl WorkoutData {
    fn new(
        history: &History,
        weights: &Weights,
        program: &Program,
        name: &str,
        error: String,
    ) -> Result<WorkoutData, anyhow::Error> {
        if let Some(workout) = program.find(name) {
            let exercises: Vec<ExerciseData> = workout
                .exercises()
                .map(|e| ExerciseData::new(history, weights, workout, e))
                .collect();
            let total_duration = if let Some(started) = history.first_started(name) {
                if let Some(finished) = history.last_completed(name) {
                    let delta = finished - started;
                    let mins = delta.num_minutes();
                    if mins > 60 {
                        format!("{:.1} hours", (mins as f32) / 60.0)
                    } else {
                        format!("{mins} mins")
                    }
                } else {
                    "-".to_owned()
                }
            } else {
                "-".to_owned()
            };
            Ok(WorkoutData {
                workout_name: name.to_owned(),
                exercises: exercises,
                total_duration,
                error,
            })
        } else {
            anyhow::bail!("Failed to find a workout named '{name}'");
        }
    }
}

#[derive(Serialize, Deserialize)]
struct ExerciseData {
    color: String,
    workout_name: String,
    name: String,
    summary: String,
    duration: String,
}

impl ExerciseData {
    fn new(
        history: &History,
        weights: &Weights,
        workout: &Workout,
        exercise: &Exercise,
    ) -> ExerciseData {
        let (color, duration) =
            if let Some(record) = history.recently_completed(&workout.name, exercise.name()) {
                let started = record.started;
                let completed = record.completed.unwrap();
                let s = (completed - started).num_seconds();
                let m = (completed - started).num_minutes();
                let mins = if s == 0 {
                    "".to_owned() // history before we actually had completed
                } else if m == 0 {
                    format!("{s} secs")
                } else if m == 1 {
                    "1 min".to_owned()
                } else {
                    format!("{m} mins")
                };
                ("text-secondary".to_owned(), mins)
            } else {
                ("".to_owned(), "-".to_owned())
            };
        ExerciseData {
            color,
            workout_name: workout.name.clone(),
            name: exercise.name().0.clone(),
            summary: summarize(weights, exercise),
            duration,
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
