/// Used for stuff like 3x12 crunches.
#[derive(Debug)]
pub struct FixedRepsExercise {
    formal_name: String,
    reps: Vec<i32>,
}

impl FixedRepsExercise {
    // TODO: do we want a validator here?
    pub fn new(formal_name: String, reps: Vec<i32>) -> FixedRepsExercise {
        FixedRepsExercise { formal_name, reps }
    }

    pub fn sets(&self) -> &Vec<i32> {
        &self.reps
    }
}
