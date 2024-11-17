use crate::days::Days;
use crate::validation_err;
use crate::{
    errors::Error,
    workout::{Schedule, Workout},
};
use chrono::{DateTime, Datelike, Duration, Local, Weekday};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Optional block periodization: blocks are scheduled for a number of weeks and then the
/// next block starts up.
#[derive(Clone, Debug, Serialize, Deserialize)]
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
    ) -> Result<(), Error> {
        self.validate_set_workouts(&workouts)?;
        self.do_set_workouts(workouts, disabled);
        Ok(())
    }

    pub fn try_add_workout(&mut self, name: &str) -> Result<(), Error> {
        self.validate_add_workout(name)?;

        let workout = Workout::new(name.to_string(), Schedule::AnyDay);
        self.do_add_workout(workout);
        Ok(())
    }

    pub fn add_workout(&mut self, workout: Workout) {
        assert!(self.validate_add_workout(&workout.name).is_ok());
        self.do_add_workout(workout);
    }

    pub fn try_change_workout_name(&mut self, old_name: &str, new_name: &str) -> Result<(), Error> {
        self.validate_change_workout_name(old_name, new_name)?;
        self.do_change_workout_name(old_name, new_name);
        Ok(())
    }

    pub fn try_set_blocks(&mut self, blocks: Vec<String>) -> Result<(), Error> {
        self.validate_set_blocks(&blocks)?;
        self.do_set_blocks(blocks);
        Ok(())
    }

    pub fn try_set_block(
        &mut self,
        old_name: &str,
        new_name: &str,
        num_weeks: i32,
        workouts: Vec<String>,
    ) -> Result<(), Error> {
        self.validate_set_block(old_name, new_name, num_weeks, &workouts)?;
        self.do_set_block(old_name, new_name, num_weeks, workouts);
        Ok(())
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

    pub fn date_to_block(&self, date: DateTime<Local>) -> Option<&Block> {
        if let Some(blocks_start) = self.blocks_start {
            let (i, _) = find_active(blocks_start, &self.blocks, date);
            Some(&self.blocks[i])
        } else {
            None
        }
    }

    pub fn in_block(&self, workout: &Workout) -> bool {
        for block in &self.blocks {
            if block.workouts.contains(&workout.name) {
                return true;
            }
        }
        false
    }

    pub fn workouts(&self) -> impl Iterator<Item = &Workout> + '_ {
        self.workouts.iter()
    }

    /// Return all workouts that should be performed on the specified date.
    pub fn find_workouts(&self, date: DateTime<Local>) -> Vec<&Workout> {
        fn valid(workout: &Workout, block: Option<&Block>) -> bool {
            match block {
                Some(b) => b.workouts.contains(&workout.name),
                None => true,
            }
        }

        let mut workouts = Vec::new();
        let today = Days::new(Local::now());
        let then = Days::new(date);
        let block = self.date_to_block(date);
        for workout in self.workouts.iter() {
            if workout.enabled && (!self.in_block(workout) || valid(workout, block)) {
                match &workout.schedule {
                    Schedule::AnyDay if then == today => {
                        workouts.push(workout); // any day workouts go at the end
                    }
                    Schedule::AnyDay => {} // too spammy to list any day workouts for all days
                    Schedule::Every(1) if then == today => {
                        workouts.push(workout); // like any day
                    }
                    Schedule::Every(1) => {}
                    Schedule::Every(n) => match workout.days_since_last_completed() {
                        Some(days) if days.value % n == 0 => {
                            workouts.insert(0, workout);
                        }
                        Some(_) => {}
                        None => workouts.insert(0, workout),
                    },
                    Schedule::Days(days) => {
                        if days.contains(&date.weekday()) {
                            workouts.insert(0, workout)
                        }
                    }
                }
            }
        }
        workouts
    }

    pub fn find(&self, workout: &str) -> Option<&Workout> {
        self.workouts.iter().find(|w| w.name == workout)
    }

    pub fn find_mut(&mut self, workout: &str) -> Option<&mut Workout> {
        self.workouts.iter_mut().find(|w| w.name == workout)
    }

    fn validate_add_workout(&self, name: &str) -> Result<(), Error> {
        if self.workouts.iter().find(|&w| w.name == name).is_some() {
            // Other checks would be done when creating workouts.
            return validation_err!("The workout name must be unique.");
        }
        Ok(())
    }

    fn do_add_workout(&mut self, workout: Workout) {
        self.workouts.push(workout);
    }

    fn validate_change_workout_name(&self, old_name: &str, new_name: &str) -> Result<(), Error> {
        if self.find(old_name).is_none() {
            return validation_err!("Didn't find old workout.");
        }

        if new_name.trim().is_empty() {
            return validation_err!("The workout name cannot be empty.");
        } else if self.workouts.iter().find(|w| w.name == new_name).is_some() {
            return validation_err!("The workout name must be unique.");
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

    fn validate_set_block(
        &self,
        old_name: &str,
        new_name: &str,
        num_weeks: i32,
        workouts: &Vec<String>,
    ) -> Result<(), Error> {
        if self.blocks.iter().find(|b| b.name == old_name).is_some() {
            if new_name.trim().is_empty() {
                return validation_err!("The block name cannot be empty.");
            } else if old_name != new_name
                && self.blocks.iter().find(|b| b.name == new_name).is_some()
            {
                return validation_err!("The block name must be unique.");
            }
            if num_weeks < 1 {
                return validation_err!("Number of weeks must be greater than zero.",);
            }
            let mut names = HashSet::new();
            for name in workouts {
                let added = names.insert(name.clone());
                if !added {
                    return validation_err!("'{name}' was listed more than once.");
                }

                if self.workouts.iter().find(|w| w.name == *name).is_none() {
                    return validation_err!("'{name}' isn't a workout.");
                }
            }
        } else {
            return validation_err!("Didn't find old block.");
        }

        Ok(())
    }

    fn do_set_block(
        &mut self,
        old_name: &str,
        new_name: &str,
        num_weeks: i32,
        workouts: Vec<String>,
    ) {
        let block = self.blocks.iter_mut().find(|b| b.name == old_name).unwrap();
        if block.name != new_name {
            block.name = new_name.to_string();
        }
        block.num_weeks = num_weeks;
        block.workouts = workouts;
    }

    fn validate_set_blocks(&self, blocks: &Vec<String>) -> Result<(), Error> {
        let mut names = HashSet::new();
        for name in blocks.iter() {
            if name.trim().is_empty() {
                return validation_err!("Block names cannot be empty.");
            }

            let added = names.insert(name.clone());
            if !added {
                return validation_err!("'{name}' appears more than once.");
            }
        }
        Ok(())
    }

    fn do_set_blocks(&mut self, blocks: Vec<String>) {
        let mut old_blocks = HashMap::new();
        while !self.blocks.is_empty() {
            let block = self.blocks.pop().unwrap();
            old_blocks.insert(block.name.clone(), block);
        }

        // Note that this will implicitly delete blocks that are no longer named.
        let mut new_blocks = Vec::new();
        for name in blocks.iter() {
            let block = if let Some(block) = old_blocks.remove(name) {
                block
            } else {
                Block::new(name.clone(), vec![], 1)
            };
            new_blocks.push(block);
        }

        self.blocks = new_blocks;
    }

    fn validate_set_workouts(&self, workouts: &Vec<&str>) -> Result<(), Error> {
        let mut names = HashSet::new();
        for name in workouts {
            if name.trim().is_empty() {
                return validation_err!("The workout name cannot be empty.");
            } else {
                let added = names.insert(name.to_owned());
                if !added {
                    return validation_err!("'{name}' appears more than once.");
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
    let mut block_start = blocks_start; // TODO should we bump this forward in somewhere?
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
