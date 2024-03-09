//! Workout specific data associated with an [`Exercise`]. This is for state which changes
//! as the user performs an exercise or is affected by exercise ordering.

/// Part of a [`Workout`] and associated with an [`Exercise`].
pub struct ExerciseInstance {
    // TOODO: should have optional expected reps array (for non-fixed reps and maybe even FixedReps)
    set: SetDetails,
    weight: Option<Vec<f32>>, // weight to use for each set
    rest: Option<i32>,
    last_rest: Option<i32>, // overrides rest
}

impl ExerciseInstance {
    pub fn new(num_sets: i32) -> ExerciseInstance {
        ExerciseInstance {
            set: SetDetails::new(num_sets),
            weight: None,
            rest: None,
            last_rest: None,
        }
    }

    pub fn with_weight(self, weight: Vec<f32>) -> ExerciseInstance {
        ExerciseInstance {
            weight: Some(weight),
            ..self
        }
    }

    pub fn with_rest(self, rest: i32) -> ExerciseInstance {
        ExerciseInstance {
            rest: Some(rest),
            ..self
        }
    }

    pub fn with_last_rest(self, last: i32) -> ExerciseInstance {
        ExerciseInstance {
            last_rest: Some(last),
            ..self
        }
    }

    pub fn current_set(&self) -> SetDetails {
        self.set
    }
}
// =======================================================================================

#[derive(Clone, Copy)]
pub struct SetDetails {
    pub current_set: i32,
    pub num_sets: i32, // TODO make sure this stays in sync when exercise is edited
}

impl SetDetails {
    pub fn new(num_sets: i32) -> SetDetails {
        SetDetails {
            current_set: 0,
            num_sets,
        }
    }
}
