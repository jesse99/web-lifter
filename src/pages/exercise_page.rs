use crate::pages::errors::Unwrapper;
use crate::{
    exercise::{Exercise, ExerciseData, ExerciseName, SetIndex, VariableReps},
    history::{CompletedSets, History, Record},
    notes::Notes,
    pages::{self, Error, SharedState},
    program::Program,
    weights::{self, WeightSet, Weights},
    workout::Workout,
};
use chrono::Local;
use serde::{Deserialize, Serialize};

pub fn get_exercise_page(
    state: SharedState,
    workout: &str,
    exercise: &str,
) -> Result<String, Error> {
    let exercise = ExerciseName(exercise.to_owned());
    let reset = reset_old(&state, workout, &exercise.0);
    {
        let program_name = {
            let program = &state.read().unwrap().user.program;
            program.name.to_owned()
        };
        let history = &mut state.write().unwrap().user.history;
        if reset || history.is_completed(&exercise) || !history.has_record(&exercise) {
            history.start(&program_name, workout, &exercise, Local::now());
        }
    }

    let handlebars = &state.read().unwrap().handlebars;
    let weights = &state.read().unwrap().user.weights;
    let notes = &state.read().unwrap().user.notes;
    let history = &state.read().unwrap().user.history;
    let program = &state.read().unwrap().user.program;

    let template = include_str!("../../files/exercise.html");
    let workout = program
        .find(&workout)
        .unwrap_or_err("failed to find workout")?;
    let exercise = workout
        .find(&exercise)
        .unwrap_or_err("failed to find exercise")?;
    let untyped = UntypedData::new(history, exercise);
    let data = ExData::new(
        history,
        notes,
        weights,
        program,
        workout,
        exercise,
        exercise.data(),
        untyped,
    );
    let contents = handlebars.render_template(template, &data)?;
    Ok(contents)
}

fn reset_old(state: &SharedState, workout: &str, exercise: &str) -> bool {
    let program = &mut state.write().unwrap().user.program;
    let workout = program.find_mut(&workout).unwrap();
    let exercise = workout
        .find_mut(&ExerciseName(exercise.to_owned()))
        .unwrap();
    let now = Local::now();
    if let Some(started) = exercise.started() {
        let elapsed = now - started;
        if elapsed.num_minutes() > 60 {
            exercise.reset(Some(now));
            true
        } else {
            false
        }
    } else {
        exercise.reset(Some(now));
        true
    }
}

#[derive(Serialize, Deserialize)]
struct ExerciseDataRecord {
    pub indicator: String,
    pub prefix: String,
    pub kind: String,
    pub label: String,
    pub rid: String,
    pub id: String,
}

#[derive(Serialize, Deserialize)]
struct RepItem {
    pub active: String, // "active" or ""
    pub title: String,  // "4 reps"
}

enum UntypedReps {
    Reps { min: i32, max: i32, unbounded: bool },
    Wait(i32),
}

// Uses for var reps and var sets.
struct Target {
    expected: VariableReps,
    new_reps: bool,
    reached_target: bool,
}

// Allows ExData::new to avoid matching on Exercise.
struct UntypedData {
    kind: String,
    num_warmups: usize,
    num_worksets: usize,
    variable_sets: bool,
    reps: UntypedReps,
    target: Option<Target>,
    weight_details_suffix: String,
}

