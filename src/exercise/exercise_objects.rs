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

impl IDurations for Durations {
    fn expected(&self) -> &Vec<i32> {
        &self.secs
    }
}
// =======================================================================================

/// Used for stuff like 3x60s planks. Target is used to signal the user to increase
/// difficulty (typically by switching to a harder variant of the exercise or adding
/// weight).
pub struct FixedReps {
    reps: Vec<i32>,
}
register_type!(FixedReps);

impl FixedReps {
    // TODO: do we want a validator here?
    pub fn new(reps: Vec<i32>) -> FixedReps {
        FixedReps { reps }
    }
}

impl IFixedReps for FixedReps {
    fn expected(&self) -> &Vec<i32> {
        &self.reps
    }
}
// =======================================================================================

pub struct FormalName {
    name: String, // the actual name, used for stuff like help, e.g. "Low-bar Squat"
}
register_type!(FormalName);

impl FormalName {
    // TODO: do we want a validator here?
    pub fn new(formal_name: String) -> FormalName {
        FormalName { name: formal_name }
    }
}

impl IFormalName for FormalName {
    fn formal_name(&self) -> &str {
        &self.name
    }
}
// =======================================================================================

pub struct LastRest {
    secs: i32,
}
register_type!(LastRest);

impl LastRest {
    pub fn new(secs: i32) -> LastRest {
        LastRest { secs }
    }
}

impl ILastRest for LastRest {
    fn rest(&self) -> i32 {
        self.secs
    }
}
// =======================================================================================

pub struct Rest {
    secs: i32,
}
register_type!(Rest);

impl Rest {
    pub fn new(secs: i32) -> Rest {
        Rest { secs }
    }
}

impl IRest for Rest {
    fn rest(&self) -> i32 {
        self.secs
    }
}
// =======================================================================================

pub struct TargetSecs {
    secs: i32,
}
register_type!(TargetSecs);

impl TargetSecs {
    pub fn new(secs: i32) -> TargetSecs {
        TargetSecs { secs }
    }
}

impl ITargetSecs for TargetSecs {
    fn target(&self) -> i32 {
        self.secs
    }
}
// =======================================================================================
