use crate::{
    exercise::{Exercise, ExerciseName, FixedReps, SetIndex, VariableReps},
    history::CompletedSets,
    pages::SharedState,
    VarRepsOptions,
};
use anyhow::Context;
use axum::http::Uri;
use chrono::Local;

pub fn post_next_exercise(
    mut state: SharedState,
    workout_name: &str,
    exercise_name: &str,
    options: Option<VarRepsOptions>,
) -> Result<Uri, anyhow::Error> {
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

    if finished {
        let user = &mut state.write().unwrap().user;
        if let Err(e) = crate::persist::save(user) {
            user.errors.push(format!("{e}"));
        }
    };

    if finished {
        complete_set(&mut state, workout_name, exercise_name, options);

        let path = format!("/workout/{workout_name}");
        let uri = url_escape::encode_path(&path);
        let uri = uri.parse()?;
        Ok(uri)
    } else {
        advance_set(&mut state, workout_name, exercise_name, options);

        let path = format!("/exercise/{workout_name}/{exercise_name}");
        let uri = url_escape::encode_path(&path);
        let uri = uri.parse()?;
        Ok(uri)
    }
}

pub fn post_reset_exercise(
    state: SharedState,
    workout_name: &str,
    exercise_name: &str,
) -> Result<Uri, anyhow::Error> {
    let exercise_name = ExerciseName(exercise_name.to_owned());

    {
        let program = &mut state.write().unwrap().user.program;
        let workout = program.find_mut(&workout_name).unwrap();
        let exercise = workout.find_mut(&exercise_name).unwrap();
        exercise.reset(Some(Local::now()));
    }

    {
        let history = &mut state.write().unwrap().user.history;
        history.abort(&exercise_name);
    }

    let path = format!("/exercise/{workout_name}/{exercise_name}");
    let uri = url_escape::encode_path(&path);
    let uri = uri.parse()?;
    Ok(uri)
}

pub fn post_append_exercise(
    state: SharedState,
    workout_name: &str,
    exercise: Exercise,
) -> Result<Uri, anyhow::Error> {
    {
        let program = &mut state.write().unwrap().user.program;
        let workout = program.find_mut(&workout_name).unwrap();
        workout.try_add_exercise(exercise)?;
    }

    {
        let user = &mut state.write().unwrap().user;
        if let Err(e) = crate::persist::save(user) {
            user.errors.push(format!("{e}")); // not fatal so we don't return an error
        }
    }

    let path = format!("/workout/{workout_name}");
    let uri = url_escape::encode_path(&path);
    let uri = uri.parse()?;
    Ok(uri)
}
pub fn post_set_exercises(
    state: SharedState,
    workout_name: &str,
    enabled: Vec<&str>,
    disabled: Vec<bool>,
) -> Result<Uri, anyhow::Error> {
    {
        let program = &mut state.write().unwrap().user.program;
        let workout = program.find_mut(&workout_name).unwrap();
        workout.try_set_exercises(enabled, disabled)?;
    }

    {
        let user = &mut state.write().unwrap().user;
        if let Err(e) = crate::persist::save(user) {
            user.errors.push(format!("{e}")); // not fatal so we don't return an error
        }
    }

    let path = format!("/workout/{workout_name}");
    let uri = url_escape::encode_path(&path);
    let uri = uri.parse()?;
    Ok(uri)
}

pub fn post_set_name(
    state: SharedState,
    workout_name: &str,
    old_name: &str,
    new_name: &str,
) -> Result<Uri, anyhow::Error> {
    if old_name != new_name {
        let old_name = ExerciseName(old_name.to_owned());
        {
            let program = &mut state.write().unwrap().user.program;
            let workout = program.find_mut(&workout_name).unwrap();
            workout.try_change_name(&old_name, new_name)?;
        }

        {
            let user = &mut state.write().unwrap().user;
            if let Err(e) = crate::persist::save(user) {
                user.errors.push(format!("{e}")); // not fatal so we don't return an error
            }
        }
    }

    let path = format!("/exercise/{workout_name}/{new_name}");
    let uri = url_escape::encode_path(&path);
    let uri = uri.parse()?;
    Ok(uri)
}

