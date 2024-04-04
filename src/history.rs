use crate::exercise::ExerciseName;
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const RECENT_MINS: i64 = 3 * 60;

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

    pub fn fixup(&mut self) {
        // for (_, records) in self.records.iter_mut() {
        //     let indexes: Vec<_> = records
        //         .iter()
        //         .enumerate()
        //         .map(|(i, r)| {
        //             if r.completed.is_none() || r.sets.is_none() {
        //                 Some(i)
        //             } else {
        //                 None
        //             }
        //         })
        //         .collect();
        //     for index in indexes.iter().rev() {
        //         if let Some(index) = index {
        //             records.remove(*index);
        //         }
        //     }
        // }
    }

    /// Adds a record with no sets.
    pub fn start(&mut self, name: &ExerciseName, record: Record) {
        // If we didn't complete the last exercise then nuke it.
        if let Some(records) = self.records.get_mut(name) {
            if let Some(last) = records.iter().last() {
                if last.completed.is_none() {
                    records.pop();
                }
            }
        }
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

    pub fn abort(&mut self, name: &ExerciseName) {
        if let Some(entries) = self.records.get_mut(name) {
            if let Some(last) = entries.last_mut() {
                if last.completed.is_none() {
                    last.sets = None;
                }
            }
        }
    }

    /// Returns records from oldest to newest.
    pub fn records(&self, name: &ExerciseName) -> impl DoubleEndedIterator<Item = &Record> + '_ {
        self.records.get(name).unwrap_or(&self.empty).iter()
    }

    pub fn is_completed(&self, name: &ExerciseName) -> bool {
        self.records(name)
            .last()
            .map(|r| r.completed)
            .flatten()
            .is_some()
    }

    pub fn recently_completed(&self, workout: &str, name: &ExerciseName) -> Option<&Record> {
        if let Some(records) = self.records.get(name) {
            for record in records.iter().rev() {
                if let Some(completed) = record.completed {
                    let delta = Local::now() - completed;
                    if delta.num_minutes() < RECENT_MINS {
                        if record.workout == workout {
                            return Some(record);
                        }
                    } else {
                        break;
                    }
                }
            }
        }
        None
    }

    /// Returns the oldest date at which a recently completed exercise in workout was
    /// started.
    pub fn first_started(&self, workout: &str) -> Option<DateTime<Local>> {
        let mut date = None;
        for (_, records) in self.records.iter() {
            for record in records.iter().rev() {
                if record.workout == workout {
                    if let Some(completed) = record.completed {
                        let delta = Local::now() - completed;
                        if delta.num_minutes() < RECENT_MINS {
                            if let Some(previous) = date {
                                if record.started < previous {
                                    date = Some(record.started);
                                }
                            } else {
                                date = Some(record.started);
                            }
                        }
                        break;
                    }
                }
            }
        }
        date
    }

    /// Returns the newest date at which a recently completed exercise in workout was
    /// finished.
    pub fn last_completed(&self, workout: &str) -> Option<DateTime<Local>> {
        let mut date = None;
        for (_, records) in self.records.iter() {
            for record in records.iter().rev() {
                if record.workout == workout {
                    if let Some(completed) = record.completed {
                        let delta = Local::now() - completed;
                        if delta.num_minutes() < RECENT_MINS {
                            if let Some(previous) = date {
                                if completed > previous {
                                    date = Some(completed);
                                }
                            } else {
                                date = Some(completed);
                            }
                        }
                        break;
                    }
                }
            }
        }
        date
    }
}
