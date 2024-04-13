// Quite similar to VariableReps but the reps are always fixed so the UI doesn't have
// a reps dropdown when performing the exerise. (And the implementation is a bit simpler
// since we don't bother with expected reps).
use crate::{exercise::SetIndex, pages::ValidationError};
use serde::{Deserialize, Serialize};

/// Reps to use for a set along with a percentage of weight.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FixedReps {
    pub reps: i32,
    pub percent: i32,
}

impl FixedReps {
    pub fn new(reps: i32, percent: i32) -> FixedReps {
        FixedReps { reps, percent }
    }
}

/// Used for stuff like 3x12 crunches.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FixedRepsExercise {
    warmups: Vec<FixedReps>,
    worksets: Vec<FixedReps>,
}

impl FixedRepsExercise {
    // TODO: do we want a validator here?
    pub fn with_reps(worksets: Vec<i32>) -> FixedRepsExercise {
        let worksets = worksets
            .iter()
            .map(|&reps| FixedReps { reps, percent: 100 })
            .collect();
        FixedRepsExercise {
            warmups: Vec::new(),
            worksets,
        }
    }

    // pub fn with_percent(warmups: Vec<FixedReps>, worksets: Vec<FixedReps>) -> FixedRepsExercise {
    //     FixedRepsExercise { warmups, worksets }
    // }

    pub fn num_warmups(&self) -> usize {
        self.warmups.len()
    }

    pub fn num_worksets(&self) -> usize {
        self.worksets.len()
    }

    pub fn worksets(&self) -> impl Iterator<Item = &FixedReps> + '_ {
        self.worksets.iter()
    }

    pub fn set(&self, index: SetIndex) -> &FixedReps {
        match index {
            SetIndex::Warmup(i) => &self.warmups[i],
            SetIndex::Workset(i) => &self.worksets[i],
        }
    }

    pub fn try_set_warmups(&mut self, warmups: Vec<FixedReps>) -> Result<(), ValidationError> {
        self.validate_warmups(&warmups)?;
        self.do_set_warmups(warmups);
        Ok(())
    }

    // pub fn set_warmups(&mut self, warmups: Vec<FixedReps>) {
    //     assert!(self.validate_warmups(&warmups).is_ok());
    //     self.do_set_warmups(warmups);
    // }

    pub fn try_set_worksets(&mut self, worksets: Vec<FixedReps>) -> Result<(), ValidationError> {
        self.validate_worksets(&worksets)?;
        self.do_set_worksets(worksets);
        Ok(())
    }

    // pub fn set_worksets(&mut self, worksets: Vec<FixedReps>) {
    //     assert!(self.validate_worksets(&worksets).is_ok());
    //     self.do_set_worksets(worksets);
    // }

    fn validate_warmups(&self, warmups: &Vec<FixedReps>) -> Result<(), ValidationError> {
        for set in warmups {
            if set.reps < 0 {
                return Err(ValidationError::new("warmup reps cannot be negative"));
            }
            if set.reps == 0 {
                return Err(ValidationError::new("warmup reps cannot be zero"));
            }
            if set.percent < 0 {
                // 0 percent is OK (for warmups)
                return Err(ValidationError::new("warmup percent cannot be negative"));
            }
            if set.percent >= 100 {
                return Err(ValidationError::new(
                    "warmup percent should be less than 100%",
                ));
            }
        }
        Ok(())
    }

    fn validate_worksets(&self, worksets: &Vec<FixedReps>) -> Result<(), ValidationError> {
        if worksets.is_empty() {
            return Err(ValidationError::new("worksets cannot be empty"));
        }
        for set in worksets {
            if set.reps < 0 {
                return Err(ValidationError::new("workset reps cannot be negative"));
            }
            if set.reps == 0 {
                return Err(ValidationError::new("workset reps cannot be zero"));
            }
            if set.percent < 0 {
                return Err(ValidationError::new("workset percent cannot be negative"));
            }
            if set.percent == 0 {
                return Err(ValidationError::new("workset percent should not be zero"));
            }
            if set.percent < 0 {
                // over 100% is OK (tho not common)
                return Err(ValidationError::new("workset percent cannot be negative"));
            }
        }
        Ok(())
    }

    fn do_set_warmups(&mut self, warmups: Vec<FixedReps>) {
        self.warmups = warmups;
    }

    fn do_set_worksets(&mut self, worksets: Vec<FixedReps>) {
        self.worksets = worksets;
    }
}
