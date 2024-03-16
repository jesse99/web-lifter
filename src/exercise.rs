//! Exercises are movements for the user to perform, e.g. a barbell squat. These may be
//! shared across programs and workouts.
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SetState {
    Implicit, // user does the exercise then presses Next button
    Timed,    // user presses Start button (aka Next), waits for timer, then does optional rest
    Finished, // user has done every set
}

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
    pub state: SetState,
    pub current_set: SetIndex,
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

    pub fn weight(&self, index: SetIndex) -> Option<f32> {
        match self {
            Exercise::Durations(_, _, _, s) => s.weight,
            Exercise::FixedReps(_, _, e, s) => {
                let percent = e.set(index).percent as f32;
                s.weight.map(|w| (percent * w) / 100.0)
            }
            Exercise::VariableReps(_, _, _, s) => s.weight,
        }
    }

    pub fn advance_weight(&mut self) {
        match self {
            Exercise::Durations(_, _, _, s) => s.weight = s.weight.map(|w| w + 5.0), // TODO need to use a weight set
            Exercise::FixedReps(_, _, _, s) => s.weight = s.weight.map(|w| w + 5.0),
            Exercise::VariableReps(_, _, _, s) => s.weight = s.weight.map(|w| w + 5.0),
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
                SetIndex::Warmup(i) => get(i, e.num_warmups(), s),
                SetIndex::Workset(i) => get(i, e.num_worksets(), s),
            },
            Exercise::VariableReps(_, _, e, s) => match index {
                SetIndex::Workset(i) => get(i, e.num_sets(), s),
                _ => None,
            },
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
        let state = SetState::Timed;
        let sets = Sets::new(state, SetIndex::Workset(0));
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
        let state = SetState::Implicit;
        let sets = if exercise.num_warmups() > 0 {
            Sets::new(state, SetIndex::Warmup(0))
        } else {
            Sets::new(state, SetIndex::Workset(0))
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
        let state = SetState::Implicit;
        let sets = Sets::new(state, SetIndex::Workset(0));
        SetsExercise {
            exercise: Exercise::VariableReps(name, formal_name, exercise, sets.clone()),
            sets,
        }
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
    fn new(state: SetState, current_set: SetIndex) -> Sets {
        Sets {
            state,
            current_set,
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
