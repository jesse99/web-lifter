//! Exercises are movements for the user to perform, e.g. a barbell squat. These may be
//! shared across programs and workouts.
use crate::{
    pages::ValidationError,
    weights::{Weight, Weights},
};
use chrono::{DateTime, Local};
use core::fmt;
use serde::{Deserialize, Serialize};
use std::fmt::Formatter;

mod durations_exercise;
mod fixed_reps_exercise;
mod variable_reps_exercise;
mod variable_sets_exercise;

pub use durations_exercise::*;
pub use fixed_reps_exercise::*;
pub use variable_reps_exercise::*;
pub use variable_sets_exercise::*;

/// Identifies an exercise. This is assigned by the user and will be something like
/// "Light Squat". It's used when listing the exercise within a [`Workout`] and to
/// persist history (across workouts and programs).
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct ExerciseName(pub String);

/// The proper exercise name, e.g. "Low-bar Squat". This is used to show help for the
/// exercise.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct FormalName(pub String);

/// Not all exercises will support all of these.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum SetIndex {
    Warmup(usize),
    Workset(usize),
    // Backoff(usize),
}

impl SetIndex {
    pub fn index(&self) -> usize {
        match self {
            SetIndex::Warmup(i) => *i,
            SetIndex::Workset(i) => *i,
        }
    }
}

/// State shared across exercise types.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExerciseData {
    pub name: ExerciseName,
    pub formal_name: FormalName,
    pub started: Option<DateTime<Local>>,
    pub finished: bool,
    pub current_index: SetIndex,
    pub weightset: Option<String>,
    pub weight: Option<f32>, // base weight to use for each workset, often modified by set percent
    pub rest: Option<i32>,   // used for work sets
    pub last_rest: Option<i32>, // overrides rest.last()
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Exercise {
    Durations(ExerciseData, DurationsExercise),
    FixedReps(ExerciseData, FixedRepsExercise),
    VariableReps(ExerciseData, VariableRepsExercise),
    VariableSets(ExerciseData, VariableSetsExercise),
}

impl Exercise {
    pub fn name(&self) -> &ExerciseName {
        match self {
            Exercise::Durations(d, _) => &d.name,
            Exercise::FixedReps(d, _) => &d.name,
            Exercise::VariableReps(d, _) => &d.name,
            Exercise::VariableSets(d, _) => &d.name,
        }
    }

    pub fn started(&self) -> Option<DateTime<Local>> {
        match self {
            Exercise::Durations(d, _) => d.started,
            Exercise::FixedReps(d, _) => d.started,
            Exercise::VariableReps(d, _) => d.started,
            Exercise::VariableSets(d, _) => d.started,
        }
    }

    pub fn reset(&mut self, new_start: Option<DateTime<Local>>) {
        match self {
            Exercise::Durations(d, _) => {
                d.current_index = SetIndex::Workset(0);
                d.finished = false;
                d.started = new_start;
            }
            Exercise::FixedReps(d, e) => {
                if e.num_warmups() > 0 {
                    d.current_index = SetIndex::Warmup(0);
                } else {
                    d.current_index = SetIndex::Workset(0);
                }
                d.finished = false;
                d.started = new_start;
            }
            Exercise::VariableReps(d, e) => {
                if e.num_warmups() > 0 {
                    d.current_index = SetIndex::Warmup(0);
                } else {
                    d.current_index = SetIndex::Workset(0);
                }
                d.finished = false;
                d.started = new_start;
            }
            Exercise::VariableSets(d, _) => {
                d.current_index = SetIndex::Workset(0);
                d.finished = false;
                d.started = new_start;
            }
        }
    }

    pub fn data(&self) -> &ExerciseData {
        match self {
            Exercise::Durations(d, _) => d,
            Exercise::FixedReps(d, _) => d,
            Exercise::VariableReps(d, _) => d,
            Exercise::VariableSets(d, _) => d,
        }
    }

    pub fn expect_durations(&self) -> (&ExerciseData, &DurationsExercise) {
        match self {
            Exercise::Durations(d, e) => (d, e),
            _ => panic!("expected durations"),
        }
    }

