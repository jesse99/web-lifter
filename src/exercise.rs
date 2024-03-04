//! Exercises are movements for the user to perform, e.g. a barbell squat. These may be
//! shared across programs and workouts.
//!
//! They are gear-object [`Component`]s and are accessed via their ITrait interfaces,
//! e.g. [`ISummary`]`. This is quite a bit nicer than using some sort of exercise enum
//! because it's easier to add new exercise types and it's easier to customize exercises.
//! For example, some [`Durations'] instance may use weights.
mod durations;
mod iformal_name;
mod isummary;

use gear_objects::*;
use paste::paste;

pub use durations::*;
pub use iformal_name::*;
pub use isummary::*;

/// Uniquely identifies an exercise. This is assigned by the user and will be something
/// like "Light Squat".
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ExerciseName(pub String);

// TODO: make want some sort of builder pattern to optionally support stuff like weights
pub fn make_durations(
    formal_name: String,
    sets: Vec<durations::Set>,
    target_secs: i32,
) -> Component {
    let mut component = Component::new("durations");

    let exercise = Durations::new(formal_name, sets, target_secs);
    add_object!(component, Durations, exercise, [IFormalName, ISummary]);

    component
}