impl UntypedData {
    fn new(history: &History, exercise: &Exercise) -> UntypedData {
        let d = exercise.data();
        let kind = match exercise {
            Exercise::Durations(_, _) => "durations".to_owned(),
            Exercise::FixedReps(_, _) => "fixed-reps".to_owned(),
            Exercise::VariableReps(_, _) => "var-reps".to_owned(),
            Exercise::VariableSets(_, _) => "var-sets".to_owned(),
        };
        let num_warmups = match exercise {
            Exercise::Durations(_, _) => 0,
            Exercise::FixedReps(_, e) => e.num_warmups(),
            Exercise::VariableReps(_, e) => e.num_warmups(),
            Exercise::VariableSets(_, _) => 0,
        };
        let num_worksets = match exercise {
            Exercise::Durations(_, e) => e.num_sets(),
            Exercise::FixedReps(_, e) => e.num_worksets(),
            Exercise::VariableReps(_, e) => e.num_worksets(),
            Exercise::VariableSets(_, e) => e.get_previous().len(),
        };
        let variable_sets = match exercise {
            Exercise::VariableSets(_, _) => true,
            _ => false,
        };
        let reps = match exercise {
            Exercise::Durations(_, e) => UntypedReps::Wait(e.set(d.current_index)),
            Exercise::FixedReps(_, e) => UntypedReps::Reps {
                min: e.set(d.current_index).reps,
                max: e.set(d.current_index).reps,
                unbounded: false,
            },
            Exercise::VariableReps(_, e) => {
                let range = e.expected_range(d.current_index);
                UntypedReps::Reps {
                    min: range.min,
                    max: range.max,
                    unbounded: false,
                }
            }
            Exercise::VariableSets(_, e) => {
                let previous = e.previous(d.current_index);
                let done: i32 = if d.current_index.index() > 0 {
                    get_var_reps_done(history, exercise.name()).iter().sum()
                } else {
                    0
                };
                let remaining = if done < e.target() {
                    e.target() - done
                } else {
                    0
                };
                if d.finished {
                    UntypedReps::Reps {
                        min: 0,
                        max: 0,
                        unbounded: true,
                    }
                } else if previous == 0 {
                    UntypedReps::Reps {
                        min: remaining,
                        max: remaining,
                        unbounded: true,
                    }
                } else if previous < remaining {
                    UntypedReps::Reps {
                        min: previous,
                        max: previous,
                        unbounded: true,
                    }
                } else if previous == remaining {
                    UntypedReps::Reps {
                        min: remaining,
                        max: remaining,
                        unbounded: false, // sic
                    }
                } else {
                    UntypedReps::Reps {
                        min: remaining,
                        max: remaining,
                        unbounded: true,
                    }
                }
            }
        };
        let target = match exercise {
            Exercise::Durations(_, _) => None,
            Exercise::FixedReps(_, _) => None,
            Exercise::VariableReps(_, e) => {
                let reps = get_var_reps_done(history, exercise.name());
                Some(Target {
                    expected: e.expected_range(d.current_index),
                    new_reps: reps != *e.expected(),
                    reached_target: reps >= e.max_expected(),
                })
            }
            Exercise::VariableSets(_, e) => {
                let previous = e.previous(d.current_index);
                let done: i32 = if d.current_index.index() > 0 {
                    get_var_reps_done(history, exercise.name()).iter().sum()
                } else {
                    0
                };
                let remaining = if done < e.target() {
                    e.target() - done
                } else {
                    0
                };
                let (expected, max) = if previous > 0 {
                    (previous, previous + 5)
                } else if remaining < 5 {
                    (remaining, remaining + 5)
                } else {
                    (5, 12)
                };

                let reps = get_var_reps_done(history, exercise.name());
                Some(Target {
                    expected: VariableReps::new(expected, max, 100),
                    new_reps: reps != *e.get_previous(),
                    reached_target: done >= e.target(),
                })
            }
        };
        let weight_details_suffix = match exercise {
            Exercise::Durations(_, e) => e
                .target()
                .map(|t| format!("target is {t}s"))
                .unwrap_or("".to_owned()),
            _ => "".to_owned(),
        };

        UntypedData {
            kind,
            num_warmups,
            num_worksets,
            variable_sets,
            reps,
            target,
            weight_details_suffix,
        }
    }
}

#[derive(Serialize, Deserialize)]
struct ExData {
    workout: String,  // "Full Body Exercises"
    exercise: String, // "RDL"
    rest: String,     // "" or "30" (seconds)
    records: Vec<ExerciseDataRecord>,
    notes: String,
    edit_weight_url: String,
    disable_edit_weight_set: String,
    edit_weight_set_url: String,

    exercise_set: String,         // "Set 1 of 3"
    exercise_set_details: String, // "8 reps @ 145 lbs"
    weight_details: String,       // "45 + 10 + 5"
    wait: String,                 // "" or "30" (seconds), this is for durations type exercises
    button_title: String,         // "Next", "Start", "Done", "Exit", etc
    hide_reps: String,            // "hidden" or ""
    update_hidden: String,        // "hidden" or ""
    advance_hidden: String,       // "hidden" or ""
    update_value: String,         // "1" or "0"
    advance_value: String,        // "1" or "0"
    reps_title: String,           // "8 reps"
    rep_items: Vec<RepItem>,
    edit_exercise_url: String,
}

