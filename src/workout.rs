use super::*;
use chrono::{DateTime, Datelike, Duration, Utc};
use std::collections::HashMap;

pub enum WorkoutOp {
    Add(ExerciseName),
}

#[derive(Clone, Debug)]
pub enum Schedule {
    /// Workout can be done whenever.
    AnyDay,

    /// Workout is scheduled for every N days, e.g. every 3rd day.
    Every(i32),

    // TODO: don't allow this to be empty
    /// Workout is scheduled for specified list of days, e.g. Mon/Wed/Fri.
    Days(Vec<Weekday>),
    // /// Workout is scheduled for specified list of weeks, e.g. for week 1 the user
    // /// might do a heavy workout and week 2 a light workout and the workouts would then
    // /// alternate week by week.
    // Weeks(Vec<i32>, Box<Schedule>),
}

/// Reports whether the workout is in progress, overdue, due in 3 days, etc.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Status {
    /// All of the exercises in the workout were completed recently.
    Completed,

    /// Workout is scheduled for N days where 0 is today, 1 is tomorrow, etc.
    Due(i32),

    /// Workout can be done whenever.
    DueAnyTime,

    /// There are no exercises in the workout.
    Empty,

    // /// An exercise has been started recently but not completed.
    // InProgress,

    // /// All the exercises are disabled.
    // NothingEnabled,
    /// Workout was scheduled in the past but was last executed N days before that.
    Overdue(i32),

    /// Some of the exercises in the workout were completed recently.
    PartiallyCompleted,
}

/// Set of [`Exercise`]s to perform all together. These are typically all performed
/// together on one day, e.g. an upper body workout might be performed on Monday and
/// Friday. Workouts are part of a [`Program`].
pub struct Workout {
    pub name: String,
    pub schedule: Schedule,
    exercises: Vec<ExerciseName>, // exercise names must be unique within a workout
    completed: HashMap<ExerciseName, DateTime<Utc>>, // when the user last did an exercise, for this workout
}

impl Workout {
    pub fn new(name: String, schedule: Schedule) -> Workout {
        Workout {
            name,
            schedule,
            exercises: Vec::new(),
            completed: HashMap::new(),
        }
    }

    pub fn validate(&mut self, op: &WorkoutOp) -> String {
        let mut err = String::new();
        match op {
            WorkoutOp::Add(name) => {
                // TODO: check to see if it's in the global exercises?
                // TODO: should names disallow HTML markup symbols?
                if name.0.trim().is_empty() {
                    err += "The exercise name cannot be empty. ";
                } else if self.exercises.iter().find(|&n| *n == *name).is_some() {
                    err += "The exercise name must be unique. ";
                }
            }
        }
        err
    }

    pub fn apply(&mut self, op: WorkoutOp) {
        assert_eq!(self.validate(&op), "");
        match op {
            WorkoutOp::Add(name) => {
                self.exercises.push(name);
            }
        }
    }

