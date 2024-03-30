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

    pub fn fixup(&mut self) {
        // if let Some(workout) = self.find_mut("Heavy OHP") {
        //     if let Some(exercise) = workout.find_mut(&ExerciseName("Chin-ups".to_owned())) {
        //         let (_, e) = exercise.expect_var_sets_mut();
        //         e.set_target(16);
        //     } else {
        //         panic!("didn't find Chin-ups");
        //     }
        //     if let Some(exercise) = workout.find_mut(&ExerciseName("Face Pulls".to_owned())) {
        //         exercise.set_weight(Some(37.5));
        //     } else {
        //         panic!("didn't find Face Pulls");
        //     }
        // } else {
        //     panic!("didn't find Heavy OHP workout");
        // }
        // if let Some(workout) = self.find_mut("Medium OHP") {
        //     if let Some(exercise) = workout.find_mut(&ExerciseName("Medium Chin-ups".to_owned())) {
        //         let (_, e) = exercise.expect_var_sets_mut();
        //         e.set_target(16);
        //     } else {
        //         panic!("didn't find Medium Chin-ups");
        //     }
        //     if let Some(exercise) = workout.find_mut(&ExerciseName("Face Pulls".to_owned())) {
        //         exercise.set_weight(Some(37.5));
        //     } else {
        //         panic!("didn't find Face Pulls");
        //     }
        // } else {
        //     panic!("didn't find Medium OHP workout");
        // }
        // if let Some(workout) = self.find_mut("Light") {
        //     if let Some(exercise) = workout.find_mut(&ExerciseName("Face Pulls".to_owned())) {
        //         exercise.set_weight(Some(37.5));
        //     } else {
        //         panic!("didn't find Face Pulls");
        //     }
        // } else {
        //     panic!("didn't find Light workout");
        // }
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

    pub fn block_schedule(&self) -> BlockSchedule {
        self.block_schedule_from(Local::now())
    }

    fn block_schedule_from(&self, now: DateTime<Local>) -> BlockSchedule {
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
}
