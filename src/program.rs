use std::collections::HashSet;

use crate::{
    pages::ValidationError,
    workout::{Schedule, Workout},
};
use chrono::{DateTime, Datelike, Duration, Local, Weekday};
use serde::{Deserialize, Serialize};

/// Optional block periodization: blocks are scheduled for a number of weeks and then the
/// next block starts up.
#[derive(Debug, Serialize, Deserialize)]
pub struct Block {
    pub name: String,          // e.g. "Heavy" or "Light"
    pub workouts: Vec<String>, // e.g. "Heavy Bench" and "Heavy OHP", can be empty if the user doesn't want to do any workouts for the block
    pub num_weeks: i32,
}

impl Block {
    pub fn new(name: String, workouts: Vec<String>, num_weeks: i32) -> Block {
        Block {
            name,
            workouts,
            num_weeks,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BlockSpan {
    workouts: Vec<String>,
    begin: DateTime<Local>, // week start, i.e. Monday
    end: DateTime<Local>,
}

/// Used by [`Workout`] to create the next scheduled label in the program page.
#[derive(Debug, Clone)]
pub struct BlockSchedule {
    pub spans: Vec<BlockSpan>, // active workout (will typically start in the past), next workout, next next, ends with next scheduled active workout
}

impl BlockSchedule {
    /// Returns true if the workout may be executed in the current block. Note that this
    /// does not mean that the next scheduled date for the workout will still be in the
    /// active block.
    pub fn is_active(&self, workout: &str) -> bool {
        self.spans
            .first()
            .map(|s| s.workouts.iter().any(|w| w == workout))
            .unwrap_or(false)
    }

    /// Returns true if the date is within the span of the active block.
    pub fn in_active(&self, date: DateTime<Local>) -> bool {
        self.spans
            .first()
            .map(|s| s.begin <= date && date < s.end)
            .unwrap_or(false)
    }

    /// Returns the next Monday at which the workout will become active. Note that if the
    /// workout is currently active this will be a future date.
    pub fn next_block_start(&self, workout: &str) -> Option<DateTime<Local>> {
        for span in self.spans.iter().skip(1) {
            if span.workouts.iter().any(|w| w == workout) {
                assert!(span.begin.weekday() == Weekday::Mon);
                return Some(span.begin);
            }
        }
        None
    }
}

/// Set of [`Workout`]`s to perform.
#[derive(Debug, Serialize, Deserialize)]
pub struct Program {
    pub name: String,
    blocks: Vec<Block>,
    blocks_start: Option<DateTime<Local>>, // TODO user can set current week to adjust this
    workouts: Vec<Workout>,
}

impl Program {
    #[allow(dead_code)]
    pub fn new(name: String) -> Program {
        Program {
            name,
            blocks: Vec::new(),
            blocks_start: None,
            workouts: Vec::new(),
        }
    }

    // TODO: workouts cannot be in multiple blocks
    pub fn with_blocks(
        name: String,
        blocks: Vec<Block>,
        now: DateTime<Local>,
        week: i32,
    ) -> Program {
        let mut program = Program {
            name,
            blocks: blocks,
            blocks_start: None,
            workouts: Vec::new(),
        };
        program.set_week(now, week);
        program
    }

    pub fn set_week(&mut self, now: DateTime<Local>, week: i32) {
        assert!(week > 0);

        // Get the start of the current week.
        let delta = now.weekday().num_days_from_monday() as i64;
        let week_start = now - Duration::days(delta);

        // Backup by the week number.
        let delta = 7 * (week - 1) as i64;
        self.blocks_start = Some(week_start - Duration::days(delta));
    }

    pub fn try_set_workouts(
        &mut self,
        workouts: Vec<&str>,
        disabled: Vec<bool>,
    ) -> Result<(), ValidationError> {
        self.validate_set_workouts(&workouts)?;
        self.do_set_workouts(workouts, disabled);
        Ok(())
    }

    pub fn try_add_workout(&mut self, name: &str) -> Result<(), ValidationError> {
        self.validate_add_workout(name)?;

        let workout = Workout::new(name.to_string(), Schedule::AnyDay);
        self.do_add_workout(workout);
        Ok(())
    }

    pub fn add_workout(&mut self, workout: Workout) {
        assert!(self.validate_add_workout(&workout.name).is_ok());
        self.do_add_workout(workout);
    }

    pub fn try_change_workout_name(
        &mut self,
        old_name: &str,
        new_name: &str,
    ) -> Result<(), ValidationError> {
        self.validate_change_workout_name(old_name, new_name)?;
        self.do_change_workout_name(old_name, new_name);
        Ok(())
    }

    pub fn fixup(&mut self) {
        // fn set_weight(program: &mut Program, workout: &str, exercise: &str, weight: f32) {
        //     use crate::exercise::ExerciseName;

        //     if let Some(workout) = program.find_mut(workout) {
        //         if let Some(exercise) = workout.find_mut(&ExerciseName(exercise.to_owned())) {
        //             exercise.set_weight(Some(weight));
        //         } else {
        //             panic!("didn't find exercise {exercise}");
        //         }
        //     } else {
        //         panic!("didn't find workout {workout}");
        //     }
        // }

        // fn set_weightset(program: &mut Program, workout: &str, exercise: &str, name: &str) {
        //     use crate::exercise::ExerciseName;

        //     if let Some(workout) = program.find_mut(workout) {
        //         if let Some(exercise) = workout.find_mut(&ExerciseName(exercise.to_owned())) {
        //             exercise.set_weightset(Some(name.to_owned()));
        //         } else {
        //             panic!("didn't find exercise {exercise}");
        //         }
        //     } else {
        //         panic!("didn't find workout {workout}");
        //     }
        // }

        // fn set_var_sets_target(program: &mut Program, workout: &str, exercise: &str, target: i32) {
        //     use crate::exercise::ExerciseName;

        //     if let Some(workout) = program.find_mut(workout) {
        //         if let Some(exercise) = workout.find_mut(&ExerciseName(exercise.to_owned())) {
        //             let (_, e) = exercise.expect_var_sets_mut();
        //             e.set_target(target);
        //         } else {
        //             panic!("didn't find exercise {exercise}");
        //         }
        //     } else {
        //         panic!("didn't find workout {workout}");
        //     }
        // }

        // self.set_week(Local::now(), 2);
        // set_weightset(self, "Light", "Stack Complex 1", "Gym Dumbbells");
        // set_weightset(self, "Light", "Stack Complex 2", "Gym Dumbbells");

        // set_weight(self, "Heavy Bench", "Heavy RDL", 235.0);
        // set_weight(self, "Heavy Bench", "Cable Abduction", 17.5);
        // set_weight(self, "Heavy OHP", "Heavy OHP", 80.0);

        // set_weight(self, "Medium Bench", "Cable Abduction", 17.5);
        // set_weight(self, "Medium Bench", "Medium RDL", 215.0);
        // set_weight(self, "Medium OHP", "OHP", 80.0);

        // set_var_sets_target(self, "Medium OHP", "Medium Chin-ups", 16);
    }

    pub fn blocks(&self) -> impl Iterator<Item = &Block> + '_ {
        self.blocks.iter()
    }

    pub fn current_block(&self) -> Option<(usize, &Block)> {
        if let Some(blocks_start) = self.blocks_start {
            let (i, _) = find_active(blocks_start, &self.blocks, Local::now());
            Some((i + 1, &self.blocks[i]))
        } else {
            None
        }
    }

    pub fn workouts(&self) -> impl Iterator<Item = &Workout> + '_ {
        self.workouts.iter()
    }

    pub fn find(&self, workout: &str) -> Option<&Workout> {
        self.workouts.iter().find(|w| w.name == workout)
    }

    pub fn find_mut(&mut self, workout: &str) -> Option<&mut Workout> {
        self.workouts.iter_mut().find(|w| w.name == workout)
    }

    pub fn block_schedule(&self) -> BlockSchedule {
        self.block_schedule_from(Local::now())
    }

    fn block_schedule_from(&self, now: DateTime<Local>) -> BlockSchedule {
        if self.blocks_start.is_none() || self.blocks.is_empty() {
            return BlockSchedule { spans: vec![] };
        }

        // Add the active block and all the successor blocks.
        let (start, mut block_start) = find_active(self.blocks_start.unwrap(), &self.blocks, now);
        let mut spans = Vec::new();
        for i in start..self.blocks.len() {
            let block_end = block_start + Duration::weeks(self.blocks[i].num_weeks as i64);
            spans.push(BlockSpan {
                workouts: self.blocks[i].workouts.clone(),
                begin: block_start,
                end: block_end,
            });
            block_start = block_end;
        }

        // Add the predecessor blocks and the active block (again).
        for i in 0..=start {
            let block_end = block_start + Duration::weeks(self.blocks[i].num_weeks as i64);
            spans.push(BlockSpan {
                workouts: self.blocks[i].workouts.clone(),
                begin: block_start,
                end: block_end,
            });
            block_start = block_end;
        }

        BlockSchedule { spans }
    }

    fn validate_add_workout(&self, name: &str) -> Result<(), ValidationError> {
        if self.workouts.iter().find(|&w| w.name == name).is_some() {
            // Other checks would be done when creating workouts.
            return Err(ValidationError::new("The workout name must be unique."));
        }
        Ok(())
    }

    fn do_add_workout(&mut self, workout: Workout) {
        self.workouts.push(workout);
    }

    fn validate_change_workout_name(
        &self,
        old_name: &str,
        new_name: &str,
    ) -> Result<(), ValidationError> {
        if self.find(old_name).is_none() {
            return Err(ValidationError::new("Didn't find old workout."));
        }

        if new_name.trim().is_empty() {
            return Err(ValidationError::new("The workout name cannot be empty."));
        } else if self.workouts.iter().find(|w| w.name == new_name).is_some() {
            return Err(ValidationError::new("The workout name must be unique."));
        }

        Ok(())
    }

    fn do_change_workout_name(&mut self, old_name: &str, new_name: &str) {
        let workout = self.find_mut(old_name).unwrap();
        workout.name = new_name.to_string();

        for block in self.blocks.iter_mut() {
            if let Some(i) = block.workouts.iter().position(|w| w == old_name) {
                block.workouts[i] = new_name.to_string();
            }
        }
    }

    fn validate_set_workouts(&self, workouts: &Vec<&str>) -> Result<(), ValidationError> {
        let mut names = HashSet::new();
        for name in workouts {
            if name.trim().is_empty() {
                return Err(ValidationError::new("The workout name cannot be empty."));
            } else {
                let added = names.insert(name.to_owned());
                if !added {
                    return Err(ValidationError::new("'{name}' appears more than once."));
                }
            }
        }
        Ok(())
    }

    fn do_set_workouts(&mut self, workouts: Vec<&str>, disabled: Vec<bool>) {
        let mut new_workouts = Vec::with_capacity(workouts.len());
        for (i, &name) in workouts.iter().enumerate() {
            let mut workout =
                if let Some(index) = self.workouts.iter().position(|w| w.name == *name) {
                    self.workouts.remove(index)
                } else {
                    Workout::new(name.to_string(), Schedule::AnyDay)
                };
            if i < disabled.len() {
                workout.enabled = !disabled[i];
            }
            new_workouts.push(workout);
        }
        self.workouts = new_workouts;
    }
}

fn find_active(
    blocks_start: DateTime<Local>,
    blocks: &Vec<Block>,
    now: DateTime<Local>,
) -> (usize, DateTime<Local>) {
    let mut block_start = blocks_start; // TODO should we bump this forward in fixup?
    loop {
        // loop because blocks_start might be way in the past
        assert!(blocks_start.weekday() == Weekday::Mon);
        for (i, block) in blocks.iter().enumerate() {
            assert!(block.num_weeks > 0);
            let block_end = block_start + Duration::weeks(block.num_weeks as i64);
            if block_start <= now && now < block_end {
                return (i, block_start);
            }
            block_start = block_end;
        }
    }
}
