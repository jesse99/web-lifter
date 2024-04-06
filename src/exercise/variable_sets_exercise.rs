use crate::{exercise::SetIndex, pages::ValidationError};
use serde::{Deserialize, Serialize};

/// Used for stuff like 20 pull-ups spread across as many sets as necessary.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VariableSetsExercise {
    target: i32,
    previous: Vec<i32>,
}

impl VariableSetsExercise {
    // TODO: do we want a validator here?
    pub fn new(target: i32) -> VariableSetsExercise {
        VariableSetsExercise {
            target,
            previous: Vec::new(),
        }
    }

    pub fn target(&self) -> i32 {
        self.target
    }

    // pub fn set_target(&mut self, target: i32) {
    //     self.target = target;
    // }

    pub fn set_previous(&mut self, previous: Vec<i32>) {
        if !previous.is_empty() {
            self.target = previous.iter().sum();
        }
        self.previous = previous;
    }

    pub fn get_previous(&self) -> &Vec<i32> {
        &self.previous
    }

    pub fn previous(&self, index: SetIndex) -> i32 {
        match index {
            SetIndex::Warmup(_) => panic!("expected workset"),
            SetIndex::Workset(i) => self.previous.get(i).copied().unwrap_or(0),
        }
    }

    pub fn try_set_target(&mut self, target: i32) -> Result<(), ValidationError> {
        self.validate_target(target)?;
        self.do_set_target(target);
        Ok(())
    }

    // pub fn set_target(&mut self, target: i32) {
    //     assert!(self.validate_target(target).is_ok());
    //     self.do_set_target(target);
    // }

    fn validate_target(&self, target: i32) -> Result<(), ValidationError> {
        if target < 0 {
            return Err(ValidationError::new("target cannot be negative"));
        }
        if target == 0 {
            return Err(ValidationError::new("target cannot be zero"));
        }
        Ok(())
    }

    fn do_set_target(&mut self, target: i32) {
        self.target = target;
        self.previous = Vec::new();
    }
}
