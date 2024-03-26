use std::collections::HashMap;

use super::*;
use chrono::{DateTime, Datelike, Duration};
use serde::{Deserialize, Serialize};

pub enum ProgramOp {
    Add(Workout),
    Del(String),
}

/// Optional block periodization: blocks are scheduled for a number of weeks and then the
/// next block starts up.
#[derive(Debug, Serialize, Deserialize)]
pub struct Block {
    pub name: String,
    pub workouts: Vec<String>, // can be empty if the user doesn't want to do any workouts for the block
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

pub struct BlockSpan {
    begin: DateTime<Local>,
    end: DateTime<Local>,
}

/// Used by [`Workout`] to create the next scheduled label in the program page.
pub struct BlockSchedule {
    blocks_start: DateTime<Local>,
    workouts: HashMap<String, BlockSpan>,
    total_weeks: i32,
}

impl BlockSchedule {
    /// Returns the number of days to add to scheduled to account for any block scheduling
    /// that may be present.
    pub fn adjustment(
        &self,
        workout: &str,
        now: DateTime<Local>,
        scheduled: DateTime<Local>,
    ) -> i32 {
        if let Some(span) = self.workouts.get(workout) {
            if span.begin <= now && now < span.end {
                if span.begin <= scheduled && scheduled < span.end {
                    // now and scheduled are in the active block
                    0
                } else {
                    // now is in the current block but next scheduled is not
                    7 * (self.total_weeks - 1)
                }
            } else if scheduled < span.begin {
                // block is coming up
                date_to_days(span.begin) - date_to_days(now)
            } else {
                // block is in the past
                let last_end = self.blocks_start + Duration::weeks(self.total_weeks as i64);
                (date_to_days(last_end) - date_to_days(now))
                    + (date_to_days(span.begin) - date_to_days(self.blocks_start))
            }
        } else {
            // workout isn't part of a block
            0
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
        assert!(week > 0);

        // Get the start of the current week.
        let delta = now.weekday().num_days_from_monday() as i64;
        let week_start = now - Duration::days(delta);

        // Backup by the week number.
        let delta = 7 * (week - 1) as i64;
        let blocks_start = Some(week_start - Duration::days(delta));
        Program {
            name,
            blocks: blocks,
            blocks_start,
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
            ProgramOp::Del(name) => {
                if self.workouts.iter().find(|&w| w.name == *name).is_none() {
                    err += "The workout name doesn't exist. ";
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
            ProgramOp::Del(name) => {
                let index = self.workouts.iter().position(|w| w.name == name).unwrap();
                self.workouts.remove(index);
            }
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

    /// Returns the number of days to the block start (a Monday) or 0 if the workout is in
    /// the current block.
    pub fn block_schedule(&self) -> BlockSchedule {
        self.block_schedule_from(Local::now())
    }

    fn block_schedule_from(&self, now: DateTime<Local>) -> BlockSchedule {
        let mut blocks_start = self.blocks_start.unwrap_or(now);
        let total_weeks = self.blocks.iter().fold(0, |sum, b| sum + b.num_weeks);
        while total_weeks > 0 && (now - blocks_start).num_weeks() >= total_weeks as i64 {
            blocks_start += Duration::weeks(total_weeks as i64);
        }

        let mut workouts = HashMap::new();
        let mut block_start = blocks_start;
        for block in self.blocks.iter() {
            let block_end = block_start + Duration::weeks(block.num_weeks as i64);
            for workout in block.workouts.iter() {
                let span = BlockSpan {
                    begin: block_start,
                    end: block_end,
                };
                workouts.insert(workout.clone(), span);
            }
            block_start = block_end;
        }

        BlockSchedule {
            blocks_start,
            workouts,
            total_weeks,
        }
    }
}

// #[cfg(test)]
// mod tests {
//     // use chrono::{DurationRound, TimeDelta};

//     use super::*;

//     #[test]
//     fn no_blocks() {
//         let now = date_from_day(Weekday::Tue);
//         let program = Program::new("Test".to_owned());
//         assert_eq!(program.days_to_block_start_from(now, "Heavy Bench"), 0);
//         assert_eq!(program.days_to_block_start_from(now, "Medium Bench"), 0);
//         assert_eq!(program.days_to_block_start_from(now, "Elliptical"), 0);

//         let now = now + Duration::weeks(1);
//         assert_eq!(program.days_to_block_start_from(now, "Medium Bench"), 0);
//         assert_eq!(program.days_to_block_start_from(now, "Elliptical"), 0);
//         assert_eq!(program.days_to_block_start_from(now, "Heavy Bench"), 0);

//         let now = now + Duration::weeks(1);
//         assert_eq!(program.days_to_block_start_from(now, "Elliptical"), 0);
//         assert_eq!(program.days_to_block_start_from(now, "Heavy Bench"), 0);
//         assert_eq!(program.days_to_block_start_from(now, "Medium Bench"), 0);
//     }

//     #[test]
//     fn weekly() {
//         let blocks = vec![
//             Block::new("Heavy".to_owned(), vec!["Heavy Bench".to_owned()], 1),
//             Block::new("Medium".to_owned(), vec!["Medium Bench".to_owned()], 1),
//             Block::new("Cardio".to_owned(), vec!["Elliptical".to_owned()], 1),
//         ];
//         let now = date_from_day(Weekday::Tue);
//         let program = Program::with_blocks("Test".to_owned(), blocks, now, 1);
//         assert_eq!(program.days_to_block_start_from(now, "Heavy Bench"), 0);
//         assert_eq!(program.days_to_block_start_from(now, "Medium Bench"), 6);
//         assert_eq!(program.days_to_block_start_from(now, "Elliptical"), 13);

//         let now = now + Duration::weeks(1);
//         assert_eq!(program.days_to_block_start_from(now, "Medium Bench"), 0);
//         assert_eq!(program.days_to_block_start_from(now, "Elliptical"), 6);
//         assert_eq!(program.days_to_block_start_from(now, "Heavy Bench"), 13);

//         let now = now + Duration::weeks(1);
//         assert_eq!(program.days_to_block_start_from(now, "Elliptical"), 0);
//         assert_eq!(program.days_to_block_start_from(now, "Heavy Bench"), 6);
//         assert_eq!(program.days_to_block_start_from(now, "Medium Bench"), 13);
//     }

//     #[test]
//     fn weekly2() {
//         let blocks = vec![
//             Block::new("Heavy".to_owned(), vec!["Heavy Bench".to_owned()], 1),
//             Block::new("Medium".to_owned(), vec!["Medium Bench".to_owned()], 1),
//             Block::new("Cardio".to_owned(), vec!["Elliptical".to_owned()], 1),
//         ];
//         let now = date_from_day(Weekday::Sun);
//         let program = Program::with_blocks("Test".to_owned(), blocks, now, 1);
//         assert_eq!(program.days_to_block_start_from(now, "Heavy Bench"), 0);
//         assert_eq!(program.days_to_block_start_from(now, "Medium Bench"), 1);
//         assert_eq!(program.days_to_block_start_from(now, "Elliptical"), 8);
//     }

//     #[test]
//     fn biweekly() {
//         let blocks = vec![
//             Block::new("Heavy".to_owned(), vec!["Heavy Bench".to_owned()], 2),
//             Block::new("Medium".to_owned(), vec!["Medium Bench".to_owned()], 2),
//             Block::new("Cardio".to_owned(), vec!["Elliptical".to_owned()], 1),
//         ];
//         let now = date_from_day(Weekday::Tue);
//         let program = Program::with_blocks("Test".to_owned(), blocks, now, 1);
//         assert_eq!(program.days_to_block_start_from(now, "Heavy Bench"), 0);
//         assert_eq!(program.days_to_block_start_from(now, "Medium Bench"), 13);
//         assert_eq!(program.days_to_block_start_from(now, "Elliptical"), 27);

//         let now = now + Duration::weeks(1);
//         assert_eq!(program.days_to_block_start_from(now, "Heavy Bench"), 0);
//         assert_eq!(program.days_to_block_start_from(now, "Medium Bench"), 6);
//         assert_eq!(program.days_to_block_start_from(now, "Elliptical"), 20);

//         let now = now + Duration::weeks(1);
//         assert_eq!(program.days_to_block_start_from(now, "Medium Bench"), 0);
//         assert_eq!(program.days_to_block_start_from(now, "Elliptical"), 13);
//         assert_eq!(program.days_to_block_start_from(now, "Heavy Bench"), 20);

//         let now = now + Duration::weeks(1);
//         assert_eq!(program.days_to_block_start_from(now, "Medium Bench"), 0);
//         assert_eq!(program.days_to_block_start_from(now, "Elliptical"), 6);
//         assert_eq!(program.days_to_block_start_from(now, "Heavy Bench"), 13);

//         let now = now + Duration::weeks(1);
//         assert_eq!(program.days_to_block_start_from(now, "Elliptical"), 0);
//         assert_eq!(program.days_to_block_start_from(now, "Heavy Bench"), 6);
//         assert_eq!(program.days_to_block_start_from(now, "Medium Bench"), 20);

//         let now = now + Duration::weeks(1);
//         assert_eq!(program.days_to_block_start_from(now, "Heavy Bench"), 0);
//         assert_eq!(program.days_to_block_start_from(now, "Medium Bench"), 13);
//         assert_eq!(program.days_to_block_start_from(now, "Elliptical"), 27);
//     }

//     fn date_from_day(wd: Weekday) -> DateTime<Local> {
//         let day = 11 + (wd as i32);
//         let text = format!("2024-03-{day}T08:00:00Z");
//         DateTime::parse_from_rfc3339(&text).unwrap().into()
//     }
// }