    pub fn exercises(&self) -> impl Iterator<Item = &ExerciseName> + '_ {
        self.exercises.iter()
    }

    // pub fn set_completed(&mut self, name: ExerciseName) {
    //     // TODO: check that this is one of our exercises
    //     // We use the Utc timezone instead of Local mostly because users can move across
    //     // timezones. Also may be handy if we, for some reason, start comparing datetime's
    //     // across users.
    //     self.completed.insert(name, Utc::now());
    // }

    pub fn status(&self) -> Status {
        self.status_from(Utc::now())
    }

    fn status_from(&self, now: DateTime<Utc>) -> Status {
        if self.exercises.is_empty() {
            return Status::Empty;
        }

        let today = Days::new(now);
        let last = self.days_since_last_completed();
        if let Some(last) = last {
            if last == today {
                if self.all_completed(today) {
                    return Status::Completed;
                } else {
                    return Status::PartiallyCompleted;
                }
            }
        }

        match &self.schedule {
            Schedule::AnyDay => Status::DueAnyTime,
            Schedule::Every(1) => {
                if last.is_some() {
                    Status::Due(0)
                } else {
                    // Little weird but this way workouts that are scheduled for a different
                    // number of days will be listed as due at staggered times.
                    Status::Due(1)
                }
            }
            Schedule::Every(n) => {
                if let Some(last) = last {
                    let due = last + *n;
                    if due < today && *n > 1 {
                        Status::Overdue(n - 1)
                    } else {
                        Status::Due(due - today)
                    }
                } else {
                    Status::Due(*n)
                }
            }
            Schedule::Days(days) => {
                if let Some(last) = last {
                    let scheduled = Days::new(self.last_scheduled_date(now, days));
                    if last >= scheduled {
                        let next = self.next_scheduled_date(now, days).unwrap();
                        let n = Days::new(next) - today;
                        Status::Due(n)
                    } else if days.contains(&now.weekday()) {
                        Status::Overdue(scheduled - last)
                    } else {
                        let next = self.next_scheduled_date(now, days).unwrap();
                        let n = Days::new(next) - today;
                        Status::Due(n)
                    }
                } else {
                    let due = days
                        .iter()
                        .map(|wd| self.weekday_to_days(now, *wd))
                        .min()
                        .unwrap();
                    Status::Due(due - today)
                }
            }
        }
    }

    fn all_completed(&self, on: Days) -> bool {
        for name in self.exercises.iter() {
            if let Some(last) = self.completed.get(&name) {
                if Days::new(*last) != on {
                    return false;
                }
            } else {
                return false;
            }
        }
        true
    }

    // Return the next day the weekday happens relative to now.
    fn weekday_to_days(&self, now: DateTime<Utc>, wd: Weekday) -> Days {
        let today = now.weekday();
        let delta = (wd as i32) - (today as i32);
        if delta >= 0 {
            Days::new(now) + delta
        } else {
            Days::new(now) + 7 + delta
        }
    }

    fn days_since_last_completed(&self) -> Option<Days> {
        self.completed.values().max().map(|d| Days::new(*d))
    }

    // Return the date the next workout should happen on.
    fn next_scheduled_date(
        &self,
        now: DateTime<Utc>,
        weekdays: &Vec<Weekday>,
    ) -> Option<DateTime<Utc>> {
        if weekdays.len() == 1 && now.weekday() == weekdays[0] {
            return Some(now);
        }

        let n = if weekdays.len() == 1 {
            (weekdays[0] as i64) - (now.weekday() as i64)
        } else {
            let today = now.weekday();
            let wd = weekdays
                .iter()
                .find(|&&wd| (wd as i32) >= (today as i32))
                .unwrap_or(&weekdays[0]);
            (*wd as i64) - (now.weekday() as i64)
        };
        if n >= 0 {
            Some(now + Duration::days(n))
        } else {
            Some(now + Duration::days(7 + n))
        }
    }

    // Return the date the previous workout should have happened on.
    fn last_scheduled_date(&self, now: DateTime<Utc>, weekdays: &Vec<Weekday>) -> DateTime<Utc> {
        let mut candidates: Vec<DateTime<Utc>> = (1..8)
            .map(|n| now - Duration::days(n))
            .filter(|d| weekdays.contains(&d.weekday()))
            .collect();
        candidates.sort_by(|x, y| (now - x).cmp(&(now - y)));
        candidates[0]
    }
}

