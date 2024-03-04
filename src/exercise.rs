// use super::*;

/// Uniquely identifies an exercise. This is assigned by the user and will be something
/// like "Light Squat".
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ExerciseName(pub String);

/// A movement for the user to perform, e.g. a barbell squat. These may be shared across
/// programs and workouts.
#[derive(Clone, Debug)]
pub struct Exercise {
    pub formal_name: String, // the actual name, used for stuff like help, e.g. "Low-bar Squat"
}

impl Exercise {
    pub fn new(formal_name: String) -> Exercise {
        Exercise { formal_name }
    }

    /// Returns stuff like:
    /// "4x20s"
    /// "3x5 reps @ 135 lbs"
    /// "2x3-5 reps, 1-5 reps @ 135 lbs"
    pub fn summary(&self) -> String {
        "4x20s".to_owned()
    }
}
