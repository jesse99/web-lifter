use super::*;

pub enum ProgramOp {
    Add(Workout),
}

/// Set of [`Workout`]`s to perform.
pub struct Program {
    pub name: String,
    workouts: Vec<Workout>,
}

impl Program {
    pub fn new(name: String) -> Program {
        Program {
            name,
            workouts: Vec::new(),
        }
    }

    pub fn validate(&mut self, op: &ProgramOp) -> String {
        let mut err = String::new();
        match op {
            ProgramOp::Add(workout) => {
                if self
                    .workouts
                    .iter()
                    .find(|&w| w.name == workout.name)
                    .is_some()
                {
                    err += "The workout name must be unique. ";
                }
            }
        }
        err
    }

    pub fn apply(&mut self, op: ProgramOp) {
        assert_eq!(self.validate(&op), "");
        match op {
            ProgramOp::Add(workout) => {
                self.workouts.push(workout);
            }
        }
    }

    pub fn workouts(&self) -> impl Iterator<Item = &Workout> + '_ {
        self.workouts.iter()
    }

    pub fn find(&self, workout: &str) -> Option<&Workout> {
        self.workouts.iter().find(|w| w.name == workout)
    }
}