    pub fn expect_durations_mut(&mut self) -> (&mut ExerciseData, &mut DurationsExercise) {
        match self {
            Exercise::Durations(d, e) => (d, e),
            _ => panic!("expected durations"),
        }
    }

    pub fn expect_var_sets_mut(&mut self) -> (&mut ExerciseData, &mut VariableSetsExercise) {
        match self {
            Exercise::VariableSets(d, e) => (d, e),
            _ => panic!("expected var_sets"),
        }
    }

    pub fn expect_fixed_reps(&self) -> (&ExerciseData, &FixedRepsExercise) {
        match self {
            Exercise::FixedReps(d, e) => (d, e),
            _ => panic!("expected fixed reps"),
        }
    }

    pub fn expect_var_reps(&self) -> (&ExerciseData, &VariableRepsExercise) {
        match self {
            Exercise::VariableReps(d, e) => (d, e),
            _ => panic!("expected var reps"),
        }
    }

    pub fn expect_var_sets(&self) -> (&ExerciseData, &VariableSetsExercise) {
        match self {
            Exercise::VariableSets(d, e) => (d, e),
            _ => panic!("expected var sets"),
        }
    }

    // pub fn expect_var_sets_mut(&mut self) -> (&mut ExerciseData, &mut VariableSetsExercise) {
    //     match self {
    //         Exercise::VariableSets(d, e) => (d, e),
    //         _ => panic!("expected var sets"),
    //     }
    // }

    /// Used for warmup sets, returned weight may be over expected weight.
    pub fn closest_weight(&self, weights: &Weights, index: SetIndex) -> Option<Weight> {
        let (target, name) = self.target_weight(index);
        if let Some(name) = name {
            target.map(|t| weights.closest(&name, t))
        } else {
            target.map(|t| weights.closest("", t))
        }
    }

    /// Used for worksets sets, returns a weight as close as possible to the expected
    /// weight but not over.
    pub fn lower_weight(&self, weights: &Weights, index: SetIndex) -> Option<Weight> {
        let (target, name) = self.target_weight(index);
        if let Some(name) = name {
            target.map(|t| weights.lower(&name, t))
        } else {
            target.map(|t| weights.closest("", t))
        }
    }

    pub fn advance_weight(&self, weights: &Weights) -> Option<Weight> {
        let (target, name) = self.base_weight();
        if let Some(name) = name {
            target.map(|t| weights.advance(&name, t))
        } else {
            target.map(|t| weights.advance("", t))
        }
    }

    pub fn try_set_weight(&mut self, weight: Option<f32>) -> Result<(), ValidationError> {
        self.validate_weight(weight)?;
        self.do_set_weight(weight);
        Ok(())
    }

    pub fn set_weight(&mut self, weight: Option<f32>) {
        assert!(self.validate_weight(weight).is_ok());
        self.do_set_weight(weight);
    }

    pub fn try_set_rest(&mut self, rest: Option<i32>) -> Result<(), ValidationError> {
        self.validate_rest(rest)?;
        self.do_set_rest(rest);
        Ok(())
    }

    // pub fn set_rest(&mut self, rest: Option<i32>) {
    //     assert!(self.validate_rest(rest).is_ok());
    //     self.do_set_rest(rest);
    // }

    pub fn try_set_last_rest(&mut self, last_rest: Option<i32>) -> Result<(), ValidationError> {
        self.validate_last_rest(last_rest)?;
        self.do_set_last_rest(last_rest);
        Ok(())
    }

    // pub fn set_last_rest(&mut self, last_rest: Option<i32>) {
    //     assert!(self.validate_last_rest(last_rest).is_ok());
    //     self.do_set_last_rest(last_rest);
    // }

    // pub fn set_weightset(&mut self, set: Option<String>) {
    //     // assert!(self.validate_weightset(weight).is_ok());
    //     match self {
    //         Exercise::Durations(d, _) => d.weightset = set,
    //         Exercise::FixedReps(d, _) => d.weightset = set,
    //         Exercise::VariableReps(d, _) => d.weightset = set,
    //         Exercise::VariableSets(d, _) => d.weightset = set,
    //     }
    // }

