//! Exercises are movements for the user to perform, e.g. a barbell squat. These may be
//! shared across programs and workouts.
use crate::*;
use core::fmt;
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
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ExerciseName(pub String);

/// The proper exercise name, e.g. "Low-bar Squat". This is used to show help for the
/// exercise.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct FormalName(pub String);

/// Not all exercises will support all of these.
#[derive(Clone, Copy, Debug)]
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

/// Used for exercises that are done multiple times, often using rest between sets.
#[derive(Clone, Debug)]
pub struct Sets {
    // TODO may want to rename this something like ExerciseOptions or SharedOptions
    pub finished: bool,
    pub current_index: SetIndex,
    weightset: Option<String>,
    weight: Option<f32>, // base weight to use for each workset, often modified by set percent
    rest: Option<i32>,   // used for work sets
    last_rest: Option<i32>, // overrides rest.last()
}

#[derive(Debug)]
pub enum Exercise {
    Durations(ExerciseName, FormalName, DurationsExercise, Sets),
    FixedReps(ExerciseName, FormalName, FixedRepsExercise, Sets),
    VariableReps(ExerciseName, FormalName, VariableRepsExercise, Sets),
}

impl Exercise {
    pub fn name(&self) -> &ExerciseName {
        match self {
            Exercise::Durations(name, _, _, _) => name,
            Exercise::FixedReps(name, _, _, _) => name,
            Exercise::VariableReps(name, _, _, _) => name,
        }
    }

    pub fn expect_durations(&self) -> (&DurationsExercise, &Sets) {
        match self {
            Exercise::Durations(_, _, e, s) => (e, s),
            _ => panic!("expected durations"),
        }
    }

    pub fn expect_fixed_reps(&self) -> (&FixedRepsExercise, &Sets) {
        match self {
            Exercise::FixedReps(_, _, e, s) => (e, s),
            _ => panic!("expected fixed reps"),
        }
    }

    pub fn expect_var_reps(&self) -> (&VariableRepsExercise, &Sets) {
        match self {
            Exercise::VariableReps(_, _, e, s) => (e, s),
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
            Exercise::Durations(_, _, _, s) => s.weight = weight,
            Exercise::FixedReps(_, _, _, s) => s.weight = weight,
            Exercise::VariableReps(_, _, _, s) => s.weight = weight,
        }
    }

    /// For the specified set, in seconds.
    pub fn rest(&self, index: SetIndex) -> Option<i32> {
        fn get(index: usize, num: usize, s: &Sets) -> Option<i32> {
            if index + 1 == num && s.last_rest.is_some() {
                s.last_rest
            } else {
                s.rest
            }
        }

        match self {
            Exercise::Durations(_, _, e, s) => match index {
                SetIndex::Workset(i) => get(i, e.num_sets(), s),
                _ => None,
            },
            Exercise::FixedReps(_, _, e, s) => match index {
                SetIndex::Workset(i) => get(i, e.num_worksets(), s),
                _ => None,
            },
            Exercise::VariableReps(_, _, e, s) => match index {
                SetIndex::Workset(i) => get(i, e.num_worksets(), s),
                _ => None,
            },
        }
    }

    fn target_weight(&self, index: SetIndex) -> (Option<f32>, &Option<String>) {
        match self {
            Exercise::Durations(_, _, _, s) => (s.weight, &s.weightset),
            Exercise::FixedReps(_, _, e, s) => {
                let percent = e.set(index).percent as f32;
                (s.weight.map(|w| (percent * w) / 100.0), &s.weightset)
            }
            Exercise::VariableReps(_, _, e, s) => {
                let percent = e.expected_range(index).percent as f32;
                (s.weight.map(|w| (percent * w) / 100.0), &s.weightset)
            }
        }
    }

    fn base_weight(&self) -> (Option<f32>, &Option<String>) {
        match self {
            Exercise::Durations(_, _, _, s) => (s.weight, &s.weightset),
            Exercise::FixedReps(_, _, _, s) => (s.weight, &s.weightset),
            Exercise::VariableReps(_, _, _, s) => (s.weight, &s.weightset),
        }
    }
}

/// Builder for ['Exercise`]'s that use sets.
pub struct SetsExercise {
    exercise: Exercise,
    sets: Sets,
}

impl SetsExercise {
    pub fn durations(
        name: ExerciseName,
        formal_name: FormalName,
        exercise: DurationsExercise,
    ) -> SetsExercise {
        let sets = Sets::new(SetIndex::Workset(0));
        SetsExercise {
            exercise: Exercise::Durations(name, formal_name, exercise, sets.clone()),
            sets,
        }
    }

    pub fn fixed_reps(
        name: ExerciseName,
        formal_name: FormalName,
        exercise: FixedRepsExercise,
    ) -> SetsExercise {
        let sets = if exercise.num_warmups() > 0 {
            Sets::new(SetIndex::Warmup(0))
        } else {
            Sets::new(SetIndex::Workset(0))
        };
        SetsExercise {
            exercise: Exercise::FixedReps(name, formal_name, exercise, sets.clone()),
            sets,
        }
    }

    pub fn variable_reps(
        name: ExerciseName,
        formal_name: FormalName,
        exercise: VariableRepsExercise,
    ) -> SetsExercise {
        let sets = Sets::new(SetIndex::Workset(0));
        SetsExercise {
            exercise: Exercise::VariableReps(name, formal_name, exercise, sets.clone()),
            sets,
        }
    }

    pub fn with_weightset(self, name: String) -> SetsExercise {
        let sets = Sets {
            weightset: Some(name),
            ..self.sets
        };
        SetsExercise { sets, ..self }
    }

    pub fn with_weight(self, weight: f32) -> SetsExercise {
        let sets = Sets {
            weight: Some(weight),
            ..self.sets
        };
        SetsExercise { sets, ..self }
    }

    pub fn with_rest(self, rest: i32) -> SetsExercise {
        let sets = Sets {
            rest: Some(rest),
            ..self.sets
        };
        SetsExercise { sets, ..self }
    }

    // pub fn with_last_rest(self, last: i32) -> SetsExercise {
    //     let sets = Sets {
    //         last_rest: Some(last),
    //         ..self.sets
    //     };
    //     SetsExercise { sets, ..self }
    // }

    pub fn finalize(self) -> Exercise {
        match self.exercise {
            Exercise::Durations(name, fname, exercise, _) => {
                Exercise::Durations(name, fname, exercise, self.sets)
            }
            Exercise::FixedReps(name, fname, exercise, _) => {
                Exercise::FixedReps(name, fname, exercise, self.sets)
            }
            Exercise::VariableReps(name, fname, exercise, _) => {
                Exercise::VariableReps(name, fname, exercise, self.sets)
            }
        }
    }
}

impl Sets {
    fn new(current_set: SetIndex) -> Sets {
        Sets {
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
