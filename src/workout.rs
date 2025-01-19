use crate::validation_err;
use crate::{
    days::Days,
    errors::Error,
    exercise::{
        BuildExercise, Exercise, ExerciseName, FixedReps, FormalName, VariableReps,
        VariableRepsExercise,
    },
};
use chrono::{DateTime, Local, Weekday};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Schedule {
    /// Workout can be done whenever.
    AnyDay,

    /// Workout is scheduled for every N days, e.g. every 3rd day.
    Every(i32),

    // TODO: don't allow this to be empty
    /// Workout is scheduled for specified list of days, e.g. Mon/Wed/Fri.
    Days(Vec<Weekday>),
}

/// Set of [`Exercise`]s to perform all together. These are typically all performed
/// together on one day, e.g. an upper body workout might be performed on Monday and
/// Friday. Workouts are part of a [`Program`].
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Workout {
    pub name: String,
    pub schedule: Schedule,
    pub enabled: bool,
    exercises: Vec<Exercise>,
    completed: HashMap<ExerciseName, DateTime<Local>>, // when the user last did an exercise, for this workout
}

impl Workout {
    pub fn new(name: String, schedule: Schedule) -> Workout {
        Workout {
            name,
            schedule,
            enabled: true,
            exercises: Vec::new(),
            completed: HashMap::new(),
        }
    }

    pub fn fixup(&mut self) {
        for e in &mut self.exercises {
            e.fixup();
        }
    }

    pub fn try_change_exercise_name(
        &mut self,
        old_name: &ExerciseName,
        new_name: &str,
    ) -> Result<(), Error> {
        self.validate_change_exercise_name(old_name, new_name)?;
        self.do_change_exercise_name(old_name, new_name);
        Ok(())
    }

    pub fn try_set_exercises(
        &mut self,
        exercises: Vec<&str>,
        disabled: Vec<bool>,
    ) -> Result<(), Error> {
        self.validate_set_exercises(&exercises)?;
        self.do_set_exercises(exercises, disabled);
        Ok(())
    }

    // pub fn set_exercises(&mut self, exercises: Vec<&str>) {
    //     assert!(self.validate_set_exercises(&exercises).is_ok());
    //     self.do_set_exercises(exercises, Vec::new());
    // }

    pub fn try_add_exercise(&mut self, exercise: Exercise) -> Result<(), Error> {
        self.validate_new_exercise_name(&exercise.name())?;
        self.do_add_exercise(exercise);
        Ok(())
    }

    pub fn add_exercise(&mut self, exercise: Exercise) {
        assert!(self.validate_new_exercise_name(&exercise.name()).is_ok());
        self.do_add_exercise(exercise);
    }

    pub fn try_set_schedule(&mut self, schedule: Schedule) -> Result<(), Error> {
        self.validate_set_schedule(&schedule)?;
        self.do_set_schedule(schedule);
        Ok(())
    }

    pub fn exercises(&self) -> impl Iterator<Item = &Exercise> + '_ {
        self.exercises.iter()
    }

    pub fn find(&self, name: &ExerciseName) -> Option<&Exercise> {
        self.exercises.iter().find(|e| e.name() == name)
    }

    pub fn find_mut(&mut self, name: &ExerciseName) -> Option<&mut Exercise> {
        self.exercises.iter_mut().find(|e| e.name() == name)
    }

    pub fn days_since_last_completed(&self) -> Option<Days> {
        self.completed.values().max().map(|d| Days::new(*d))
    }

    fn validate_change_exercise_name(
        &self,
        old_name: &ExerciseName,
        new_name: &str,
    ) -> Result<(), Error> {
        if self.find(old_name).is_none() {
            return validation_err!("Didn't find old exercise.");
        }

        let new_name = ExerciseName(new_name.to_owned());
        self.validate_new_exercise_name(&new_name)?;

        Ok(())
    }

    fn do_change_exercise_name(&mut self, old_name: &ExerciseName, new_name: &str) {
        let exercise = self.find_mut(old_name).unwrap();
        let new_name = ExerciseName(new_name.to_owned());
        match exercise {
            Exercise::Durations(d, _) => d.name = new_name,
            Exercise::FixedReps(d, _) => d.name = new_name,
            Exercise::VariableReps(d, _) => d.name = new_name,
            Exercise::VariableSets(d, _) => d.name = new_name,
        }
    }

    fn validate_new_exercise_name(&self, name: &ExerciseName) -> Result<(), Error> {
        if name.0.trim().is_empty() {
            return validation_err!("The exercise name cannot be empty.");
        } else if self.exercises.iter().find(|e| e.name() == name).is_some() {
            return validation_err!("The exercise name must be unique.");
        }
        Ok(())
    }

    fn do_add_exercise(&mut self, exercise: Exercise) {
        self.exercises.push(exercise);
    }

    fn validate_set_schedule(&self, schedule: &Schedule) -> Result<(), Error> {
        match schedule {
            Schedule::AnyDay => (),
            Schedule::Every(n) => {
                if *n <= 0 {
                    return validation_err!("N should be greater than zero.");
                }
            }
            Schedule::Days(days) => {
                if days.is_empty() {
                    return validation_err!("At least one day should be scheduled.",);
                }
            }
        }
        Ok(())
    }

    fn do_set_schedule(&mut self, schedule: Schedule) {
        self.schedule = schedule;
    }

    fn validate_set_exercises(&self, exercises: &Vec<&str>) -> Result<(), Error> {
        let mut names = HashSet::new();
        for name in exercises {
            if name.trim().is_empty() {
                return validation_err!("The exercise name cannot be empty.");
            } else {
                let added = names.insert(name.to_owned());
                if !added {
                    return validation_err!("'{name}' appears more than once.");
                }
            }
        }
        Ok(())
    }

    fn do_set_exercises(&mut self, exercises: Vec<&str>, disabled: Vec<bool>) {
        let mut new_exercises = Vec::with_capacity(exercises.len());
        for (i, &name) in exercises.iter().enumerate() {
            let mut exercise =
                if let Some(index) = self.exercises.iter().position(|e| e.name().0 == *name) {
                    self.exercises.remove(index)
                } else {
                    default_exercise(name.to_owned())
                };
            if i < disabled.len() {
                let d = exercise.data_mut();
                d.enabled = !disabled[i];
            }
            new_exercises.push(exercise);
        }
        self.exercises = new_exercises;
    }
}

fn default_exercise(name: String) -> Exercise {
    let warmups = vec![
        FixedReps::new(5, 70),
        FixedReps::new(3, 80),
        FixedReps::new(1, 90),
    ];
    let worksets = vec![VariableReps::new(3, 5, 100); 3];
    let e = VariableRepsExercise::new(warmups, worksets);
    let name = ExerciseName(name);
    let formal_name = FormalName("".to_owned());
    BuildExercise::variable_reps(name, formal_name, e)
        .with_rest_mins(2.0)
        .finalize()
}