pub fn post_set_formal_name(
    state: SharedState,
    workout_name: &str,
    exercise_name: &str,
    name: &str,
) -> Result<Uri, anyhow::Error> {
    let exercise_name = ExerciseName(exercise_name.to_owned());
    {
        let program = &mut state.write().unwrap().user.program;
        let workout = program.find_mut(&workout_name).unwrap();
        let exercise = workout.find_mut(&exercise_name).unwrap();
        exercise.try_set_formal_name(name)?;
    }

    {
        let user = &mut state.write().unwrap().user;
        if let Err(e) = crate::persist::save(user) {
            user.errors.push(format!("{e}")); // not fatal so we don't return an error
        }
    }

    let path = format!("/exercise/{workout_name}/{exercise_name}");
    let uri = url_escape::encode_path(&path);
    let uri = uri.parse()?;
    Ok(uri)
}

pub fn post_set_weight(
    state: SharedState,
    workout_name: &str,
    exercise_name: &str,
    weight: Option<f32>,
) -> Result<Uri, anyhow::Error> {
    let exercise_name = ExerciseName(exercise_name.to_owned());

    {
        let program = &mut state.write().unwrap().user.program;
        let workout = program.find_mut(&workout_name).unwrap();
        let exercise = workout.find_mut(&exercise_name).unwrap();
        exercise.try_set_weight(weight)?;
    }

    {
        let user = &mut state.write().unwrap().user;
        if let Err(e) = crate::persist::save(user) {
            user.errors.push(format!("{e}")); // not fatal so we don't return an error
        }
    }

    let path = format!("/exercise/{workout_name}/{exercise_name}");
    let uri = url_escape::encode_path(&path);
    let uri = uri.parse()?;
    Ok(uri)
}

pub fn post_set_note(
    state: SharedState,
    workout_name: &str,
    exercise_name: &str,
    note: String,
) -> Result<Uri, anyhow::Error> {
    let exercise_name = ExerciseName(exercise_name.to_owned());

    {
        let formal_name = {
            let program = &state.read().unwrap().user.program;
            let workout = program.find(&workout_name).unwrap();
            let exercise = workout.find(&exercise_name).unwrap();
            let d = exercise.data();
            d.formal_name.clone()
        };

        let notes = &mut state.write().unwrap().user.notes;
        notes.set_markdown(formal_name, note);
    }

    {
        let user = &mut state.write().unwrap().user;
        if let Err(e) = crate::persist::save(user) {
            user.errors.push(format!("{e}")); // not fatal so we don't return an error
        }
    }

    let path = format!("/exercise/{workout_name}/{exercise_name}");
    let uri = url_escape::encode_path(&path);
    let uri = uri.parse()?;
    Ok(uri)
}

pub fn post_revert_note(
    state: SharedState,
    workout_name: &str,
    exercise_name: &str,
) -> Result<Uri, anyhow::Error> {
    let exercise_name = ExerciseName(exercise_name.to_owned());

    {
        let formal_name = {
            let program = &state.read().unwrap().user.program;
            let workout = program.find(&workout_name).unwrap();
            let exercise = workout.find(&exercise_name).unwrap();
            let d = exercise.data();
            d.formal_name.clone()
        };

        let notes = &mut state.write().unwrap().user.notes;
        notes.revert_markdown(formal_name);
    }

    {
        let user = &mut state.write().unwrap().user;
        if let Err(e) = crate::persist::save(user) {
            user.errors.push(format!("{e}")); // not fatal so we don't return an error
        }
    }

    let path = format!("/exercise/{workout_name}/{exercise_name}");
    let uri = url_escape::encode_path(&path);
    let uri = uri.parse()?;
    Ok(uri)
}

