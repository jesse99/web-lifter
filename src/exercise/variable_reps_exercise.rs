#[derive(Clone, Copy, Debug)]
pub struct RepRange {
    pub min: i32,
    pub max: i32,
}

impl RepRange {
    pub fn new(min: i32, max: i32) -> RepRange {
        assert!(min <= max);
        RepRange { min, max }
    }
}

/// Used for stuff like 4-8 squats.
#[derive(Clone, Debug)]
pub struct VariableRepsExercise {
    reps: Vec<RepRange>,
    expected: Vec<i32>,
}

impl VariableRepsExercise {
    // TODO: do we want a validator here?
    pub fn new(reps: Vec<RepRange>) -> VariableRepsExercise {
        VariableRepsExercise {
            reps,
            expected: Vec::new(),
        }
    }

    /// Min and max reps for each set.
    pub fn sets(&self) -> &Vec<RepRange> {
        &self.reps
    }

    /// What the user wants to do up to the most he user is expected to do. Can be larger
    /// than what they did last time, or sometimes even smaller.
    pub fn expected(&self, set_index: i32) -> RepRange {
        let set_index = set_index as usize;
        let (min, max) = if set_index < self.expected.len() && set_index < self.reps.len() {
            (
                std::cmp::min(self.expected[set_index], self.reps[set_index].max),
                self.reps[set_index].max,
            )
        } else if set_index < self.reps.len() {
            // Typically this happens if expected is empty. Possibly also number of sets
            // was changed (although doing that should reset expected).
            (self.reps[set_index].min, self.reps[set_index].max)
        } else {
            assert!(false);
            (4, 8)
        };
        RepRange { min, max }
    }
}
