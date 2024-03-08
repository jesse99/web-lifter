use gear_objects::*;
use paste::paste;

/// Amount of time to rest after each set. The amount of time to rest for the last set
/// can be over-ridden using ILastSet.
pub trait IRest {
    fn rest(&self) -> i32;
}
register_type!(IRest);

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
