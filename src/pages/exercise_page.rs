use std::iter;

use crate::*;
use anyhow::Context;
use chrono::Utc;

pub fn get_exercise_page(
    state: SharedState,
    workout_name: &str,
    exercise_name: &str,
) -> Result<String, InternalError> {
    let engine = &state.read().unwrap().engine;
    let history = &state.read().unwrap().history;
    let program = &state.read().unwrap().program;

    let template = include_str!("../../files/exercise.html");
    let workout = program
        .find(&workout_name)
        .context("failed to find workout")?;
    let exercise = workout
        .find(&ExerciseName(exercise_name.to_owned()))
        .context("failed to find exercise")?;
    let data = ExerciseData::new(history, program, workout, exercise);
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
            complete_set(&mut state, workout_name, exercise_name);
            get_workout_page(state, workout_name)
        }
    } else {
        complete_non_set(&mut state, workout_name, exercise_name);
        get_workout_page(state, workout_name)
    }
}

fn advance_set(state: &mut SharedState, workout_name: &str, exercise_name: &str) {
    let program = &mut state.write().unwrap().program;
    let workout = program.find_mut(&workout_name).unwrap();
    let exercise = workout
        .find_mut(&ExerciseName(exercise_name.to_owned()))
        .unwrap();
    match exercise {
        Exercise::Durations(_, _, _, s) => {
            s.current_set += 1;
        }
        Exercise::FixedReps(_, _, _, s) => {
            s.current_set += 1;
        }
    }
}

fn complete_set(state: &mut SharedState, workout_name: &str, exercise_name: &str) {
    let exercise_name = ExerciseName(exercise_name.to_owned());
    let record = {
        let program = &state.read().unwrap().program;
        let workout = program.find(&workout_name).unwrap();
        let exercise = workout.find(&exercise_name).unwrap();

        let sets = match exercise {
            Exercise::Durations(_, _, e, s) => {
                if let Some(ref w) = s.weight {
                    assert!(e.sets().len() == w.len());
                    CompletedSets::Durations(
                        iter::zip(e.sets().iter().copied(), w.iter().copied().map(|x| Some(x)))
                            .collect(),
                    )
                } else {
                    let n = e.sets().len();
                    CompletedSets::Durations(
                        iter::zip(e.sets().iter().copied(), iter::repeat(None))
                            .take(n)
                            .collect(),
                    )
                }
            }
            Exercise::FixedReps(_, _, e, s) => {
                if let Some(ref w) = s.weight {
                    assert!(e.sets().len() == w.len());
                    CompletedSets::Reps(
                        iter::zip(e.sets().iter().copied(), w.iter().copied().map(|x| Some(x)))
                            .collect(),
                    )
                } else {
                    let n = e.sets().len();
                    CompletedSets::Reps(
                        iter::zip(e.sets().iter().copied(), iter::repeat(None).take(n)).collect(),
                    )
                }
            }
        };
        Record {
            program: program.name.clone(),
            workout: workout_name.to_owned(),
            date: Utc::now(),
            sets: Some(sets),
            comment: None,
        }
    };

    {
        let program = &mut state.write().unwrap().program;
        let workout = program.find_mut(&workout_name).unwrap();
        let exercise = workout.find_mut(&exercise_name).unwrap();

        match exercise {
            Exercise::Durations(_, _, _, s) => {
                s.current_set = 0;
            }
            Exercise::FixedReps(_, _, _, s) => {
                s.current_set = 0;
            }
        }
    }

    let history = &mut state.write().unwrap().history;
    history.add(&exercise_name, record);
}

fn complete_non_set(state: &mut SharedState, workout_name: &str, exercise_name: &str) {
    let exercise_name = ExerciseName(exercise_name.to_owned());

    let record = {
        let program = &state.read().unwrap().program;
        Record {
            program: program.name.clone(),
            workout: workout_name.to_owned(),
            date: Utc::now(),
            sets: None, // TODO these will probably support (a) weight
            comment: None,
        }
    };

    let history = &mut state.write().unwrap().history;
    history.add(&exercise_name, record);
}

#[derive(Serialize, Deserialize)]
struct ExerciseData {
    workout: String,              // "Full Body Exercises"
    exercise: String,             // "RDL"
    exercise_set: String,         // "Set 1 of 3"
    exercise_set_details: String, // "8 reps @ 135 lbs"
    rest: String,                 // "" or "30" (seconds)
    records: Vec<String>,
}

impl ExerciseData {
    fn new(history: &History, program: &Program, w: &Workout, e: &Exercise) -> ExerciseData {
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

        let records = history
            .records(e.name())
            .rev()
            .filter(|r| r.program == program.name && r.workout == workout) // TODO add a way to disable this?
            .take(100) // TODO add a button to pull down another 100 of history?
            .map(|r| record_to_str(r))
            .collect();

        ExerciseData {
            workout,
            exercise,
            exercise_set,
            exercise_set_details,
            rest,
            records,
        }
    }
}

fn record_to_str(record: &Record) -> String {
    let mut result = String::new();

    // TODO
    // do we want to use stuff like "today", "yesterday", "3 days ago"?
    // wouldn't that get weird for stuff further back?
    // make this a setting?
    result += &record.date.format("%-d %b %Y").to_string();

    if let Some(ref sets) = record.sets {
        let labels: Vec<String> = match sets {
            CompletedSets::Durations(s) => s.iter().map(|x| durations_to_str(x)).collect(),
            CompletedSets::Reps(s) => s.iter().map(|x| reps_to_str(x)).collect(),
        };
        result += ", ";
        result += &join_labels(labels);
    }

    if let Some(ref comment) = record.comment {
        result += ", ";
        result += comment;
    }
    result
}

fn durations_to_str(entry: &(i32, Option<f32>)) -> String {
    if let Some(w) = entry.1 {
        format!("{} secs @ {:.1} lbs", entry.0, w) // TODO use a short or friendly time function, also a weight fn
    } else {
        format!("{} secs", entry.0)
    }
}

fn reps_to_str(entry: &(i32, Option<f32>)) -> String {
    if let Some(w) = entry.1 {
        format!("{} reps @ {:.1} lbs", entry.0, w) // TODO use a weight fn
    } else {
        format!("{} reps", entry.0)
    }
}
