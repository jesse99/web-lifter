use crate::*;
use anyhow::Context;
use std::iter;

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
    let set_state = {
        let program = &state.read().unwrap().program;
        let workout = program
            .find(&workout_name)
            .context("failed to find workout")?;
        let exercise = workout
            .find(&ExerciseName(exercise_name.to_owned()))
            .context("failed to find exercise")?;
        match exercise {
            Exercise::Durations(_, _, _, s) => s.state,
            Exercise::FixedReps(_, _, _, s) => s.state,
        }
    };

    if set_state == SetState::Finished {
        complete_set(&mut state, workout_name, exercise_name);
        get_workout_page(state, workout_name)
    } else {
        advance_set(&mut state, workout_name, exercise_name);
        get_exercise_page(state, workout_name, exercise_name)
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
            if s.current_set + 1 == s.num_sets {
                s.state = SetState::Finished
            } else {
                s.current_set += 1;
            }
        }
        Exercise::FixedReps(_, _, _, s) => {
            if s.current_set + 1 == s.num_sets {
                s.state = SetState::Finished
            } else {
                s.current_set += 1;
            }
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
                s.state = SetState::Timed;
            }
            Exercise::FixedReps(_, _, _, s) => {
                s.current_set = 0;
                s.state = SetState::Implicit;
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
struct ExerciseDataRecord {
    pub indicator: String,
    pub text: String,
    pub id: String,
}

#[derive(Serialize, Deserialize)]
struct ExerciseData {
    workout: String,              // "Full Body Exercises"
    exercise: String,             // "RDL"
    exercise_set: String,         // "Set 1 of 3"
    exercise_set_details: String, // "8 reps @ 135 lbs"
    wait: String,                 // "" or "30" (seconds), this is for durations type exercises
    rest: String,                 // "" or "30" (seconds)
    button_title: String,         // "Next", "Start", "Done", "Exit", etc
    records: Vec<ExerciseDataRecord>,
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

        let (state, current_set, num_sets) = match e {
            Exercise::Durations(_, _, _, s) => (s.state, s.current_set, s.num_sets),
            Exercise::FixedReps(_, _, _, s) => (s.state, s.current_set, s.num_sets),
        };
        let wait = if let SetState::Finished = state {
            "0".to_owned()
        } else {
            match e {
                Exercise::Durations(_, _, e, _) => format!("{}", e.sets()[current_set as usize]),
                Exercise::FixedReps(_, _, _, _) => "0".to_owned(),
            }
        };
        let rest = match state {
            SetState::Finished => "0".to_owned(),
            _ => e.rest().map_or("".to_owned(), |r| format!("{r}")),
        };
        // TODO: if current_set + 1 == num_sets then need to show reps stepper (if variable reps)
        let button_title = match state {
            SetState::Implicit => {
                if current_set + 1 < num_sets {
                    "Next".to_owned()
                } else {
                    "Done".to_owned()
                }
            }
            SetState::Timed => "Start".to_owned(),
            SetState::Finished => "Exit".to_owned(),
        };
        let records0: Vec<&Record> = history
            .records(e.name())
            .rev()
            .filter(|r| r.program == program.name && r.workout == workout) // TODO add a way to disable this?
            .take(100) // TODO add a button to pull down another 100 of history?
            .collect();
        let records = records0
            .iter()
            .enumerate()
            .map(|(i, r)| (get_delta(&records0, i), r))
            .map(|(d, r)| record_to_record(d, r))
            .collect();

        ExerciseData {
            workout,
            exercise,
            exercise_set,
            exercise_set_details,
            wait,
            rest,
            records,
            button_title,
        }
    }
}

// Note that here records goes from newest to oldest.
fn get_delta(records: &Vec<&Record>, i: usize) -> i32 {
    if i + 1 < records.len() {
        let older = records[i + 1];
        let newer = records[i];
        let values = match newer.sets {
            Some(CompletedSets::Durations(ref new_sets)) => match older.sets {
                Some(CompletedSets::Durations(ref old_sets)) => {
                    let new_durations = aggregate_durations(new_sets);
                    let old_durations = aggregate_durations(old_sets);
                    Some((new_durations, old_durations))
                }
                Some(CompletedSets::Reps(_)) => None,
                None => None, // in theory we can get a mismatch if the user keeps an exercise name but changes the exercise type
            },
            Some(CompletedSets::Reps(ref new_sets)) => match older.sets {
                Some(CompletedSets::Durations(_)) => None,
                Some(CompletedSets::Reps(ref old_sets)) => {
                    let new_durations = aggregate_reps(new_sets);
                    let old_durations = aggregate_reps(old_sets);
                    Some((new_durations, old_durations))
                }
                None => None,
            },
            None => None,
        };
        if let Some((new_value, old_value)) = values {
            if new_value > old_value {
                return 1;
            } else if new_value < old_value {
                return -1;
            }
        }
    }
    0
}

fn aggregate_durations(sets: &Vec<(i32, Option<f32>)>) -> f32 {
    sets.iter().fold(0.0, |sum, x| match x {
        (duration, Some(weight)) => sum + (*duration as f32) * weight,
        (duration, None) => sum + (*duration as f32),
    })
}

fn aggregate_reps(sets: &Vec<(i32, Option<f32>)>) -> f32 {
    sets.iter().fold(0.0, |sum, x| match x {
        (reps, Some(weight)) => sum + (*reps as f32) * weight,
        (reps, None) => sum + (*reps as f32),
    })
}

fn record_to_record(delta: i32, record: &Record) -> ExerciseDataRecord {
    let mut text = String::new();

    let indicator = if delta > 0 {
        "▲ ".to_owned() // BLACK UP-POINTING TRIANGLE
    } else if delta < 0 {
        "▼ ".to_owned() // BLACK DOWN-POINTING TRIANGLE
    } else {
        "✸ ".to_owned() // HEAVY EIGHT POINTED RECTILINEAR BLACK STAR
    };

    // TODO
    // do we want to use stuff like "today", "yesterday", "3 days ago"?
    // wouldn't that get weird for stuff further back?
    // make this a setting?
    text += &record.date.format("%-d %b %Y").to_string();

    if let Some(ref sets) = record.sets {
        let labels: Vec<String> = match sets {
            CompletedSets::Durations(s) => s.iter().map(|x| durations_to_str(x)).collect(),
            CompletedSets::Reps(s) => s.iter().map(|x| reps_to_str(x)).collect(),
        };
        text += ", ";
        text += &join_labels(labels);
    }

    if let Some(ref comment) = record.comment {
        text += ", ";
        text += comment;
    }

    let id = if delta > 0 {
        "better_record".to_owned()
    } else if delta < 0 {
        "worse_record".to_owned()
    } else {
        "same_record".to_owned()
    };

    ExerciseDataRecord {
        indicator,
        text,
        id,
    }
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
