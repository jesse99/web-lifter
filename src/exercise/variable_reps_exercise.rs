use crate::*;

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

    pub fn num_sets(&self) -> usize {
        self.reps.len()
    }

    pub fn set_expected(&mut self, expected: Vec<i32>) {
        self.expected = expected;
    }

    /// Minimum the user is currently expected to do.
    pub fn min_expected(&self) -> Vec<i32> {
        self.reps.iter().map(|r| r.min).collect()
    }

    /// Maximum the user is expected to do.
    pub fn max_expected(&self) -> Vec<i32> {
        self.reps.iter().map(|r| r.max).collect()
    }

    pub fn expected(&self) -> &Vec<i32> {
        &self.expected
    }

    /// What the user wants to do up to the most the user is expected to do. Can be larger
    /// than what they did last time, or sometimes even smaller.
    pub fn expected_range(&self, index: SetIndex) -> VariableReps {
        match index {
            SetIndex::Warmup(_) => todo!(),
            SetIndex::Workset(i) => {
                let (min, max, percent) = if i < self.expected.len() && i < self.reps.len() {
                    (
                        std::cmp::min(self.expected[i], self.reps[i].max),
                        self.reps[i].max,
                        self.reps[i].percent,
                    )
                } else if i < self.reps.len() {
                    // Typically this happens if expected is empty. Possibly also number of sets
                    // was changed (although doing that should reset expected).
                    (self.reps[i].min, self.reps[i].max, self.reps[i].percent)
                } else {
                    assert!(false);
                    (4, 8, 100)
                };
                VariableReps { min, max, percent }
            }
        }
    }
}
