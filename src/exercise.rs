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

/// Used for exercises that are done multiple times, often using rest between sets.
#[derive(Clone, Debug)]
pub struct Sets {
    pub state: SetState,
    pub current_set: i32,
    pub num_sets: i32,
    pub weight: Option<f32>, // base weight to use for each set, often modified by set percent
    pub rest: Option<i32>,
    pub last_rest: Option<i32>, // overrides rest.last()
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

    pub fn current_set(&self) -> Option<(i32, i32)> {
        match self {
            Exercise::Durations(_, _, _, sets) => Some((sets.current_set, sets.num_sets)),
            Exercise::FixedReps(_, _, _, sets) => Some((sets.current_set, sets.num_sets)),
            Exercise::VariableReps(_, _, _, sets) => Some((sets.current_set, sets.num_sets)),
        }
    }

    pub fn current_weight(&self) -> Option<f32> {
        match self {
            Exercise::Durations(_, _, _, sets) => sets.weight,
            Exercise::FixedReps(_, _, e, sets) => {
                let percent = e.worksets()[sets.current_set as usize].percent as f32;
                sets.weight.map(|w| (percent * w) / 100.0)
            }
            Exercise::VariableReps(_, _, _, sets) => sets.weight,
        }
    }

    /// For the current set, in seconds.
    pub fn rest(&self) -> Option<i32> {
        let sets = match self {
            Exercise::Durations(_, _, _, sets) => sets,
            Exercise::FixedReps(_, _, _, sets) => sets,
            Exercise::VariableReps(_, _, _, sets) => sets,
        };
        if sets.current_set + 1 == sets.num_sets && sets.last_rest.is_some() {
            sets.last_rest
        } else {
            sets.rest
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
        let num_sets = exercise.sets().len() as i32;
        let dummy = Sets::new(state, 0);
        SetsExercise {
            exercise: Exercise::Durations(name, formal_name, exercise, dummy),
            sets: Sets::new(state, num_sets),
        }
    }

    pub fn fixed_reps(
        name: ExerciseName,
        formal_name: FormalName,
        exercise: FixedRepsExercise,
    ) -> SetsExercise {
        let state = SetState::Implicit;
        let num_sets = exercise.worksets().len() as i32;
        let dummy = Sets::new(state, 0);
        SetsExercise {
            exercise: Exercise::FixedReps(name, formal_name, exercise, dummy),
            sets: Sets::new(state, num_sets),
        }
    }

    pub fn variable_reps(
        name: ExerciseName,
        formal_name: FormalName,
        exercise: VariableRepsExercise,
    ) -> SetsExercise {
        let state = SetState::Implicit;
        let num_sets = exercise.sets().len() as i32;
        let dummy = Sets::new(state, 0);
        SetsExercise {
            exercise: Exercise::VariableReps(name, formal_name, exercise, dummy),
            sets: Sets::new(state, num_sets),
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
    fn new(state: SetState, num_sets: i32) -> Sets {
        Sets {
            state,
            current_set: 0,
            num_sets,
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
