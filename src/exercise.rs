//! Exercises are movements for the user to perform, e.g. a barbell squat. These may be
//! shared across programs and workouts.
//!
//! They are gear-object [`Component`]s and are accessed via their ITrait interfaces,
//! e.g. [`ISummary`]`. This is quite a bit nicer than using some sort of exercise enum
//! because it's easier to add new exercise types and it's easier to customize exercises.
//! For example, some [`Durations'] instance may use weights.
mod durations;
mod fixed_reps;
mod formal_name;
mod isummary;
mod last_rest;
mod rest;
mod target_secs;

use gear_objects::*;
use paste::paste;

pub use durations::*;
pub use fixed_reps::*;
pub use formal_name::*;
pub use isummary::*;
pub use last_rest::*;
pub use rest::*;
pub use target_secs::*;

/// Uniquely identifies an exercise. This is assigned by the user and will be something
/// like "Light Squat".
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ExerciseName(pub String);

pub struct BuildExercise {
    component: Component,
}

impl BuildExercise {
    pub fn with_target_secs(self, target: i32) -> BuildExercise {
        let mut component = self.component;
        let target = TargetSecs::new(target);
        add_object!(component, TargetSecs, target, [ITargetSecs]);
        BuildExercise { component }
    }

    pub fn with_rest(self, secs: i32) -> BuildExercise {
        let mut component = self.component;
        let rest = Rest::new(secs);
        add_object!(component, Rest, rest, [IRest]);
        BuildExercise { component }
    }

    pub fn with_last_rest(self, secs: i32) -> BuildExercise {
        let mut component = self.component;
        let rest = LastRest::new(secs);
        add_object!(component, LastRest, rest, [ILastRest]);
        BuildExercise { component }
    }

    pub fn finalize(self) -> Component {
        self.component
    }
}

pub fn build_durations(formal_name: String, secs: Vec<i32>) -> BuildExercise {
    let mut component = Component::new("durations");

    let exercise = Durations::new(secs);
    add_object!(component, Durations, exercise, [ISummary]);

    let formal_name = FormalName::new(formal_name);
    add_object!(component, FormalName, formal_name, [IFormalName]);

    BuildExercise { component }
}

pub fn build_fixed_reps(formal_name: String, reps: Vec<i32>) -> BuildExercise {
    let mut component = Component::new("fixed reps");

    let exercise = FixedReps::new(reps);
    add_object!(component, FixedReps, exercise, [ISummary]);

    let formal_name = FormalName::new(formal_name);
    add_object!(component, FormalName, formal_name, [IFormalName]);

    BuildExercise { component }
}
