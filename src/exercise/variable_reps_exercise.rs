use super::FixedReps;
use crate::validation_err;
use crate::{exercise::SetIndex, pages::Error};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
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
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VariableRepsExercise {
    warmups: Vec<FixedReps>,
    worksets: Vec<VariableReps>,
    expected: Vec<i32>,
}

impl VariableRepsExercise {
    // TODO: do we want a validator here?
    pub fn new(warmups: Vec<FixedReps>, worksets: Vec<VariableReps>) -> VariableRepsExercise {
        VariableRepsExercise {
            warmups,
            worksets,
            expected: Vec::new(),
        }
    }

    pub fn num_warmups(&self) -> usize {
        self.warmups.len()
    }

    pub fn num_worksets(&self) -> usize {
        self.worksets.len()
    }

    pub fn set_expected(&mut self, expected: Vec<i32>) {
        assert!(expected.len() == self.worksets.len());
        self.expected = expected;
    }

    /// Minimum the user is currently expected to do.
    pub fn min_expected(&self) -> Vec<i32> {
        self.worksets.iter().map(|r| r.min).collect()
    }

    /// Maximum the user is expected to do.
    pub fn max_expected(&self) -> Vec<i32> {
        self.worksets.iter().map(|r| r.max).collect()
    }

    pub fn expected(&self) -> &Vec<i32> {
        &self.expected
    }

    /// What the user wants to do up to the most the user is expected to do. Can be larger
    /// than what they did last time, or sometimes even smaller.
    pub fn expected_range(&self, index: SetIndex) -> VariableReps {
        match index {
            SetIndex::Warmup(i) => VariableReps::new(
                self.warmups[i].reps,
                self.warmups[i].reps,
                self.warmups[i].percent,
            ),
            SetIndex::Workset(i) => {
                let (min, max, percent) = if i < self.expected.len() && i < self.worksets.len() {
                    (
                        std::cmp::min(self.expected[i], self.worksets[i].max),
                        self.worksets[i].max,
                        self.worksets[i].percent,
                    )
                } else if i < self.worksets.len() {
                    // Typically this happens if expected is empty. Possibly also number of sets
                    // was changed (although doing that should reset expected).
                    (
                        self.worksets[i].min,
                        self.worksets[i].max,
                        self.worksets[i].percent,
                    )
                } else {
                    assert!(false);
                    (4, 8, 100)
                };
                VariableReps { min, max, percent }
            }
        }
    }

    pub fn warmup(&self, index: usize) -> &FixedReps {
        &self.warmups[index]
    }

    pub fn workset(&self, index: usize) -> &VariableReps {
        &self.worksets[index]
    }

    pub fn try_set_warmups(&mut self, warmups: Vec<FixedReps>) -> Result<(), Error> {
        self.validate_warmups(&warmups)?;
        self.do_set_warmups(warmups);
        Ok(())
    }

    // pub fn set_warmups(&mut self, warmups: Vec<FixedReps>) {
    //     assert!(self.validate_warmups(&warmups).is_ok());
    //     self.do_set_warmups(warmups);
    // }

    pub fn try_set_worksets(&mut self, worksets: Vec<VariableReps>) -> Result<(), Error> {
        self.validate_worksets(&worksets)?;
        self.do_set_worksets(worksets);
        Ok(())
    }

    // pub fn set_worksets(&mut self, worksets: Vec<VariableReps>) {
    //     assert!(self.validate_worksets(&worksets).is_ok());
    //     self.do_set_worksets(worksets);
    // }

    fn validate_warmups(&self, warmups: &Vec<FixedReps>) -> Result<(), Error> {
        for set in warmups {
            if set.reps < 0 {
                return validation_err!("warmup reps cannot be negative");
            }
            if set.reps == 0 {
                return validation_err!("warmup reps cannot be zero");
            }
            if set.percent < 0 {
                // 0 percent is OK (for warmups)
                return validation_err!("warmup percent cannot be negative");
            }
            if set.percent >= 100 {
                return validation_err!("warmup percent should be less than 100%",);
            }
        }
        Ok(())
    }

    fn validate_worksets(&self, worksets: &Vec<VariableReps>) -> Result<(), Error> {
        if worksets.is_empty() {
            return validation_err!("worksets cannot be empty");
        }
        for set in worksets {
            if set.min < 0 {
                return validation_err!("workset min reps cannot be negative");
            }
            if set.min == 0 {
                return validation_err!("workset min reps cannot be zero");
            }
            if set.min > set.max {
                return validation_err!("workset min reps should be <= max reps",);
            }
            if set.percent < 0 {
                return validation_err!("workset percent cannot be negative");
            }
            if set.percent == 0 {
                return validation_err!("workset percent should not be zero");
            }
            if set.percent < 0 {
                // over 100% is OK (tho not common)
                return validation_err!("workset percent cannot be negative");
            }
        }
        Ok(())
    }

    fn do_set_warmups(&mut self, warmups: Vec<FixedReps>) {
        self.warmups = warmups;
    }

    fn do_set_worksets(&mut self, worksets: Vec<VariableReps>) {
        self.worksets = worksets;
        self.expected = Vec::new();
    }
}