impl ExData {
    fn new(
        history: &History,
        notes: &Notes,
        weights: &Weights,
        program: &Program,
        workout: &Workout,
        exercise: &Exercise,
        d: &ExerciseData,
        data: UntypedData,
    ) -> ExData {
        // Below is common to all exercise types.
        let rest = if d.finished {
            "0".to_owned()
        } else {
            match d.current_index {
                SetIndex::Warmup(_) => "0".to_owned(),
                SetIndex::Workset(_) => exercise
                    .rest(d.current_index)
                    .map_or("0".to_owned(), |r| format!("{r}")),
            }
        };
        let records = ExData::get_records(history, program, workout, exercise);
        let notes = notes.html(&d.formal_name);
        let edit_weight_url = if d.weightset.is_some() {
            format!("/edit-weight/{}/{}", workout.name, exercise.name())
        } else {
            format!("/edit-any-weight/{}/{}", workout.name, exercise.name())
        };
        let (disable_edit_weight_set, edit_weight_set_url) = if let Some(name) = &d.weightset {
            match weights.get(&name) {
                Some(WeightSet::Discrete(_)) => (
                    "".to_owned(),
                    format!("/edit-discrete-weight/{}/{}", workout.name, exercise.name()),
                ),
                Some(WeightSet::DualPlates(_, _)) => (
                    "".to_owned(),
                    format!("/edit-plates-weight/{}/{}", workout.name, exercise.name()),
                ),
                None => ("disabled".to_owned(), "#".to_owned()),
            }
        } else {
            ("disabled".to_owned(), "#".to_owned())
        };

        // Below depends on the exercise type.
        let exercise_set = if data.variable_sets {
            if data.num_worksets > 0 && !d.finished {
                format!(
                    "Set {} of {}+",
                    d.current_index.index() + 1,
                    data.num_worksets
                )
            } else {
                format!("Set {}", d.current_index.index() + 1,)
            }
        } else if data.num_warmups > 0 {
            match d.current_index {
                SetIndex::Warmup(i) => format!("Warmup {} of {}", i + 1, data.num_warmups),
                SetIndex::Workset(i) => {
                    format!("Workset {} of {}", i + 1, data.num_worksets)
                }
            }
        } else {
            format!(
                "Set {} of {}",
                d.current_index.index() + 1,
                data.num_worksets
            )
        };

        let w = match d.current_index {
            SetIndex::Warmup(_) => exercise.closest_weight(weights, d.current_index),
            SetIndex::Workset(_) => exercise.lower_weight(weights, d.current_index),
        };
        let suffix = w
            .clone()
            .map_or("".to_owned(), |w| format!(" @ {}", w.text()));
        let exercise_set_details = match data.reps {
            UntypedReps::Reps {
                min,
                max,
                unbounded,
            } => {
                if unbounded {
                    format!("{max}+ reps{suffix}")
                } else if min < max {
                    format!("{min}-{max} reps{suffix}")
                } else if max == 1 {
                    format!("1 rep{suffix}")
                } else {
                    format!("{max} reps{suffix}")
                }
            }
            UntypedReps::Wait(w) => format!("{w}s{suffix}"),
        };

        let wdetails = w
            .clone()
            .map(|w| w.details())
            .flatten()
            .unwrap_or("".to_owned());
        let weight_details = if data.weight_details_suffix.is_empty() {
            wdetails
        } else {
            format!("{wdetails} ({})", data.weight_details_suffix) // kinda lame formatting (tho this will likely be rare)
        };

        let wait = match data.reps {
            UntypedReps::Wait(w) => {
                if d.finished {
                    "0".to_owned()
                } else {
                    format!("{w}")
                }
            }
            _ => "0".to_owned(),
        };

        let button_title = if d.finished {
            "Exit".to_owned()
        } else {
            match data.reps {
                UntypedReps::Wait(_) => "Start".to_owned(),
                _ => {
                    if data.variable_sets {
                        "Next".to_owned()
                    } else {
                        match d.current_index {
                            SetIndex::Warmup(_) => "Next".to_owned(),
                            SetIndex::Workset(i) => {
                                if i + 1 < data.num_worksets {
                                    "Next".to_owned()
                                } else {
                                    "Done".to_owned()
                                }
                            }
                        }
                    }
                }
            }
        };
        let edit_exercise_url = format!("/edit-{}/{}/{}", data.kind, workout.name, exercise.name());

        let mut hide_reps = "hidden".to_owned();
        let mut reps_title = "".to_owned();
        let mut rep_items = Vec::new();
        let mut update_hidden = "hidden".to_owned();
        let mut update_value = "0".to_owned();
        let mut advance_hidden = "hidden".to_owned();
        let mut advance_value = "0".to_owned();
        if let Some(target) = data.target {
            let in_workset = if let SetIndex::Workset(_) = d.current_index {
                true
            } else {
                false
            };
            hide_reps = if d.finished || !in_workset {
                "hidden".to_owned()
            } else {
                "".to_owned()
            };
            reps_title = reps_to_title(target.expected.min);
            rep_items = reps_to_vec(target.expected);

            (update_hidden, update_value) = if d.finished && target.new_reps {
                ("".to_owned(), "1".to_owned())
            } else {
                ("hidden".to_owned(), "0".to_owned())
            };

            (advance_hidden, advance_value) = if d.finished && w.is_some() && target.reached_target
            {
                ("".to_owned(), "0".to_owned())
            } else {
                ("hidden".to_owned(), "0".to_owned())
            };
        }

        ExData {
            workout: workout.name.clone(),
            exercise: exercise.name().0.clone(),
            rest,
            records,
            notes,
            edit_weight_url,
            disable_edit_weight_set,
            edit_weight_set_url,

            exercise_set,
            exercise_set_details,
            weight_details,
            wait,
            button_title,
            edit_exercise_url,

            hide_reps,
            update_hidden,
            advance_hidden,
            update_value,
            advance_value,
            reps_title,
            rep_items,
        }
    }

