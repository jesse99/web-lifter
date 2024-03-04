use chrono::{DateTime, Datelike, DurationRound, TimeDelta, Utc};
use std::fmt::{self, Formatter};
use std::hash::Hash;
use std::ops::{Add, Sub};

/// Represents number of days since year 1 day 1.
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Days {
    pub value: i32,
}

impl Days {
    // /// Rounds the current date to the nearest day.
    // pub fn now() -> Days {
    //     Days {
    //         value: Utc::now()
    //             .duration_round(TimeDelta::days(1))
    //             .unwrap()
    //             .num_days_from_ce(),
    //     }
    // }

    /// Rounds the date to the nearest day.
    pub fn new(date: DateTime<Utc>) -> Days {
        Days {
            value: date
                .duration_round(TimeDelta::days(1))
                .unwrap()
                .num_days_from_ce(),
        }
    }
}

impl Add<i32> for Days {
    type Output = Days;

    fn add(self, rhs: i32) -> Days {
        Days {
            value: self.value + rhs,
        }
    }
}

impl Sub for Days {
    type Output = i32;

    fn sub(self, rhs: Self) -> i32 {
        self.value - rhs.value
    }
}

impl fmt::Display for Days {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} days", self.value)
    }
}

impl fmt::Debug for Days {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} days", self.value)
    }
}