pub fn post_set_durations(
    state: SharedState,
    workout_name: &str,
    exercise_name: &str,
    durations: Vec<i32>,
    target: Option<i32>,
) -> Result<Uri, anyhow::Error> {
    let exercise_name = ExerciseName(exercise_name.to_owned());

    {
        let program = &mut state.write().unwrap().user.program;
        let workout = program.find_mut(&workout_name).unwrap();
        let exercise = workout.find_mut(&exercise_name).unwrap();
        let (_, e) = exercise.expect_durations_mut();
        e.try_set_durations(durations)?;
        e.try_set_target(target)?;
    }

    {
        let user = &mut state.write().unwrap().user;
        if let Err(e) = crate::persist::save(user) {
            user.errors.push(format!("{e}")); // not fatal so we don't return an error
        }
    }

    let path = format!("/exercise/{workout_name}/{exercise_name}");
    let uri = url_escape::encode_path(&path);
    let uri = uri.parse()?;
    Ok(uri)
}

pub fn post_set_fixed_reps(
    state: SharedState,
    workout_name: &str,
    exercise_name: &str,
    warmups: Vec<FixedReps>,
    worksets: Vec<FixedReps>,
) -> Result<Uri, anyhow::Error> {
    let exercise_name = ExerciseName(exercise_name.to_owned());

    {
        let program = &mut state.write().unwrap().user.program;
        let workout = program.find_mut(&workout_name).unwrap();
        let exercise = workout.find_mut(&exercise_name).unwrap();
        let (d, e) = exercise.expect_fixed_reps_mut();
        e.try_set_warmups(warmups)?;
        e.try_set_worksets(worksets)?;

        if !d.finished {
            exercise.reset(exercise.started());
        }
    }

    {
        let user = &mut state.write().unwrap().user;
        if let Err(e) = crate::persist::save(user) {
            user.errors.push(format!("{e}")); // not fatal so we don't return an error
        }
    }

    let path = format!("/exercise/{workout_name}/{exercise_name}");
    let uri = url_escape::encode_path(&path);
    let uri = uri.parse()?;
    Ok(uri)
}

pub fn post_set_var_reps(
    state: SharedState,
    workout_name: &str,
    exercise_name: &str,
    warmups: Vec<FixedReps>,
    worksets: Vec<VariableReps>,
) -> Result<Uri, anyhow::Error> {
    let exercise_name = ExerciseName(exercise_name.to_owned());

    {
        let program = &mut state.write().unwrap().user.program;
        let workout = program.find_mut(&workout_name).unwrap();
        let exercise = workout.find_mut(&exercise_name).unwrap();
        let (d, e) = exercise.expect_var_reps_mut();
        e.try_set_warmups(warmups)?;
        e.try_set_worksets(worksets)?;

        if !d.finished {
            exercise.reset(exercise.started());
        }
    }

    {
        let user = &mut state.write().unwrap().user;
        if let Err(e) = crate::persist::save(user) {
            user.errors.push(format!("{e}")); // not fatal so we don't return an error
        }
    }

    let path = format!("/exercise/{workout_name}/{exercise_name}");
    let uri = url_escape::encode_path(&path);
    let uri = uri.parse()?;
    Ok(uri)
}

pub fn post_set_var_sets(
    state: SharedState,
    workout_name: &str,
    exercise_name: &str,
    target: i32,
) -> Result<Uri, anyhow::Error> {
    let exercise_name = ExerciseName(exercise_name.to_owned());

    {
        let program = &mut state.write().unwrap().user.program;
        let workout = program.find_mut(&workout_name).unwrap();
        let exercise = workout.find_mut(&exercise_name).unwrap();
        let (_, e) = exercise.expect_var_sets_mut();
        e.try_set_target(target)?;
    }

    {
        let user = &mut state.write().unwrap().user;
        if let Err(e) = crate::persist::save(user) {
            user.errors.push(format!("{e}")); // not fatal so we don't return an error
        }
    }

    let path = format!("/exercise/{workout_name}/{exercise_name}");
    let uri = url_escape::encode_path(&path);
    let uri = uri.parse()?;
    Ok(uri)
}

