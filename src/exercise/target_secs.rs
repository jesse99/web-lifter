use gear_objects::*;
use paste::paste;

/// Optional goal seconds. Can be used with a durations exercise: when the user hits the
/// target they will typically switch to a harder version of the exercise or add weight.
pub trait ITargetSecs {
    fn target(&self) -> i32;
}
register_type!(ITargetSecs);

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
