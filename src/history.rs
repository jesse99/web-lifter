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
    pub comment: Option<String>, // TODO this could be set when user edits a record
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

    /// Used to add a record with no sets.
    pub fn add(&mut self, name: &ExerciseName, record: Record) {
        assert!(record.sets.is_none());
        let list = self.records.entry(name.clone()).or_insert(Vec::new());
        list.push(record);
    }

    /// Append a Durations set onto the last added record.
    pub fn append_duration(&mut self, name: &ExerciseName, duration: i32, weight: Option<f32>) {
        let entries = self.records.get_mut(name).unwrap();
        let last = entries.last_mut().unwrap();
        if last.sets.is_none() {
            last.sets = Some(CompletedSets::Durations(Vec::new()));
        }
        match last.sets {
            Some(CompletedSets::Durations(ref mut sets)) => sets.push((duration, weight)),
            _ => panic!("expected Durations"),
        }
    }

    /// Append a Reps set onto the last added record.
    pub fn append_reps(&mut self, name: &ExerciseName, reps: i32, weight: Option<f32>) {
        let entries = self.records.get_mut(name).unwrap();
        let last = entries.last_mut().unwrap();
        if last.sets.is_none() {
            last.sets = Some(CompletedSets::Reps(Vec::new()));
        }
        match last.sets {
            Some(CompletedSets::Reps(ref mut sets)) => sets.push((reps, weight)),
            _ => panic!("expected Reps"),
        }
    }

    /// Returns revords from oldest to newest.
    pub fn records(&self, name: &ExerciseName) -> impl DoubleEndedIterator<Item = &Record> + '_ {
        self.records.get(name).unwrap_or(&self.empty).iter()
    }
}
