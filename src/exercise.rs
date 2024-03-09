//! Exercises are movements for the user to perform, e.g. a barbell squat. These may be
//! shared across programs and workouts.
//!
//! They are gear-object [`Component`]s and are accessed via their ITrait interfaces,
//! e.g. [`ISummary`]`. This is quite a bit nicer than using some sort of exercise enum
//! because it's easier to add new exercise types and it's easier to customize exercises.
//! For example, some [`Durations'] instance may use weights.
use core::fmt;
use gear_objects::*;
use paste::paste;
use std::fmt::Formatter;

mod exercise_interfaces;
mod exercise_objects;
mod workout_interfaces;
mod workout_objects;

pub use exercise_interfaces::*;
pub use exercise_objects::*;
pub use workout_interfaces::*;
pub use workout_objects::*;

/// Uniquely identifies an exercise. This is assigned by the user and will be something
/// like "Light Squat".
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ExerciseName(pub String);

pub struct BuildExercise {
    exercise: Component,
    workout: Component,
}

impl BuildExercise {
    pub fn with_target_secs(self, target: i32) -> BuildExercise {
        let mut exercise = self.exercise;
        let target = TargetSecs::new(target);
        add_object!(exercise, TargetSecs, target, [ITargetSecs]);
        BuildExercise { exercise, ..self }
    }

    pub fn with_rest(self, secs: i32) -> BuildExercise {
        let mut exercise = self.exercise;
        let rest = Rest::new(secs);
        add_object!(exercise, Rest, rest, [IRest]);
        BuildExercise { exercise, ..self }
    }

    pub fn with_last_rest(self, secs: i32) -> BuildExercise {
        let mut exercise = self.exercise;
        let rest = LastRest::new(secs);
        add_object!(exercise, LastRest, rest, [ILastRest]);
        BuildExercise { exercise, ..self }
    }

    pub fn finalize(self) -> (Component, Component) {
        (self.exercise, self.workout)
    }
}

pub fn build_durations(formal_name: String, secs: Vec<i32>) -> BuildExercise {
    let mut exercise = Component::new("durations");
    let mut workout = Component::new("durations");

    let num_sets = secs.len();
    let durations = Durations::new(secs);
    add_object!(exercise, Durations, durations, [IDurations]);

    let formal_name = FormalName::new(formal_name);
    add_object!(exercise, FormalName, formal_name, [IFormalName]);

    let set = Set::new(num_sets as i32);
    add_object!(workout, Set, set, [ISetDetails]);

    BuildExercise { exercise, workout }
}

pub fn build_fixed_reps(formal_name: String, reps: Vec<i32>) -> BuildExercise {
    let mut exercise = Component::new("fixed reps");
    let mut workout = Component::new("fixed reps");

    let num_sets = reps.len();
    let freps = FixedReps::new(reps);
    add_object!(exercise, FixedReps, freps, [IFixedReps]);

    let formal_name = FormalName::new(formal_name);
    add_object!(exercise, FormalName, formal_name, [IFormalName]);

    let set = Set::new(num_sets as i32);
    add_object!(workout, Set, set, [ISetDetails]);

    BuildExercise { exercise, workout }
}

impl fmt::Display for ExerciseName {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
