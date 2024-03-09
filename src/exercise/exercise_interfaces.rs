//! These are the interfaces associated with exercises within [`Exercises`]. They are for
//! state which doesn't change as the user performs an exercise.
use gear_objects::*;
use paste::paste;

/// Number of seconds the user is expected to spend for each set, e.g. 3x30s planks.
pub trait IDurations {
    fn expected(&self) -> &Vec<i32>;
}
register_type!(IDurations);
// =======================================================================================

/// Optional goal seconds. Can be used with a durations exercise: when the user hits the
/// target they will typically switch to a harder version of the exercise or add weight.
pub trait ITargetSecs {
    fn target(&self) -> i32;
}
register_type!(ITargetSecs);
// =======================================================================================

/// Number of reps user is expected to perform for each set, e.g. 3x5 reps.
pub trait IFixedReps {
    fn expected(&self) -> &Vec<i32>;
}
register_type!(IFixedReps);
// =======================================================================================

/// The canonical name for an exercise, e.g. "Low-bar Squat". This is used to lookup notes
/// for an exercise but not to identify an exercise within a workout.
pub trait IFormalName {
    fn formal_name(&self) -> &str;
}
register_type!(IFormalName);
// =======================================================================================
