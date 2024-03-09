//! Exercises are movements for the user to perform, e.g. a barbell squat. These may be
//! shared across programs and workouts.
use core::fmt;
use std::fmt::Formatter;

mod durations_exercise;
mod exercise_instance;
mod fixed_reps_exercise;

pub use durations_exercise::*;
pub use exercise_instance::*;
pub use fixed_reps_exercise::*;

/// Uniquely identifies an exercise. This is assigned by the user and will be something
/// like "Light Squat".
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ExerciseName(pub String);

#[derive(Debug)]
pub enum Exercise {
    Durations(DurationsExercise),
    FixedReps(FixedRepsExercise),
}

impl Exercise {
    pub fn num_sets(&self) -> i32 {
        match self {
            Exercise::Durations(exercise) => exercise.sets().len() as i32,
            Exercise::FixedReps(exercise) => exercise.sets().len() as i32,
        }
    }
}

impl fmt::Display for ExerciseName {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
