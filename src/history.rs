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
    pub comment: Option<String>,
    pub id: u64,
}

/// Records details about the completion of each exercise. Note that this is shared across
/// workouts and programs.
#[derive(Debug, Serialize, Deserialize)]
pub struct History {
    records: HashMap<ExerciseName, Vec<Record>>, // most recent record is last
    empty: Vec<Record>,
    next_id: u64,
}

impl History {
    pub fn new() -> History {
        History {
            records: HashMap::new(),
            empty: Vec::new(),
            next_id: 1,
        }
    }

    pub fn fixup(&mut self) {
        // for (_, records) in self.records.iter_mut() {
        //     for record in records {
        //         if record.id == 0 {
        //             record.id = self.next_id;
        //             self.next_id += 1;
        //         }
        //     }
        // }
    }

    /// Adds a record with no sets.
    pub fn start(
        &mut self,
        program: &str,
        workout: &str,
        exercise: &ExerciseName,
        started: DateTime<Local>,
    ) {
        // If we didn't complete the last exercise then nuke it.
        if let Some(records) = self.records.get_mut(exercise) {
            if let Some(last) = records.iter().last() {
                if last.completed.is_none() {
                    records.pop();
                }
            }
        }

        let record = Record {
            program: program.to_owned(),
            workout: workout.to_owned(),
            started,
            completed: None,
            sets: None,
            comment: None,
            id: self.next_id,
        };
        self.next_id += 1;
        let list = self.records.entry(exercise.clone()).or_insert(Vec::new());
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
    pub fn finish(&mut self, name: &ExerciseName, completed: DateTime<Local>) {
        let entries = self.records.get_mut(name).unwrap();
        let last = entries.last_mut().unwrap();
        last.completed = Some(completed);
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

    pub fn find_record(&self, exercise: &ExerciseName, id: u64) -> Result<&Record, anyhow::Error> {
        if let Some(records) = self.records.get(exercise) {
            if let Some(r) = records.iter().rev().find(|r| r.id == id) {
                Ok(r)
            } else {
                Err(anyhow::Error::msg(
                    "Couldn't find record for exercise {exercise} and id {id}",
                ))
            }
        } else {
            Err(anyhow::Error::msg(
                "Couldn't find records for exercise {exercise}",
            ))
        }
    }

    pub fn find_record_mut(
        &mut self,
        exercise: &ExerciseName,
        id: u64,
    ) -> Result<&mut Record, anyhow::Error> {
        if let Some(records) = self.records.get_mut(exercise) {
            if let Some(r) = records.iter_mut().rev().find(|r| r.id == id) {
                Ok(r)
            } else {
                Err(anyhow::Error::msg(
                    "Couldn't find record for exercise {exercise} and id {id}",
                ))
            }
        } else {
            Err(anyhow::Error::msg(
                "Couldn't find records for exercise {exercise}",
            ))
        }
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