pub fn post_set_rest(
    state: SharedState,
    workout_name: &str,
    exercise_name: &str,
    rest: Option<i32>,
    last_rest: Option<i32>,
) -> Result<Uri, anyhow::Error> {
    let exercise_name = ExerciseName(exercise_name.to_owned());

    {
        let program = &mut state.write().unwrap().user.program;
        let workout = program.find_mut(&workout_name).unwrap();
        let exercise = workout.find_mut(&exercise_name).unwrap();
        exercise.try_set_rest(rest)?;
        exercise.try_set_last_rest(last_rest)?;
    }

    {
        let user = &mut state.write().unwrap().user;
        if let Err(e) = crate::persist::save(user) {
            user.errors.push(format!("{e}")); // not fatal so we don't return an error
        }
    }

    let path = format!("/exercise/{workout_name}/{exercise_name}");
    let uri = url_escape::encode_path(&path);
    let uri = uri.parse()?;
    Ok(uri)
}

pub fn post_set_durs_record(
    state: SharedState,
    workout_name: &str,
    exercise_name: &str,
    durations: Vec<i32>,
    weights: Vec<f32>,
    comment: String,
    id: u64,
) -> Result<Uri, anyhow::Error> {
    let exercise_name = ExerciseName(exercise_name.to_owned());

    {
        let history = &mut state.write().unwrap().user.history;
        let record = history.find_record_mut(&exercise_name, id)?;
        let sets = if durations.len() == weights.len() {
            durations
                .iter()
                .copied()
                .zip(weights.iter().map(|w| Some(*w)))
                .collect()
        } else if weights.is_empty() {
            durations.iter().map(|r| (*r, None)).collect()
        } else {
            return Err(anyhow::Error::msg(
                "Weights must be empty or match durations",
            ));
        };
        record.sets = Some(CompletedSets::Durations(sets));
        if !comment.is_empty() {
            record.comment = Some(comment);
        } else {
            record.comment = None;
        }
    }

    let path = format!("/exercise/{workout_name}/{exercise_name}");
    let uri = url_escape::encode_path(&path);
    let uri = uri.parse()?;
    Ok(uri)
}

pub fn post_set_reps_record(
    state: SharedState,
    workout_name: &str,
    exercise_name: &str,
    reps: Vec<i32>,
    weights: Vec<f32>,
    comment: String,
    id: u64,
) -> Result<Uri, anyhow::Error> {
    let exercise_name = ExerciseName(exercise_name.to_owned());

    {
        let history = &mut state.write().unwrap().user.history;
        let record = history.find_record_mut(&exercise_name, id)?;
        let sets = if reps.len() == weights.len() {
            reps.iter()
                .copied()
                .zip(weights.iter().map(|w| Some(*w)))
                .collect()
        } else if weights.is_empty() {
            reps.iter().map(|r| (*r, None)).collect()
        } else {
            return Err(anyhow::Error::msg("Weights must be empty or match reps"));
        };
        record.sets = Some(CompletedSets::Reps(sets));
        if !comment.is_empty() {
            record.comment = Some(comment);
        } else {
            record.comment = None;
        }
    }

    let path = format!("/exercise/{workout_name}/{exercise_name}");
    let uri = url_escape::encode_path(&path);
    let uri = uri.parse()?;
    Ok(uri)
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
        history.finish(&exercise_name, Local::now());
    }

    if let Some(options) = options {
        let mut new_expected = {
            let history = &state.read().unwrap().user.history;
            let program = &state.read().unwrap().user.program;
            let workout = program.find(&workout_name).unwrap();
            let exercise = workout.find(&exercise_name).unwrap();
            super::get_var_reps_done(history, &exercise.name())
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
            super::get_var_reps_done(history, &name).iter().sum() // ok to call this if not var sets
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
