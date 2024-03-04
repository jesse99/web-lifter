use super::*;
use std::collections::HashMap;

pub enum ExercisesOp {
    Add(ExerciseName, Exercise),
}

/// A movement for the user to perform, e.g. a barbell squat. Part of a [`Workout`].
#[derive(Clone, Debug)]
pub struct Exercises {
    pub exercises: HashMap<ExerciseName, Exercise>, // the actual name, used for stuff like help, e.g. "Low-bar Squat"
}

impl Exercises {
    pub fn new() -> Exercises {
        Exercises {
            exercises: HashMap::new(),
        }
    }

    pub fn validate(&mut self, op: &ExercisesOp) -> String {
        let mut err = String::new();
        match op {
            ExercisesOp::Add(name, _) => {
                if name.0.trim().is_empty() {
                    err += "The exercise name cannot be empty. ";
                } else if self.exercises.keys().find(|&n| *n == *name).is_some() {
                    err += "The exercise name must be unique. ";
                }
            }
        }
        err
    }

    pub fn apply(&mut self, op: ExercisesOp) {
        assert_eq!(self.validate(&op), "");
        match op {
            ExercisesOp::Add(name, exercise) => {
                self.exercises.insert(name, exercise);
            }
        }
    }

    pub fn find(&self, name: &ExerciseName) -> Option<&Exercise> {
        self.exercises.get(&name)
    }
}
