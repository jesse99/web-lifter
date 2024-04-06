use crate::{exercise::SetIndex, pages::ValidationError};
use serde::{Deserialize, Serialize};

/// Used for stuff like 3x60s planks. Target is used to signal the user to increase
/// difficulty (typically by switching to a harder variant of the exercise or adding
/// weight).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DurationsExercise {
    secs: Vec<i32>,
    target_secs: Option<i32>, // TODO: support this
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

    pub fn num_sets(&self) -> usize {
        self.secs.len()
    }

    pub fn target(&self) -> Option<i32> {
        self.target_secs
    }

    pub fn set(&self, index: SetIndex) -> i32 {
        match index {
            SetIndex::Workset(i) => self.secs[i],
            _ => panic!("expected workset"),
        }
    }

    pub fn try_set_durations(&mut self, durations: Vec<i32>) -> Result<(), ValidationError> {
        self.validate_durations(&durations)?;
        self.do_set_durations(durations);
        Ok(())
    }

    // pub fn set_durations(&mut self, durations: Vec<i32>) {
    //     assert!(self.validate_durations(&durations).is_ok());
    //     self.do_set_durations(durations);
    // }

    pub fn try_set_target(&mut self, target: Option<i32>) -> Result<(), ValidationError> {
        self.validate_target(target)?;
        self.do_set_target(target);
        Ok(())
    }

    // pub fn set_target(&mut self, target: Option<i32>) {
    //     assert!(self.validate_target(target).is_ok());
    //     self.do_set_target(target);
    // }

    fn validate_durations(&self, durations: &Vec<i32>) -> Result<(), ValidationError> {
        if durations.is_empty() {
            return Err(ValidationError::new("durations cannot be empty"));
        }
        for duration in durations {
            if *duration < 0 {
                return Err(ValidationError::new("duration cannot be negative"));
            }
            if *duration == 0 {
                return Err(ValidationError::new("duration cannot zero negative"));
            }
        }
        Ok(())
    }

    fn do_set_durations(&mut self, durations: Vec<i32>) {
        self.secs = durations;
    }

    fn validate_target(&self, target: Option<i32>) -> Result<(), ValidationError> {
        if let Some(target) = target {
            if target < 0 {
                return Err(ValidationError::new("target cannot be negative"));
            }
            if target == 0 {
                return Err(ValidationError::new("target cannot be zero"));
            }
        }
        Ok(())
    }

    fn do_set_target(&mut self, target: Option<i32>) {
        self.target_secs = target;
    }
}
