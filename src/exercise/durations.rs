use crate::*;
use gear_objects::*;
use paste::paste;

/// Used for stuff like 3x60s planks.
pub struct Durations {
    secs: Vec<i32>,
}
register_type!(Durations);

impl Durations {
    // TODO: do we want a validator here?
    pub fn new(secs: Vec<i32>) -> Durations {
        Durations { secs }
    }
}

impl ISummary for Durations {
    fn summary(&self) -> String {
        format!("{}x{}s", self.secs.len(), self.secs[0]) // TODO: do better here
    }
}
