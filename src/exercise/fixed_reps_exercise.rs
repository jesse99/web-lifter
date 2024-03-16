/// Reps to use for a set along with a percentage of weight.
#[derive(Clone, Debug)]
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
#[derive(Clone, Debug)]
pub struct FixedRepsExercise {
    warmup: Vec<FixedReps>,
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
            warmup: Vec::new(),
            worksets,
        }
    }

    pub fn with_percent(warmup: Vec<FixedReps>, worksets: Vec<FixedReps>) -> FixedRepsExercise {
        FixedRepsExercise { warmup, worksets }
    }

    pub fn warmup(&self) -> &Vec<FixedReps> {
        &self.warmup
    }

    pub fn worksets(&self) -> &Vec<FixedReps> {
        &self.worksets
    }
}
