#[derive(Clone, Copy, Debug)]
pub struct VariableReps {
    pub min: i32,
    pub max: i32,
    pub percent: i32,
}

impl VariableReps {
    pub fn new(min: i32, max: i32, percent: i32) -> VariableReps {
        assert!(min <= max);
        VariableReps { min, max, percent }
    }
}

/// Used for stuff like 4-8 squats.
#[derive(Clone, Debug)]
pub struct VariableRepsExercise {
    reps: Vec<VariableReps>,
    expected: Vec<i32>,
}

impl VariableRepsExercise {
    // TODO: do we want a validator here?
    pub fn new(reps: Vec<VariableReps>) -> VariableRepsExercise {
        VariableRepsExercise {
            reps,
            expected: Vec::new(),
        }
    }

    /// Min and max reps for each set.
    pub fn sets(&self) -> &Vec<VariableReps> {
        &self.reps
    }

    pub fn set_expected(&mut self, expected: Vec<i32>) {
        self.expected = expected;
    }

    /// Minimum the user is currently expected to do.
    pub fn min_expected(&self) -> &Vec<i32> {
        &self.expected
    }

    /// Maximum the user is expected to do.
    pub fn max_expected(&self) -> Vec<i32> {
        self.reps.iter().map(|r| r.max).collect()
    }

    /// What the user wants to do up to the most the user is expected to do. Can be larger
    /// than what they did last time, or sometimes even smaller.
    pub fn expected_range(&self, set_index: i32) -> VariableReps {
        let set_index = set_index as usize;
        let (min, max, percent) = if set_index < self.expected.len() && set_index < self.reps.len()
        {
            (
                std::cmp::min(self.expected[set_index], self.reps[set_index].max),
                self.reps[set_index].max,
                self.reps[set_index].percent,
            )
        } else if set_index < self.reps.len() {
            // Typically this happens if expected is empty. Possibly also number of sets
            // was changed (although doing that should reset expected).
            (
                self.reps[set_index].min,
                self.reps[set_index].max,
                self.reps[set_index].percent,
            )
        } else {
            assert!(false);
            (4, 8, 100)
        };
        VariableReps { min, max, percent }
    }
}
