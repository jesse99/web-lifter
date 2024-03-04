use core::sync::atomic::Ordering;
use gear_objects::*;
use paste::paste;

/// Returns a brief description of an exercise, e.g. it will return things like:
/// "4x20s"
/// "3x5 reps @ 135 lbs"
/// "2x3-5 reps, 1-5 reps @ 135 lbs"
pub trait ISummary {
    fn summary(&self) -> String;
}
register_type!(ISummary);
