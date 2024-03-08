use gear_objects::*;
use paste::paste;

/// Optional trait overriding the IRest trait for the last set.
pub trait ILastRest {
    fn rest(&self) -> i32;
}
register_type!(ILastRest);

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
