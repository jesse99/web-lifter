use crate::*;
use gear_objects::*;
use paste::paste;

pub struct Set {
    current_set: i32,
    num_sets: i32, // TODO make sure this stays in sync when exercise is edited
}
register_type!(Set);

impl Set {
    pub fn new(num_sets: i32) -> Set {
        Set {
            current_set: 0,
            num_sets,
        }
    }
}

impl ISetDetails for Set {
    fn expected(&self) -> SetDetails {
        SetDetails {
            index: self.current_set,
            count: self.num_sets,
        }
    }
}
