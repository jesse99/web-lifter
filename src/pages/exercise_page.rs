use crate::*;
use anyhow::Context;

pub fn get_exercise_page(
    state: SharedState,
    workout_name: &str,
    exercise_name: &str,
) -> Result<String, InternalError> {
    fn get_new_record(state: &SharedState, workout_name: &str) -> Record {
        let program = &state.read().unwrap().user.program;

        Record {
            program: program.name.clone(),
            workout: workout_name.to_owned(),
            started: Local::now(),
            completed: None,
            sets: None,
            comment: None,
        }
    }

    let exercise_name = ExerciseName(exercise_name.to_owned());
    let reset = reset_old(&state, workout_name, &exercise_name.0);
    {
        let record = get_new_record(&state, workout_name);
        let history = &mut state.write().unwrap().user.history;
        if reset || history.is_completed(&exercise_name) {
            history.start(&exercise_name, record);
        }
    }

    let handlebars = &state.read().unwrap().handlebars;
    let weights = &state.read().unwrap().user.weights;
    let notes = &state.read().unwrap().user.notes;
    let history = &state.read().unwrap().user.history;
    let program = &state.read().unwrap().user.program;

    let template = include_str!("../../files/exercise.html");
    let workout = program
        .find(&workout_name)
        .context("failed to find workout")?;
    let exercise = workout
        .find(&exercise_name)
        .context("failed to find exercise")?;
    let data = ExerciseData::new(weights, notes, history, program, workout, exercise);
    Ok(handlebars
        .render_template(template, &data)
        .context("failed to render template")?)
}

pub fn post_next_exercise_page(
    mut state: SharedState,
    workout_name: &str,
    exercise_name: &str,
    options: Option<VarRepsOptions>,
) -> Result<String, InternalError> {
    let finished = {
        let program = &state.read().unwrap().user.program;
        let workout = program
            .find(&workout_name)
            .context("failed to find workout")?;
        let exercise = workout
            .find(&ExerciseName(exercise_name.to_owned()))
            .context("failed to find exercise")?;
        match exercise {
            Exercise::Durations(d, _) => d.finished,
            Exercise::FixedReps(d, _) => d.finished,
            Exercise::VariableReps(d, _) => d.finished,
            Exercise::VariableSets(d, _) => d.finished,
        }
    };

    let error = if finished {
        let user = &state.read().unwrap().user; // TODO pass error to workout page?
        match persist::save(user) {
            Ok(_) => String::new(),
            Err(e) => format!("{e}"),
        }
    } else {
        String::new()
    };

    if finished {
        complete_set(&mut state, workout_name, exercise_name, options);
        get_workout_page(state, workout_name, error)
    } else {
        advance_set(&mut state, workout_name, exercise_name, options);
        get_exercise_page(state, workout_name, exercise_name)
    }
}