// Check status()
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty() {
        let workout = Workout::new("test".to_owned(), Schedule::AnyDay);
        assert_eq!(workout.status(), Status::Empty);
        // can't test complete because there are no exercises to complete
    }

    #[test]
    fn any_day() {
        let mut workout = Workout::new("test".to_owned(), Schedule::AnyDay);
        let name = ExerciseName("Squat".to_owned());
        workout.apply(WorkoutOp::Add(name.clone()));

        // not completed
        assert_eq!(workout.status(), Status::DueAnyTime);

        // completed two days ago
        let date = Utc::now() - Duration::days(2);
        workout.completed.insert(name.clone(), date);
        assert_eq!(workout.status(), Status::DueAnyTime);

        // completed yesterday
        let date = Utc::now() - Duration::days(1);
        workout.completed.insert(name.clone(), date);
        assert_eq!(workout.status(), Status::DueAnyTime);

        // completed today
        let date = Utc::now();
        workout.completed.insert(name.clone(), date);
        assert_eq!(workout.status(), Status::Completed);
    }

    #[test]
    fn every_1_days() {
        let mut workout = Workout::new("test".to_owned(), Schedule::Every(1)); // bit silly
        let name = ExerciseName("Squat".to_owned());
        workout.apply(WorkoutOp::Add(name.clone()));

        // not completed
        assert_eq!(workout.status(), Status::Due(1));

        // completed two days ago
        let date = Utc::now() - Duration::days(2);
        workout.completed.insert(name.clone(), date);
        assert_eq!(workout.status(), Status::Due(0));

        // completed yesterday
        let date = Utc::now() - Duration::days(1);
        workout.completed.insert(name.clone(), date);
        assert_eq!(workout.status(), Status::Due(0));

        // completed today
        let date = Utc::now();
        workout.completed.insert(name.clone(), date);
        assert_eq!(workout.status(), Status::Completed);
    }

    #[test]
    fn every_2_days() {
        let mut workout = Workout::new("test".to_owned(), Schedule::Every(2));
        let name = ExerciseName("Squat".to_owned());
        workout.apply(WorkoutOp::Add(name.clone()));

        // not completed
        let now = date_from_day_hour(Weekday::Mon, 20);
        assert_eq!(workout.status_from(now), Status::Due(2));

        // completed three days ago
        let date = now - Duration::days(3);
        workout.completed.insert(name.clone(), date);
        assert_eq!(workout.status_from(now), Status::Overdue(1));

        // completed two days ago
        let date = now - Duration::days(2);
        workout.completed.insert(name.clone(), date);
        assert_eq!(workout.status_from(now), Status::Due(0));

        // completed 1.8 days ago
        let date = now - (Duration::days(1) + Duration::hours(19));
        workout.completed.insert(name.clone(), date);
        assert_eq!(workout.status_from(now), Status::Due(0));

        // completed 1.25 days ago
        let date = now - (Duration::days(1) + Duration::hours(6));
        workout.completed.insert(name.clone(), date);
        assert_eq!(workout.status_from(now), Status::Due(1));

        // completed yesterday
        let date = now - Duration::days(1);
        workout.completed.insert(name.clone(), date);
        assert_eq!(workout.status_from(now), Status::Due(1));

        // completed 0.8 days ago
        let date = now - Duration::hours(19);
        workout.completed.insert(name.clone(), date);
        assert_eq!(workout.status_from(now), Status::Due(1));
    }

    fn date_from_day(wd: Weekday) -> DateTime<Utc> {
        let day = 11 + (wd as i32);
        let text = format!("2024-03-{day}T08:00:00Z");
        DateTime::parse_from_rfc3339(&text).unwrap().into()
    }

    fn date_from_day_hour(wd: Weekday, hour: i32) -> DateTime<Utc> {
        let day = 11 + (wd as i32);
        let text = format!("2024-03-{day}T{hour:02}:00:00Z");
        DateTime::parse_from_rfc3339(&text).unwrap().into()
    }

    #[test]
    fn days_not_completed1() {
        // due Monday
        let days = vec![Weekday::Mon];
        let mut workout = Workout::new("test".to_owned(), Schedule::Days(days));
        let name = ExerciseName("Squat".to_owned());
        workout.apply(WorkoutOp::Add(name.clone()));

        let now = date_from_day(Weekday::Mon);
        assert_eq!(workout.status_from(now), Status::Due(0));

        let now = date_from_day(Weekday::Tue);
        assert_eq!(workout.status_from(now), Status::Due(6));

        let now = date_from_day(Weekday::Sat);
        assert_eq!(workout.status_from(now), Status::Due(2));

        // due Friday
        let days = vec![Weekday::Fri];
        let mut workout = Workout::new("test".to_owned(), Schedule::Days(days));
        let name = ExerciseName("Squat".to_owned());
        workout.apply(WorkoutOp::Add(name.clone()));

        let now = date_from_day(Weekday::Mon);
        assert_eq!(workout.status_from(now), Status::Due(4));

        let now = date_from_day(Weekday::Tue);
        assert_eq!(workout.status_from(now), Status::Due(3));

        let now = date_from_day(Weekday::Fri);
        assert_eq!(workout.status_from(now), Status::Due(0));

        let now = date_from_day(Weekday::Sat);
        assert_eq!(workout.status_from(now), Status::Due(6));
    }

    #[test]
    fn days_not_completed3() {
        let days = vec![Weekday::Mon, Weekday::Wed, Weekday::Fri];
        let mut workout = Workout::new("test".to_owned(), Schedule::Days(days));
        let name = ExerciseName("Squat".to_owned());
        workout.apply(WorkoutOp::Add(name.clone()));

        let now = date_from_day(Weekday::Mon);
        assert_eq!(workout.status_from(now), Status::Due(0));

        let now = date_from_day(Weekday::Tue);
        assert_eq!(workout.status_from(now), Status::Due(1));

        let now = date_from_day(Weekday::Wed);
        assert_eq!(workout.status_from(now), Status::Due(0));

        let now = date_from_day(Weekday::Sat);
        assert_eq!(workout.status_from(now), Status::Due(2));

        let now = date_from_day(Weekday::Sun);
        assert_eq!(workout.status_from(now), Status::Due(1));
    }

    #[test]
    fn days1() {
        let days = vec![Weekday::Mon];
        let mut workout = Workout::new("test".to_owned(), Schedule::Days(days));
        let name = ExerciseName("Squat".to_owned());
        workout.apply(WorkoutOp::Add(name.clone()));

        // completed Monday but two weeks ago
        let date = date_from_day(Weekday::Mon) - Duration::days(14);
        let now = date_from_day(Weekday::Mon);
        workout.completed.insert(name.clone(), date);
        assert_eq!(workout.status_from(now), Status::Overdue(7));

        let now = date_from_day(Weekday::Tue);
        workout.completed.insert(name.clone(), date);
        assert_eq!(workout.status_from(now), Status::Due(6));

        let now = date_from_day(Weekday::Wed);
        workout.completed.insert(name.clone(), date);
        assert_eq!(workout.status_from(now), Status::Due(5));

        // completed last Monday
        let date = date_from_day_hour(Weekday::Mon, 8) - Duration::days(7);
        let now = date_from_day_hour(Weekday::Mon, 9);
        workout.completed.insert(name.clone(), date);
        assert_eq!(workout.status_from(now), Status::Due(0));

        let now = date_from_day_hour(Weekday::Tue, 9);
        workout.completed.insert(name.clone(), date);
        assert_eq!(workout.status_from(now), Status::Due(6));

        // completed Monday
        let date = date_from_day_hour(Weekday::Mon, 8);
        let now = date_from_day_hour(Weekday::Mon, 9);
        workout.completed.insert(name.clone(), date);
        assert_eq!(workout.status_from(now), Status::Completed);

        // completed Tuesday
        let now = date_from_day(Weekday::Tue);
        assert_eq!(workout.status_from(now), Status::Due(6));
    }

    #[test]
    fn days3() {
        let days = vec![Weekday::Mon, Weekday::Wed, Weekday::Fri];
        let mut workout = Workout::new("test".to_owned(), Schedule::Days(days));
        let name = ExerciseName("Squat".to_owned());
        workout.apply(WorkoutOp::Add(name.clone()));

        // completed Monday but two weeks ago
        let date = date_from_day(Weekday::Mon) - Duration::days(14);
        let now = date_from_day(Weekday::Mon);
        workout.completed.insert(name.clone(), date);
        assert_eq!(workout.status_from(now), Status::Overdue(11));

        let now = date_from_day(Weekday::Tue);
        workout.completed.insert(name.clone(), date);
        assert_eq!(workout.status_from(now), Status::Due(1));

        let now = date_from_day(Weekday::Wed);
        workout.completed.insert(name.clone(), date);
        assert_eq!(workout.status_from(now), Status::Overdue(14));

        // completed Monday but a week ago
        let date = date_from_day_hour(Weekday::Mon, 8) - Duration::days(7);
        let now = date_from_day_hour(Weekday::Mon, 9);
        workout.completed.insert(name.clone(), date);
        assert_eq!(workout.status_from(now), Status::Overdue(4));

        let now = date_from_day(Weekday::Tue);
        assert_eq!(workout.status_from(now), Status::Due(1));

        let now = date_from_day(Weekday::Wed);
        assert_eq!(workout.status_from(now), Status::Overdue(7));

        let now = date_from_day(Weekday::Thu);
        assert_eq!(workout.status_from(now), Status::Due(1));

        let now = date_from_day(Weekday::Fri);
        assert_eq!(workout.status_from(now), Status::Overdue(9));

        // completed Monday
        let date = date_from_day_hour(Weekday::Mon, 8);
        let now = date_from_day_hour(Weekday::Mon, 9);
        workout.completed.insert(name.clone(), date);
        assert_eq!(workout.status_from(now), Status::Completed);

        let now = date_from_day(Weekday::Tue);
        assert_eq!(workout.status_from(now), Status::Due(1));

        let now = date_from_day(Weekday::Wed);
        assert_eq!(workout.status_from(now), Status::Due(0));

        let now = date_from_day(Weekday::Thu);
        assert_eq!(workout.status_from(now), Status::Due(1));

        let now = date_from_day(Weekday::Fri);
        assert_eq!(workout.status_from(now), Status::Overdue(2));

        // completed last Fri
        let date = date_from_day_hour(Weekday::Fri, 8) - Duration::days(7);
        let now = date_from_day_hour(Weekday::Mon, 9);
        workout.completed.insert(name.clone(), date);
        assert_eq!(workout.status_from(now), Status::Due(0));

        let now = date_from_day(Weekday::Tue);
        assert_eq!(workout.status_from(now), Status::Due(1));

        let now = date_from_day(Weekday::Wed);
        assert_eq!(workout.status_from(now), Status::Overdue(3));

        let now = date_from_day(Weekday::Thu);
        assert_eq!(workout.status_from(now), Status::Due(1));

        let now = date_from_day(Weekday::Fri);
        assert_eq!(workout.status_from(now), Status::Overdue(5));
    }

    #[test]
    fn multiple() {
        let days = vec![Weekday::Mon, Weekday::Wed, Weekday::Fri];
        let mut workout = Workout::new("test".to_owned(), Schedule::Days(days));
        let name1 = ExerciseName("Squat".to_owned());
        workout.apply(WorkoutOp::Add(name1.clone()));
        let name2 = ExerciseName("Bench".to_owned());
        workout.apply(WorkoutOp::Add(name2.clone()));

        // not completed
        let now = date_from_day(Weekday::Mon);
        assert_eq!(workout.status_from(now), Status::Due(0));

        // complete one
        let date = date_from_day_hour(Weekday::Mon, 8);
        workout.completed.insert(name1.clone(), date);
        assert_eq!(workout.status_from(now), Status::PartiallyCompleted);

        // complete both
        let date = date_from_day_hour(Weekday::Mon, 9);
        workout.completed.insert(name2.clone(), date);
        assert_eq!(workout.status_from(now), Status::Completed);
    }
}