    fn get_records(
        history: &History,
        program: &Program,
        workout: &Workout,
        exercise: &Exercise,
    ) -> Vec<ExerciseDataRecord> {
        let completed = history
            .records(exercise.name())
            .last()
            .map(|r| r.completed)
            .flatten();
        let in_progress = completed.is_none();
        let records: Vec<&Record> = history
            .records(exercise.name())
            .rev()
            .filter(|r| r.program == program.name && r.workout == workout.name && r.sets.is_some()) // TODO add a way to disable this?
            .take(100) // TODO add a button to pull down another 100 of history?
            .collect();
        records
            .iter()
            .enumerate()
            .map(|(i, r)| record_to_record(get_delta(&records, i), r, i == 0 && in_progress))
            .collect()
    }
}

fn reps_to_title(reps: i32) -> String {
    if reps == 1 {
        "1 rep".to_owned()
    } else {
        format!("{} reps", reps)
    }
}

fn reps_to_vec(reps: VariableReps) -> Vec<RepItem> {
    (1..=reps.max)
        .map(|n| {
            let title = if n == 1 {
                "1 rep".to_owned()
            } else {
                format!("{n} reps")
            };
            let active = if reps.min == n {
                "active".to_owned()
            } else {
                "".to_owned()
            };
            RepItem { title, active }
        })
        .collect()
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
                    let new_reps = aggregate_reps(new_sets);
                    let old_reps = aggregate_reps(old_sets);
                    Some((new_reps, old_reps))
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

fn record_to_record(delta: i32, record: &Record, in_progress: bool) -> ExerciseDataRecord {
    let indicator = if in_progress && record.completed.is_none() {
        "-  ".to_owned()
    } else if delta > 0 {
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
    let mut prefix = record.started.format("%-d %b %Y").to_string();

    let (kind, mut label) = if let Some(ref sets) = record.sets {
        prefix += ", ";
        match sets {
            CompletedSets::Durations(s) => ("durs".to_owned(), durations_to_str(s)),
            CompletedSets::Reps(s) => ("reps".to_owned(), reps_to_str(s)),
        }
    } else {
        ("".to_owned(), "".to_owned())
    };

    if let Some(ref comment) = record.comment {
        label += &format!(", {comment}")
    };

    let id = if delta > 0 && !in_progress {
        "better_record".to_owned()
    } else if delta < 0 && !in_progress {
        "worse_record".to_owned()
    } else {
        "same_record".to_owned()
    };

    ExerciseDataRecord {
        indicator,
        prefix,
        kind,
        label,
        rid: format!("{}", record.id),
        id,
    }
}

fn durations_to_str(sets: &Vec<(i32, Option<f32>)>) -> String {
    num_to_str(sets, "secs") // TODO will need to also pass in a fn so we can get short times
}

fn reps_to_str(sets: &Vec<(i32, Option<f32>)>) -> String {
    num_to_str(sets, "reps")
}

fn num_to_str(sets: &Vec<(i32, Option<f32>)>, unit: &str) -> String {
    if sets.iter().all(|s| s.1.is_none()) {
        let reps: Vec<_> = sets.iter().map(|x| format!("{}", x.0)).collect();
        let reps = pages::join_labels(reps);
        reps + " " + unit
    } else if !sets.is_empty() && sets[0].1.is_some() && sets.iter().all(|s| s.1 == sets[0].1) {
        let reps: Vec<_> = sets.iter().map(|x| format!("{}", x.0)).collect();
        let reps = pages::join_labels(reps);
        let weight = weights::format_weight(sets[0].1.unwrap(), " lbs");
        format!("{reps} {unit} @ {weight}")
    } else {
        pages::join_labels(
            sets.iter()
                .map(|x| {
                    let reps = format!("{}", x.0);
                    if let Some(weight) = x.1 {
                        let weight = weights::format_weight(weight, " lbs");
                        format!("{reps} @ {weight}")
                    } else {
                        reps
                    }
                })
                .collect(),
        )
    }
}

pub fn get_var_reps_done(history: &History, name: &ExerciseName) -> Vec<i32> {
    let last = history.records(name).last().map_or(&None, |r| &r.sets);
    match last {
        Some(CompletedSets::Reps(v)) => v.iter().map(|t| t.0).collect(),
        _ => Vec::new(),
    }
}
