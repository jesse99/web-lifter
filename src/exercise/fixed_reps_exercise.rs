use crate::*;
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

    pub fn set(&self, index: SetIndex) -> &FixedReps {
        match index {
            SetIndex::Warmup(i) => &self.warmups[i],
            SetIndex::Workset(i) => &self.worksets[i],
        }
    }
}
