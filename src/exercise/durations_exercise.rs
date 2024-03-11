/// Used for stuff like 3x60s planks. Target is used to signal the user to increase
/// difficulty (typically by switching to a harder variant of the exercise or adding
/// weight).
#[derive(Clone, Debug)]
pub struct DurationsExercise {
    secs: Vec<i32>,
    target_secs: Option<i32>,
}

impl DurationsExercise {
    // TODO: do we want a validator here?
    pub fn new(secs: Vec<i32>) -> DurationsExercise {
        DurationsExercise {
            secs,
            target_secs: None,
        }
    }

    pub fn with_target_secs(self, secs: i32) -> DurationsExercise {
        DurationsExercise {
            target_secs: Some(secs),
            ..self
        }
    }

    pub fn sets(&self) -> &Vec<i32> {
        &self.secs
    }
}
