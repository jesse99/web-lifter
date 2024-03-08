use crate::*;
use gear_objects::*;
use paste::paste;

/// Used for stuff like 3x60s planks. Target is used to signal the user to increase
/// difficulty (typically by switching to a harder variant of the exercise or adding
/// weight).
pub struct FixedReps {
    reps: Vec<i32>,
}
register_type!(FixedReps);

impl FixedReps {
    // TODO: do we want a validator here?
    pub fn new(reps: Vec<i32>) -> FixedReps {
        FixedReps { reps }
    }
}

impl ISummary for FixedReps {
    fn summary(&self) -> String {
        format!("{}x{}", self.reps.len(), self.reps[0]) // TODO: do better here
    }
}
