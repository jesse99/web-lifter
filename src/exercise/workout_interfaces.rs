//! These are the interfaces associated with exercises within a [`Workout`]. They are for
//! state which changes as the user performs an exercise or is affected by exercise
//! ordering. Note that we call an exercise within a workout an instance of the exercise.
use gear_objects::*;
use paste::paste;

#[derive(Copy, Clone)]
pub struct SetDetails {
    /// index for set being executed
    pub index: i32,

    /// total number of sets
    pub count: i32,
}

/// Where the user is at currently within the exercise.
pub trait ISetDetails {
    fn expected(&self) -> SetDetails;
}
register_type!(ISetDetails);
// =======================================================================================

/// Weight the user is expected to use for each set.
pub trait IExpectedWeight {
    fn expected(&self) -> Vec<f32>;
}
register_type!(IExpectedWeight);
// =======================================================================================

/// Amount of time to rest after each set. The amount of time to rest for the last set
/// can be over-ridden using ILastSet.
pub trait IRest {
    fn rest(&self) -> i32;
}
register_type!(IRest);
// =======================================================================================

/// Optional trait overriding the IRest trait for the last set.
pub trait ILastRest {
    fn rest(&self) -> i32;
}
register_type!(ILastRest);
// =======================================================================================

// TODO add IExpectedReps (for variable reps or possibly to override IFixedReps)
