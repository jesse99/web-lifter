use core::sync::atomic::Ordering;
use gear_objects::*;
use paste::paste;

/// The canonical name for an exercise, e.g. "Low-bar Squat". This is used to lookup notes
/// for an exercise but not to identify an exercise within a workout.
pub trait IFormalName {
    fn formal_name(&self) -> &str;
}
register_type!(IFormalName);
