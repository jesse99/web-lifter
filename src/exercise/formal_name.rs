use gear_objects::*;
use paste::paste;

/// The canonical name for an exercise, e.g. "Low-bar Squat". This is used to lookup notes
/// for an exercise but not to identify an exercise within a workout.
pub trait IFormalName {
    fn formal_name(&self) -> &str;
}
register_type!(IFormalName);

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