fn reset_old(state: &SharedState, workout_name: &str, exercise_name: &str) -> bool {
    let program = &mut state.write().unwrap().user.program;
    let workout = program.find_mut(&workout_name).unwrap();
    let exercise = workout
        .find_mut(&ExerciseName(exercise_name.to_owned()))
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

fn advance_set(
    state: &mut SharedState,
    workout_name: &str,
    exercise_name: &str,
    options: Option<VarRepsOptions>,
) {
    fn in_workset(state: &mut SharedState, workout_name: &str, exercise_name: &str) -> bool {
        let program = &state.read().unwrap().user.program;
        let workout = program.find(&workout_name).unwrap();
        let exercise = workout
            .find(&ExerciseName(exercise_name.to_owned()))
            .unwrap();
        match exercise {
            Exercise::Durations(_, _) => true,
            Exercise::FixedReps(d, _) => match d.current_index {
                SetIndex::Workset(_) => true,
                _ => false,
            },
            Exercise::VariableReps(d, _) => match d.current_index {
                SetIndex::Workset(_) => true,
                _ => false,
            },
            Exercise::VariableSets(_, _) => true,
        }
    }

    fn advance_current(state: &mut SharedState, workout_name: &str, exercise_name: &str) {
        let name = ExerciseName(exercise_name.to_owned());
        let var_sets_done: i32 = {
            let history = &state.read().unwrap().user.history;
            get_var_reps_done(history, &name).iter().sum() // ok to call this if not var sets
        };

        let program = &mut state.write().unwrap().user.program;
        let workout = program.find_mut(&workout_name).unwrap();
        let exercise = workout
            .find_mut(&ExerciseName(exercise_name.to_owned()))
            .unwrap();
        match exercise {
            Exercise::Durations(d, e) => match d.current_index {
                SetIndex::Workset(i) => {
                    if i + 1 == e.num_sets() {
                        d.finished = true
                    } else {
                        d.current_index = SetIndex::Workset(i + 1);
                    }
                }
                _ => panic!("Expected workset"),
            },
            Exercise::FixedReps(d, e) => match d.current_index {
                SetIndex::Warmup(i) => {
                    if i + 1 == e.num_warmups() {
                        d.current_index = SetIndex::Workset(0);
                    } else {
                        d.current_index = SetIndex::Warmup(i + 1);
                    }
                }
                SetIndex::Workset(i) => {
                    if i + 1 == e.num_worksets() {
                        d.finished = true
                    } else {
                        d.current_index = SetIndex::Workset(i + 1);
                    }
                }
            },
            Exercise::VariableReps(d, e) => match d.current_index {
                SetIndex::Warmup(i) => {
                    if i + 1 == e.num_warmups() {
                        d.current_index = SetIndex::Workset(0);
                    } else {
                        d.current_index = SetIndex::Warmup(i + 1);
                    }
                }
                SetIndex::Workset(i) => {
                    if i + 1 == e.num_worksets() {
                        d.finished = true
                    } else {
                        d.current_index = SetIndex::Workset(i + 1);
                    }
                }
            },
            Exercise::VariableSets(d, e) => match d.current_index {
                SetIndex::Workset(i) => {
                    if var_sets_done >= e.target() {
                        d.finished = true;
                    } else {
                        d.current_index = SetIndex::Workset(i + 1);
                    }
                }
                _ => panic!("Expected workset"),
            },
        }
    }

    fn append_result(
        state: &mut SharedState,
        workout_name: &str,
        exercise_name: &str,
        options: Option<VarRepsOptions>,
    ) {
        let name = ExerciseName(exercise_name.to_owned());
        let (duration, reps, weight) = {
            let weights = &state.read().unwrap().user.weights;
            let program = &state.read().unwrap().user.program;
            let workout = program.find(&workout_name).unwrap();
            let exercise = workout.find(&name).unwrap();
            match exercise {
                Exercise::Durations(d, e) => (
                    Some(e.set(d.current_index)),
                    None,
                    exercise.closest_weight(weights, d.current_index),
                ),
                Exercise::FixedReps(d, e) => (
                    None,
                    Some(e.set(d.current_index).reps),
                    match d.current_index {
                        SetIndex::Warmup(_) => exercise.closest_weight(weights, d.current_index),
                        SetIndex::Workset(_) => exercise.lower_weight(weights, d.current_index),
                    },
                ),
                Exercise::VariableReps(d, _) => (
                    None,
                    options.map(|o| o.reps),
                    match d.current_index {
                        SetIndex::Warmup(_) => exercise.closest_weight(weights, d.current_index),
                        SetIndex::Workset(_) => exercise.lower_weight(weights, d.current_index),
                    },
                ),
                Exercise::VariableSets(d, _) => (
                    None,
                    options.map(|o| o.reps),
                    exercise.lower_weight(weights, d.current_index),
                ),
            }
        };
        if let Some(duration) = duration {
            let history = &mut state.write().unwrap().user.history;
            history.append_duration(&name, duration, weight.map(|w| w.value()));
        } else if let Some(reps) = reps {
            let history = &mut state.write().unwrap().user.history;
            history.append_reps(&name, reps, weight.map(|w| w.value()));
        } else {
            panic!("expected duration or reps");
        }
    }

    if in_workset(state, workout_name, exercise_name) {
        append_result(state, workout_name, exercise_name, options);
    }
    advance_current(state, workout_name, exercise_name);
}

fn complete_set(
    state: &mut SharedState,
    workout_name: &str,
    exercise_name: &str,
    options: Option<VarRepsOptions>,
) {
    let exercise_name = ExerciseName(exercise_name.to_owned());
    {
        // Reset current set to start
        let program = &mut state.write().unwrap().user.program;
        let workout = program.find_mut(&workout_name).unwrap();
        let exercise = workout.find_mut(&exercise_name).unwrap();
        exercise.reset(None);
    }

    {
        let history = &mut state.write().unwrap().user.history;
        history.finish(&exercise_name);
    }

    if let Some(options) = options {
        let mut new_expected = {
            let history = &state.read().unwrap().user.history;
            let program = &state.read().unwrap().user.program;
            let workout = program.find(&workout_name).unwrap();
            let exercise = workout.find(&exercise_name).unwrap();
            get_var_reps_done(history, &exercise.name())
        };

        if options.advance == 1 {
            // Advance weight (for VariableReps)
            let new_weight = {
                let weights = &state.read().unwrap().user.weights;
                let program = &state.read().unwrap().user.program;
                let workout = program.find(&workout_name).unwrap();
                let exercise = workout.find(&exercise_name).unwrap();
                exercise.advance_weight(weights).map(|w| w.value())
            };

            let program = &mut state.write().unwrap().user.program;
            let workout = program.find_mut(&workout_name).unwrap();
            let exercise = workout.find_mut(&exercise_name).unwrap();
            exercise.set_weight(new_weight);
            new_expected = match exercise {
                Exercise::VariableReps(_, e) => e.min_expected().clone(),
                Exercise::VariableSets(_, _) => new_expected, // not sure what something better would be
                _ => panic!("expected Exercise::VariableReps"),
            }
        }
        if options.update == 1 {
            // Update expected (for VariableReps)
            let program = &mut state.write().unwrap().user.program;
            let workout = program.find_mut(&workout_name).unwrap();
            let exercise = workout.find_mut(&exercise_name).unwrap();
            match exercise {
                Exercise::VariableReps(_, e) => {
                    e.set_expected(new_expected);
                }
                Exercise::VariableSets(_, e) => {
                    e.set_previous(new_expected);
                }
                _ => panic!("expected Exercise::VariableReps or VariableSets"),
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
struct ExerciseDataRecord {
    pub indicator: String,
    pub text: String,
    pub id: String,
}

#[derive(Serialize, Deserialize)]
struct RepItem {
    pub active: String, // "active" or ""
    pub title: String,  // "4 reps"
}

#[derive(Serialize, Deserialize)]
struct ExerciseData {
    workout: String,              // "Full Body Exercises"
    exercise: String,             // "RDL"
    exercise_set: String,         // "Set 1 of 3"
    exercise_set_details: String, // "8 reps @ 145 lbs"
    weight_details: String,       // "45 + 10 + 5"
    wait: String,                 // "" or "30" (seconds), this is for durations type exercises
    rest: String,                 // "" or "30" (seconds)
    button_title: String,         // "Next", "Start", "Done", "Exit", etc
    records: Vec<ExerciseDataRecord>,
    hide_reps: String,      // "hidden" or ""
    update_hidden: String,  // "hidden" or ""
    advance_hidden: String, // "hidden" or ""
    update_value: String,   // "1" or "0"
    advance_value: String,  // "1" or "0"
    reps_title: String,     // "8 reps"
    rep_items: Vec<RepItem>,
    notes: String,
}

impl ExerciseData {
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

    fn with_durations(
        weights: &Weights,
        notes: &Notes,
        history: &History,
        program: &Program,
        workout: &Workout,
        exercise: &Exercise,
    ) -> ExerciseData {
        let (d, e) = exercise.expect_durations();
        let exercise_set = format!("Set {} of {}", d.current_index.index() + 1, e.num_sets());

        let w = exercise.closest_weight(weights, d.current_index);
        let suffix = w
            .clone()
            .map_or("".to_owned(), |w| format!(" @ {}", w.text()));
        let exercise_set_details = format!("{}s{suffix}", e.set(d.current_index));

        let wdetails = w.map(|w| w.details()).flatten().unwrap_or("".to_owned());
        let tdetails = e
            .target()
            .map(|t| format!("target is {t}s"))
            .unwrap_or("".to_owned());
        let weight_details = if wdetails.is_empty() {
            tdetails
        } else {
            format!("{wdetails} ({tdetails})") // kinda lame formatting (tho this will likely be rare)
        };

        let wait = if d.finished {
            "0".to_owned()
        } else {
            format!("{}", e.set(d.current_index))
        };
        let rest = if d.finished {
            "0".to_owned()
        } else {
            exercise
                .rest(d.current_index)
                .map_or("0".to_owned(), |r| format!("{r}"))
        };
        let records = ExerciseData::get_records(history, program, workout, exercise);
        let button_title = if d.finished {
            "Exit".to_owned()
        } else {
            "Start".to_owned()
        };

        let notes = notes.html(&d.formal_name);

        let hide_reps = "hidden".to_owned(); // these are all for var reps exercises
        let update_hidden = "hidden".to_owned();
        let advance_hidden = "hidden".to_owned();
        let update_value = "0".to_owned();
        let advance_value = "0".to_owned();
        let reps_title = "".to_owned();
        let rep_items = Vec::new();

        let workout = workout.name.clone();
        let exercise = exercise.name().0.clone();
        ExerciseData {
            workout,
            exercise,
            exercise_set,
            exercise_set_details,
            weight_details,
            wait,
            rest,
            records,
            button_title,
            hide_reps,
            update_hidden,
            advance_hidden,
            update_value,
            advance_value,
            reps_title,
            rep_items,
            notes,
        }
    }

    fn with_fixed_reps(
        weights: &Weights,
        notes: &Notes,
        history: &History,
        program: &Program,
        workout: &Workout,
        exercise: &Exercise,
    ) -> ExerciseData {
        let (d, e) = exercise.expect_fixed_reps();
        let exercise_set = if e.num_warmups() > 0 {
            match d.current_index {
                SetIndex::Warmup(i) => format!("Warmup {} of {}", i + 1, e.num_warmups()),
                SetIndex::Workset(i) => format!("Workset {} of {}", i + 1, e.num_worksets()),
            }
        } else {
            format!(
                "Set {} of {}",
                d.current_index.index() + 1,
                e.num_worksets()
            )
        };

        let w = match d.current_index {
            SetIndex::Warmup(_) => exercise.closest_weight(weights, d.current_index),
            SetIndex::Workset(_) => exercise.lower_weight(weights, d.current_index),
        };
        let suffix = w
            .clone()
            .map_or("".to_owned(), |w| format!(" @ {}", w.text()));
        let reps = e.set(d.current_index).reps;
        let reps = if reps == 1 {
            "1 rep".to_owned()
        } else {
            format!("{reps} reps")
        };
        let exercise_set_details = format!("{reps}{suffix}");
        let weight_details = w.map(|w| w.details()).flatten().unwrap_or("".to_owned());

        let wait = "0".to_owned(); // for durations
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
        let records = ExerciseData::get_records(history, program, workout, exercise);
        let button_title = if d.finished {
            "Exit".to_owned()
        } else {
            match d.current_index {
                SetIndex::Warmup(_) => "Next".to_owned(),
                SetIndex::Workset(i) => {
                    if i + 1 < e.num_worksets() {
                        "Next".to_owned()
                    } else {
                        "Done".to_owned()
                    }
                }
            }
        };

        let notes = notes.html(&d.formal_name);

        let hide_reps = "hidden".to_owned(); // these are all for var reps exercises
        let update_hidden = "hidden".to_owned();
        let advance_hidden = "hidden".to_owned();
        let update_value = "0".to_owned();
        let advance_value = "0".to_owned();
        let reps_title = "".to_owned();
        let rep_items = Vec::new();

        let workout = workout.name.clone();
        let exercise = exercise.name().0.clone();
        ExerciseData {
            workout,
            exercise,
            exercise_set,
            exercise_set_details,
            weight_details,
            wait,
            rest,
            records,
            button_title,
            hide_reps,
            update_hidden,
            advance_hidden,
            update_value,
            advance_value,
            reps_title,
            rep_items,
            notes,
        }
    }

    fn with_var_sets(
        weights: &Weights,
        notes: &Notes,
        history: &History,
        program: &Program,
        workout: &Workout,
        exercise: &Exercise,
    ) -> ExerciseData {
        let (d, e) = exercise.expect_var_sets();
        let num_sets = e.get_previous().len();
        let exercise_set = if num_sets > 0 && !d.finished {
            format!("Set {} of {}+", d.current_index.index() + 1, num_sets)
        } else {
            format!("Set {}", d.current_index.index() + 1,)
        };

        let w = exercise.lower_weight(weights, d.current_index);
        let suffix = w
            .clone()
            .map_or("".to_owned(), |w| format!(" @ {}", w.text()));

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
        let previous = e.previous(d.current_index);
        let exercise_set_details = if d.finished {
            "".to_owned()
        } else if previous == 0 {
            format!("{}+ reps{suffix}", remaining)
        } else if previous < remaining {
            format!("{}+ reps{suffix}", previous)
        } else if previous == remaining {
            format!("{} reps{suffix}", remaining)
        } else {
            format!("{}+ reps{suffix}", remaining)
        };
        let weight_details = w
            .clone()
            .map(|w| w.details())
            .flatten()
            .unwrap_or("".to_owned());

        let wait = "0".to_owned(); // for durations
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
        let records = ExerciseData::get_records(history, program, workout, exercise);
        let button_title = if d.finished {
            "Exit".to_owned()
        } else {
            "Next".to_owned()
        };

        let notes = notes.html(&d.formal_name);

        let hide_reps = if d.finished {
            "hidden".to_owned()
        } else {
            "".to_owned()
        };
        let (expected, max) = if previous > 0 {
            (previous, previous + 5)
        } else if remaining < 5 {
            (remaining, remaining + 5)
        } else {
            (5, 12)
        };
        let reps_title = reps_to_title(expected);
        let rep_items = reps_to_vec(VariableReps::new(expected, max, 100));

        let reps = get_var_reps_done(history, exercise.name());
        let (update_hidden, update_value) = if d.finished && reps != *e.get_previous() {
            ("".to_owned(), "1".to_owned())
        } else {
            ("hidden".to_owned(), "0".to_owned())
        };

        let (advance_hidden, advance_value) = if d.finished && w.is_some() && done >= e.target() {
            ("".to_owned(), "0".to_owned())
        } else {
            ("hidden".to_owned(), "0".to_owned())
        };

        let workout = workout.name.clone();
        let exercise = exercise.name().0.clone();
        ExerciseData {
            workout,
            exercise,
            exercise_set,
            exercise_set_details,
            weight_details,
            wait,
            rest,
            records,
            button_title,
            hide_reps,
            update_hidden,
            advance_hidden,
            update_value,
            advance_value,
            reps_title,
            rep_items,
            notes,
        }
    }

    fn with_var_reps(
        weights: &Weights,
        notes: &Notes,
        history: &History,
        program: &Program,
        workout: &Workout,
        exercise: &Exercise,
    ) -> ExerciseData {
        let (d, e) = exercise.expect_var_reps();
        let exercise_set = if e.num_warmups() > 0 {
            match d.current_index {
                SetIndex::Warmup(i) => format!("Warmup {} of {}", i + 1, e.num_warmups()),
                SetIndex::Workset(i) => format!("Workset {} of {}", i + 1, e.num_worksets()),
            }
        } else {
            format!(
                "Set {} of {}",
                d.current_index.index() + 1,
                e.num_worksets()
            )
        };

        let w = match d.current_index {
            SetIndex::Warmup(_) => exercise.closest_weight(weights, d.current_index),
            SetIndex::Workset(_) => exercise.lower_weight(weights, d.current_index),
        };
        let suffix = w
            .clone()
            .map_or("".to_owned(), |w| format!(" @ {}", w.text()));
        let exercise_set_details = {
            let range = e.expected_range(d.current_index);
            if range.min < range.max {
                format!("{}-{} reps{suffix}", range.min, range.max)
            } else {
                format!("{} reps{suffix}", range.max)
            }
        };
        let weight_details = w
            .clone()
            .map(|w| w.details())
            .flatten()
            .unwrap_or("".to_owned());

        let wait = "0".to_owned(); // for durations
        let rest = if d.finished {
            "0".to_owned()
        } else {
            exercise
                .rest(d.current_index)
                .map_or("0".to_owned(), |r| format!("{r}"))
        };
        let records = ExerciseData::get_records(history, program, workout, exercise);
        let button_title = if d.finished {
            "Exit".to_owned()
        } else {
            match d.current_index {
                SetIndex::Warmup(_) => "Next".to_owned(),
                SetIndex::Workset(i) => {
                    if i + 1 < e.num_worksets() {
                        "Next".to_owned()
                    } else {
                        "Done".to_owned()
                    }
                }
            }
        };

        let notes = notes.html(&d.formal_name);

        let in_workset = if let SetIndex::Workset(_) = d.current_index {
            true
        } else {
            false
        };
        let hide_reps = if d.finished || !in_workset {
            "hidden".to_owned()
        } else {
            "".to_owned()
        };
        let expected = e.expected_range(d.current_index);
        let reps_title = reps_to_title(expected.min);
        let rep_items = reps_to_vec(expected);

        let reps = get_var_reps_done(history, exercise.name());
        let (update_hidden, update_value) = if d.finished && reps != *e.expected() {
            ("".to_owned(), "1".to_owned())
        } else {
            ("hidden".to_owned(), "0".to_owned())
        };

        let (advance_hidden, advance_value) =
            if d.finished && w.is_some() && reps >= e.max_expected() {
                ("".to_owned(), "0".to_owned())
            } else {
                ("hidden".to_owned(), "0".to_owned())
            };

        let workout = workout.name.clone();
        let exercise = exercise.name().0.clone();
        ExerciseData {
            workout,
            exercise,
            exercise_set,
            exercise_set_details,
            weight_details,
            wait,
            rest,
            records,
            button_title,
            hide_reps,
            update_hidden,
            advance_hidden,
            update_value,
            advance_value,
            reps_title,
            rep_items,
            notes,
        }
    }

    fn new(
        weights: &Weights,
        notes: &Notes,
        history: &History,
        program: &Program,
        workout: &Workout,
        exercise: &Exercise,
    ) -> ExerciseData {
        match exercise {
            Exercise::Durations(_, _) => {
                ExerciseData::with_durations(weights, notes, history, program, workout, exercise)
            }
            Exercise::FixedReps(_, _) => {
                ExerciseData::with_fixed_reps(weights, notes, history, program, workout, exercise)
            }
            Exercise::VariableReps(_, _) => {
                ExerciseData::with_var_reps(weights, notes, history, program, workout, exercise)
            }
            Exercise::VariableSets(_, _) => {
                ExerciseData::with_var_sets(weights, notes, history, program, workout, exercise)
            }
        }
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
    let mut text = String::new();

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
    text += &record.started.format("%-d %b %Y").to_string();

    if let Some(ref sets) = record.sets {
        text += ", ";
        text += &match sets {
            CompletedSets::Durations(s) => durations_to_str(s),
            CompletedSets::Reps(s) => reps_to_str(s),
        };
    }

    if let Some(ref comment) = record.comment {
        text += ", ";
        text += comment;
    }

    let id = if delta > 0 && !in_progress {
        "better_record".to_owned()
    } else if delta < 0 && !in_progress {
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

fn durations_to_str(sets: &Vec<(i32, Option<f32>)>) -> String {
    num_to_str(sets, "secs") // TODO will need to also pass in a fn so we can get short times
}

fn reps_to_str(sets: &Vec<(i32, Option<f32>)>) -> String {
    num_to_str(sets, "reps")
}

fn num_to_str(sets: &Vec<(i32, Option<f32>)>, unit: &str) -> String {
    if sets.iter().all(|s| s.1.is_none()) {
        let reps: Vec<_> = sets.iter().map(|x| format!("{}", x.0)).collect();
        let reps = join_labels(reps);
        reps + " " + unit
    } else if !sets.is_empty() && sets[0].1.is_some() && sets.iter().all(|s| s.1 == sets[0].1) {
        let reps: Vec<_> = sets.iter().map(|x| format!("{}", x.0)).collect();
        let reps = join_labels(reps);
        let weight = format_weight(sets[0].1.unwrap(), " lbs");
        format!("{reps} {unit} @ {weight}")
    } else {
        join_labels(
            sets.iter()
                .map(|x| {
                    let reps = format!("{}", x.0);
                    if let Some(weight) = x.1 {
                        let weight = format_weight(weight, " lbs");
                        format!("{reps} @ {weight}")
                    } else {
                        reps
                    }
                })
                .collect(),
        )
    }
}

fn get_var_reps_done(history: &History, name: &ExerciseName) -> Vec<i32> {
    let last = history.records(name).last().map_or(&None, |r| &r.sets);
    match last {
        Some(CompletedSets::Reps(v)) => v.iter().map(|t| t.0).collect(),
        _ => Vec::new(),
    }
}
