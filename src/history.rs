use chrono::{DateTime, Utc};
use std::collections::HashMap;

use crate::ExerciseName;

#[derive(Debug)]
pub enum CompletedSets {
    Durations(Vec<(i32, Option<f32>)>),
    Reps(Vec<(i32, Option<f32>)>),
}

/// Result of completing an exercise. Saved into [`History`].
#[derive(Debug)]
pub struct Record {
    pub program: String,
    pub workout: String,
    pub date: DateTime<Utc>,
    pub sets: Option<CompletedSets>,
    pub comment: Option<String>,
}

/// Records details about the completion of each exercise. Note that this is shared across
/// workouts and programs.
pub struct History {
    records: HashMap<ExerciseName, Vec<Record>>, // most recent record is last
    empty: Vec<Record>,
}

impl History {
    pub fn new() -> History {
        History {
            records: HashMap::new(),
            empty: Vec::new(),
        }
    }

    pub fn add(&mut self, name: &ExerciseName, record: Record) {
        let list = self.records.entry(name.clone()).or_insert(Vec::new());
        list.push(record);
    }

    pub fn records(&self, name: &ExerciseName) -> impl DoubleEndedIterator<Item = &Record> + '_ {
        self.records.get(name).unwrap_or(&self.empty).iter()
    }
}
