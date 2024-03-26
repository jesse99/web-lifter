use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::ExerciseName;

#[derive(Debug, Serialize, Deserialize)]
pub enum CompletedSets {
    Durations(Vec<(i32, Option<f32>)>),
    Reps(Vec<(i32, Option<f32>)>),
}

/// Result of completing an exercise. Saved into [`History`].
#[derive(Debug, Serialize, Deserialize)]
pub struct Record {
    pub program: String,
    pub workout: String,
    pub started: DateTime<Local>,
    pub completed: Option<DateTime<Local>>,
    pub sets: Option<CompletedSets>,
    pub comment: Option<String>, // TODO this could be set when user edits a record
}

/// Records details about the completion of each exercise. Note that this is shared across
/// workouts and programs.
#[derive(Debug, Serialize, Deserialize)]
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

    /// Adds a record with no sets.
    pub fn start(&mut self, name: &ExerciseName, record: Record) {
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

    /// Appended all the sets.
    pub fn finish(&mut self, name: &ExerciseName) {
        let entries = self.records.get_mut(name).unwrap();
        let last = entries.last_mut().unwrap();
        last.completed = Some(Local::now());
    }

    /// Returns records from oldest to newest.
    pub fn records(&self, name: &ExerciseName) -> impl DoubleEndedIterator<Item = &Record> + '_ {
        self.records.get(name).unwrap_or(&self.empty).iter()
    }
}
