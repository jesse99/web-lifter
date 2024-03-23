//! Exercises are movements for the user to perform, e.g. a barbell squat. These may be
//! shared across programs and workouts.
use crate::*;
use core::fmt;
use serde::{Deserialize, Serialize};
use std::fmt::Formatter;

mod durations_exercise;
mod fixed_reps_exercise;
mod variable_reps_exercise;

pub use durations_exercise::*;
pub use fixed_reps_exercise::*;
pub use variable_reps_exercise::*;

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
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
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
    pub finished: bool,
    pub current_index: SetIndex,
    weightset: Option<String>,
    weight: Option<f32>, // base weight to use for each workset, often modified by set percent
    rest: Option<i32>,   // used for work sets
    last_rest: Option<i32>, // overrides rest.last()
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Exercise {
    Durations(ExerciseData, DurationsExercise),
    FixedReps(ExerciseData, FixedRepsExercise),
    VariableReps(ExerciseData, VariableRepsExercise),
}

impl Exercise {
    pub fn name(&self) -> &ExerciseName {
        match self {
            Exercise::Durations(d, _) => &d.name,
            Exercise::FixedReps(d, _) => &d.name,
            Exercise::VariableReps(d, _) => &d.name,
        }
    }

    pub fn expect_durations(&self) -> (&ExerciseData, &DurationsExercise) {
        match self {
            Exercise::Durations(d, e) => (d, e),
            _ => panic!("expected durations"),
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

    pub fn set_weight(&mut self, weight: Option<f32>) {
        match self {
            Exercise::Durations(d, _) => d.weight = weight,
            Exercise::FixedReps(d, _) => d.weight = weight,
            Exercise::VariableReps(d, _) => d.weight = weight,
        }
    }

    /// For the specified set, in seconds.
    pub fn rest(&self, index: SetIndex) -> Option<i32> {
        fn get(index: usize, num: usize, s: &ExerciseData) -> Option<i32> {
            if index + 1 == num && s.last_rest.is_some() {
                s.last_rest
            } else {
                s.rest
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
        }
    }

    fn base_weight(&self) -> (Option<f32>, &Option<String>) {
        match self {
            Exercise::Durations(d, _) => (d.weight, &d.weightset),
            Exercise::FixedReps(d, _) => (d.weight, &d.weightset),
            Exercise::VariableReps(d, _) => (d.weight, &d.weightset),
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

    pub fn with_rest(self, rest: i32) -> BuildExercise {
        let data = ExerciseData {
            rest: Some(rest),
            ..self.data
        };
        BuildExercise { data, ..self }
    }

    pub fn with_last_rest(self, last: i32) -> BuildExercise {
        let data = ExerciseData {
            last_rest: Some(last),
            ..self.data
        };
        BuildExercise { data, ..self }
    }

    pub fn finalize(self) -> Exercise {
        match self.exercise {
            Exercise::Durations(_, exercise) => Exercise::Durations(self.data, exercise),
            Exercise::FixedReps(_, exercise) => Exercise::FixedReps(self.data, exercise),
            Exercise::VariableReps(_, exercise) => Exercise::VariableReps(self.data, exercise),
        }
    }
}

impl ExerciseData {
    fn new(name: ExerciseName, formal_name: FormalName, current_set: SetIndex) -> ExerciseData {
        ExerciseData {
            name,
            formal_name,
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
