/// Used for stuff like 3x12 crunches.
#[derive(Debug)]
pub struct FixedRepsExercise {
    reps: Vec<i32>,
}

impl FixedRepsExercise {
    // TODO: do we want a validator here?
    pub fn new(reps: Vec<i32>) -> FixedRepsExercise {
        FixedRepsExercise { reps }
    }

    pub fn sets(&self) -> &Vec<i32> {
        &self.reps
    }
}