    /// For the specified set, in seconds.
    pub fn rest(&self, index: SetIndex) -> Option<i32> {
        fn get(index: usize, num: usize, d: &ExerciseData) -> Option<i32> {
            if index + 1 == num && d.last_rest.is_some() {
                d.last_rest
            } else {
                d.rest
            }
        }

        match self {
            Exercise::Durations(d, e) => match index {
                SetIndex::Workset(i) => get(i, e.num_sets(), d),
                _ => None,
            },
            Exercise::FixedReps(d, e) => match index {
                SetIndex::Workset(i) => get(i, e.num_worksets(), d),
                _ => None,
            },
            Exercise::VariableReps(d, e) => match index {
                SetIndex::Workset(i) => get(i, e.num_worksets(), d),
                _ => None,
            },
            Exercise::VariableSets(d, _) => d.rest, // last_rest isn't used because we don't know the last set until it's done
        }
    }

    fn target_weight(&self, index: SetIndex) -> (Option<f32>, &Option<String>) {
        match self {
            Exercise::Durations(d, _) => (d.weight, &d.weightset),
            Exercise::FixedReps(d, e) => {
                let percent = e.set(index).percent as f32;
                (d.weight.map(|w| (percent * w) / 100.0), &d.weightset)
            }
            Exercise::VariableReps(d, e) => {
                let percent = e.expected_range(index).percent as f32;
                (d.weight.map(|w| (percent * w) / 100.0), &d.weightset)
            }
            Exercise::VariableSets(d, _) => (d.weight, &d.weightset),
        }
    }

    fn base_weight(&self) -> (Option<f32>, &Option<String>) {
        match self {
            Exercise::Durations(d, _) => (d.weight, &d.weightset),
            Exercise::FixedReps(d, _) => (d.weight, &d.weightset),
            Exercise::VariableReps(d, _) => (d.weight, &d.weightset),
            Exercise::VariableSets(d, _) => (d.weight, &d.weightset),
        }
    }

    fn validate_weight(&self, weight: Option<f32>) -> Result<(), ValidationError> {
        if let Some(weight) = weight {
            if weight < 0.0 {
                return Err(ValidationError::new("Weight cannot be negative"));
            }
            if weight.is_nan() {
                return Err(ValidationError::new("Weight cannot be a Not A Number"));
            }
            if weight.is_infinite() {
                return Err(ValidationError::new("Weight cannot be a infinite"));
            }
        }
        Ok(())
    }

    fn do_set_weight(&mut self, weight: Option<f32>) {
        match self {
            Exercise::Durations(d, _) => d.weight = weight,
            Exercise::FixedReps(d, _) => d.weight = weight,
            Exercise::VariableReps(d, _) => d.weight = weight,
            Exercise::VariableSets(d, _) => d.weight = weight,
        }
    }

    fn validate_rest(&self, rest: Option<i32>) -> Result<(), ValidationError> {
        if let Some(rest) = rest {
            if rest < 0 {
                return Err(ValidationError::new("Rest cannot be negative"));
            }
        }
        Ok(())
    }

    fn do_set_rest(&mut self, rest: Option<i32>) {
        match self {
            Exercise::Durations(d, _) => d.rest = rest,
            Exercise::FixedReps(d, _) => d.rest = rest,
            Exercise::VariableReps(d, _) => d.rest = rest,
            Exercise::VariableSets(d, _) => d.rest = rest,
        }
    }

    fn validate_last_rest(&self, last_rest: Option<i32>) -> Result<(), ValidationError> {
        if let Some(last_rest) = last_rest {
            if last_rest < 0 {
                return Err(ValidationError::new("Last rest cannot be negative"));
            }
        }
        Ok(())
    }

