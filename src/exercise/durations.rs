use crate::*;
use core::sync::atomic::Ordering;
use gear_objects::*;
use paste::paste;

pub struct Set {
    pub secs: i32,
    pub rest_secs: i32,
}

impl Set {
    pub fn new(secs: i32, rest_secs: i32) -> Set {
        Set { secs, rest_secs }
    }
}

/// Used for stuff like 3x60s planks. Target is used to signal the user to increase
/// difficulty (typically by switching to a harder variant of the exercise or adding
/// weight).
pub struct Durations {
    formal_name: String, // the actual name, used for stuff like help, e.g. "Low-bar Squat"
    sets: Vec<Set>,
    target_secs: i32,
}
register_type!(Durations);

impl Durations {
    // TODO: do we want a validator here?
    pub fn new(formal_name: String, sets: Vec<Set>, target_secs: i32) -> Durations {
        Durations {
            formal_name,
            sets,
            target_secs,
        }
    }
}

impl IFormalName for Durations {
    fn formal_name(&self) -> &str {
        &self.formal_name
    }
}

impl ISummary for Durations {
    fn summary(&self) -> String {
        format!("{}x{}s", self.sets.len(), self.sets[0].secs) // TODO: do better here
    }
}