    fn do_set_last_rest(&mut self, last_rest: Option<i32>) {
        match self {
            Exercise::Durations(d, _) => d.last_rest = last_rest,
            Exercise::FixedReps(d, _) => d.last_rest = last_rest,
            Exercise::VariableReps(d, _) => d.last_rest = last_rest,
            Exercise::VariableSets(d, _) => d.last_rest = last_rest,
        }
    }
}

/// Builder for ['Exercise`]'s that use sets.
pub struct BuildExercise {
    exercise: Exercise,
    data: ExerciseData,
}

impl BuildExercise {
    pub fn durations(
        name: ExerciseName,
        formal_name: FormalName,
        exercise: DurationsExercise,
    ) -> BuildExercise {
        let data = ExerciseData::new(name, formal_name, SetIndex::Workset(0));
        BuildExercise {
            exercise: Exercise::Durations(data.clone(), exercise),
            data,
        }
    }

    #[allow(dead_code)]
    pub fn fixed_reps(
        name: ExerciseName,
        formal_name: FormalName,
        exercise: FixedRepsExercise,
    ) -> BuildExercise {
        let data = if exercise.num_warmups() > 0 {
            ExerciseData::new(name, formal_name, SetIndex::Warmup(0))
        } else {
            ExerciseData::new(name, formal_name, SetIndex::Workset(0))
        };
        BuildExercise {
            exercise: Exercise::FixedReps(data.clone(), exercise),
            data,
        }
    }

    pub fn variable_reps(
        name: ExerciseName,
        formal_name: FormalName,
        exercise: VariableRepsExercise,
    ) -> BuildExercise {
        let data = ExerciseData::new(name, formal_name, SetIndex::Workset(0));
        BuildExercise {
            exercise: Exercise::VariableReps(data.clone(), exercise),
            data,
        }
    }

    pub fn variable_sets(
        name: ExerciseName,
        formal_name: FormalName,
        exercise: VariableSetsExercise,
    ) -> BuildExercise {
        let data = ExerciseData::new(name, formal_name, SetIndex::Workset(0));
        BuildExercise {
            exercise: Exercise::VariableSets(data.clone(), exercise),
            data,
        }
    }

    pub fn with_weightset(self, name: String) -> BuildExercise {
        let data = ExerciseData {
            weightset: Some(name),
            ..self.data
        };
        BuildExercise { data, ..self }
    }

    pub fn with_weight(self, weight: f32) -> BuildExercise {
        let data = ExerciseData {
            weight: Some(weight),
            ..self.data
        };
        BuildExercise { data, ..self }
    }

    pub fn with_rest_mins(self, rest: f32) -> BuildExercise {
        let data = ExerciseData {
            rest: Some((60.0 * rest) as i32),
            ..self.data
        };
        BuildExercise { data, ..self }
    }

    pub fn with_rest_secs(self, rest: i32) -> BuildExercise {
        let data = ExerciseData {
            rest: Some(rest),
            ..self.data
        };
        BuildExercise { data, ..self }
    }

    pub fn with_last_rest_mins(self, rest: f32) -> BuildExercise {
        let data = ExerciseData {
            last_rest: Some((60.0 * rest) as i32),
            ..self.data
        };
        BuildExercise { data, ..self }
    }

    pub fn finalize(self) -> Exercise {
        match self.exercise {
            Exercise::Durations(_, exercise) => Exercise::Durations(self.data, exercise),
            Exercise::FixedReps(_, exercise) => Exercise::FixedReps(self.data, exercise),
            Exercise::VariableReps(_, exercise) => Exercise::VariableReps(self.data, exercise),
            Exercise::VariableSets(_, exercise) => Exercise::VariableSets(self.data, exercise),
        }
    }
}

impl ExerciseData {
    fn new(name: ExerciseName, formal_name: FormalName, current_set: SetIndex) -> ExerciseData {
        ExerciseData {
            name,
            formal_name,
            started: None,
            finished: false,
            current_index: current_set,
            weightset: None,
            weight: None,
            rest: None,
            last_rest: None,
        }
    }
}

impl fmt::Display for ExerciseName {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl fmt::Display for FormalName {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
