use crate::{
    days::Days,
    exercise::{
        BuildExercise, Exercise, ExerciseName, FixedReps, FormalName, VariableReps,
        VariableRepsExercise,
    },
    pages::ValidationError,
    program::BlockSchedule,
};
use chrono::{DateTime, Datelike, Duration, Local, Weekday};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Schedule {
    /// Workout can be done whenever.
    AnyDay,

    /// Workout is scheduled for every N days, e.g. every 3rd day.
    Every(i32),

    // TODO: don't allow this to be empty
    /// Workout is scheduled for specified list of days, e.g. Mon/Wed/Fri.
    Days(Vec<Weekday>),
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
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Workout {
    pub name: String,
    pub schedule: Schedule,
    exercises: Vec<Exercise>,
    completed: HashMap<ExerciseName, DateTime<Local>>, // when the user last did an exercise, for this workout
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

    pub fn try_set_exercises(
        &mut self,
        exercises: Vec<&str>,
        disabled: Vec<bool>,
    ) -> Result<(), ValidationError> {
        self.validate_set_exercises(&exercises)?;
        self.do_set_exercises(exercises, disabled);
        Ok(())
    }

    // pub fn set_exercises(&mut self, exercises: Vec<&str>) {
    //     assert!(self.validate_set_exercises(&exercises).is_ok());
    //     self.do_set_exercises(exercises, Vec::new());
    // }

    // pub fn try_add_exercise(&mut self, exercise: Exercise) -> Result<(), ValidationError> {
    //     self.validate_add_exercise(&exercise)?;
    //     self.do_add_exercise(exercise);
    //     Ok(())
    // }

    pub fn add_exercise(&mut self, exercise: Exercise) {
        assert!(self.validate_add_exercise(&exercise).is_ok());
        self.do_add_exercise(exercise);
    }

    // pub fn try_remove_exercise(&mut self, name: &ExerciseName) -> Result<(), ValidationError> {
    //     self.validate_remove_exercise(name)?;
    //     self.do_remove_exercise(name);
    //     Ok(())
    // }

    pub fn remove_exercise(&mut self, name: &ExerciseName) {
        assert!(self.validate_remove_exercise(name).is_ok());
        self.do_remove_exercise(name);
    }

    pub fn exercises(&self) -> impl Iterator<Item = &Exercise> + '_ {
        self.exercises.iter()
    }

    pub fn find(&self, name: &ExerciseName) -> Option<&Exercise> {
        self.exercises.iter().find(|e| e.name() == name)
    }

    pub fn find_mut(&mut self, name: &ExerciseName) -> Option<&mut Exercise> {
        self.exercises.iter_mut().find(|e| e.name() == name)
    }

    pub fn status(&self, bschedule: BlockSchedule) -> Status {
        self.adjusted_status(Local::now(), bschedule)
    }

    fn adjusted_status(&self, now: DateTime<Local>, bschedule: BlockSchedule) -> Status {
        let status = self.status_from(now);
        // println!("workout: {} status: {status:?} now: {now}", self.name);
        if bschedule.is_active(&self.name) {
            let ok = match status {
                Status::Completed => true,
                Status::Due(n) => bschedule.in_active(now + Duration::days(n as i64)),
                Status::DueAnyTime => true,
                Status::Empty => true,
                Status::Overdue(_) => true,
                Status::PartiallyCompleted => true,
            };
            if ok {
                // println!("   returning status");
                return status;
            }
        }
        if let Some(block_start) = bschedule.next_block_start(&self.name) {
            let status = self.status_from(block_start);
            // println!("   block_start: {block_start} status: {status:?}");
            match status {
                Status::Completed => status,
                Status::Due(n) => {
                    let today = Days::new(now);
                    let then = Days::new(block_start + Duration::days(n as i64));
                    // println!("   today: {today} then: {then} (due)");
                    return Status::Due(then - today);
                }
                Status::DueAnyTime => status,
                Status::Empty => status,
                Status::Overdue(n) => {
                    let today = Days::new(now);
                    let then = Days::new(block_start - Duration::days(n as i64));
                    // println!("   then: {then} (overdue)");
                    return Status::Due(then - today);
                }
                Status::PartiallyCompleted => status,
            }
        } else {
            // println!("   returning status (no next block)");
            status
        }
    }

    fn status_from(&self, now: DateTime<Local>) -> Status {
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
                    let delta = due - today;

                    if delta < 0 && *n > 1 {
                        Status::Overdue(n - 1)
                    } else {
                        Status::Due(delta)
                    }
                } else {
                    Status::Due(*n)
                }
            }
            Schedule::Days(days) => {
                if let Some(last) = last {
                    let last_scheduled = Days::new(self.last_scheduled_date(now, days));
                    let scheduled = self.next_scheduled_date(now, days).unwrap();
                    if last >= last_scheduled {
                        let n = Days::new(scheduled) - today;
                        Status::Due(n)
                    } else if days.contains(&now.weekday()) {
                        Status::Overdue(last_scheduled - last)
                    } else {
                        let n = Days::new(scheduled) - today;
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
        for instance in self.exercises.iter() {
            if let Some(last) = self.completed.get(instance.name()) {
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
    fn weekday_to_days(&self, now: DateTime<Local>, wd: Weekday) -> Days {
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
        now: DateTime<Local>,
        weekdays: &Vec<Weekday>,
    ) -> Option<DateTime<Local>> {
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
    fn last_scheduled_date(
        &self,
        now: DateTime<Local>,
        weekdays: &Vec<Weekday>,
    ) -> DateTime<Local> {
        let mut candidates: Vec<DateTime<Local>> = (1..8)
            .map(|n| now - Duration::days(n))
            .filter(|d| weekdays.contains(&d.weekday()))
            .collect();
        candidates.sort_by(|x, y| (now - x).cmp(&(now - y)));
        candidates[0]
    }

    fn validate_add_exercise(&self, exercise: &Exercise) -> Result<(), ValidationError> {
        let name = exercise.name();
        if name.0.trim().is_empty() {
            return Err(ValidationError::new("The exercise name cannot be empty."));
        } else if self.exercises.iter().find(|e| e.name() == name).is_some() {
            return Err(ValidationError::new("The exercise name must be unique."));
        }
        Ok(())
    }

    fn do_add_exercise(&mut self, exercise: Exercise) {
        self.exercises.push(exercise);
    }

    fn validate_remove_exercise(&self, name: &ExerciseName) -> Result<(), ValidationError> {
        if self.exercises.iter().find(|&e| e.name() == name).is_none() {
            return Err(ValidationError::new("The exercise name doesn't exist."));
        }
        Ok(())
    }

    fn do_remove_exercise(&mut self, name: &ExerciseName) {
        let index = self
            .exercises
            .iter()
            .position(|e| e.name() == name)
            .unwrap();
        self.exercises.remove(index);
        self.completed.remove(&name);
    }

    fn validate_set_exercises(&self, exercises: &Vec<&str>) -> Result<(), ValidationError> {
        let mut names = HashSet::new();
        for name in exercises {
            if name.trim().is_empty() {
                return Err(ValidationError::new("The exercise name cannot be empty."));
            } else {
                let added = names.insert(name.to_owned());
                if !added {
                    return Err(ValidationError::new("'{name}' appears more than once."));
                }
            }
        }
        Ok(())
    }

    fn do_set_exercises(&mut self, exercises: Vec<&str>, disabled: Vec<bool>) {
        let mut new_exercises = Vec::with_capacity(exercises.len());
        for (i, &name) in exercises.iter().enumerate() {
            let mut exercise =
                if let Some(index) = self.exercises.iter().position(|e| e.name().0 == *name) {
                    self.exercises.remove(index)
                } else {
                    default_exercise(name.to_owned())
                };
            if i < disabled.len() {
                let d = exercise.data_mut();
                d.enabled = !disabled[i];
            }
            new_exercises.push(exercise);
        }
        self.exercises = new_exercises;
    }
}

fn default_exercise(name: String) -> Exercise {
    let warmups = vec![
        FixedReps::new(5, 70),
        FixedReps::new(3, 80),
        FixedReps::new(1, 90),
    ];
    let worksets = vec![VariableReps::new(3, 5, 100); 3];
    let e = VariableRepsExercise::new(warmups, worksets);
    let name = ExerciseName(name);
    let formal_name = FormalName("".to_owned());
    BuildExercise::variable_reps(name, formal_name, e)
        .with_rest_mins(2.0)
        .finalize()
}

#[cfg(test)]
mod tests {
    use crate::exercise::FixedRepsExercise;

    use super::*;

    #[test]
    fn empty() {
        let workout = Workout::new("test".to_owned(), Schedule::AnyDay);
        assert_eq!(workout.status_from(Local::now()), Status::Empty);
        // can't test complete because there are no exercises to complete
    }

    #[test]
    fn any_day() {
        let (mut workout, name) = build_squat(Schedule::AnyDay);

        // not completed
        assert_eq!(workout.status_from(Local::now()), Status::DueAnyTime);

        // completed two days ago
        let date = Local::now() - Duration::days(2);
        workout.completed.insert(name.clone(), date);
        assert_eq!(workout.status_from(Local::now()), Status::DueAnyTime);

        // completed yesterday
        let date = Local::now() - Duration::days(1);
        workout.completed.insert(name.clone(), date);
        assert_eq!(workout.status_from(Local::now()), Status::DueAnyTime);

        // completed today
        let date = Local::now();
        workout.completed.insert(name.clone(), date);
        assert_eq!(workout.status_from(Local::now()), Status::Completed);
    }

    #[test]
    fn every_1_days() {
        let (mut workout, name) = build_squat(Schedule::Every(1)); // bit silly

        // not completed
        assert_eq!(workout.status_from(Local::now()), Status::Due(1));

        // completed two days ago
        let date = Local::now() - Duration::days(2);
        workout.completed.insert(name.clone(), date);
        assert_eq!(workout.status_from(Local::now()), Status::Due(0));

        // completed yesterday
        let date = Local::now() - Duration::days(1);
        workout.completed.insert(name.clone(), date);
        assert_eq!(workout.status_from(Local::now()), Status::Due(0));

        // completed today
        let date = Local::now();
        workout.completed.insert(name.clone(), date);
        assert_eq!(workout.status_from(Local::now()), Status::Completed);
    }

    #[test]
    fn every_2_days() {
        let (mut workout, name) = build_squat(Schedule::Every(2));

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

        // completed 1.04 days ago
        let date = now - (Duration::days(1) + Duration::hours(1));
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

    fn date_from_day(wd: Weekday) -> DateTime<Local> {
        let day = 11 + (wd as i32);
        let text = format!("2024-03-{day}T08:00:00Z");
        DateTime::parse_from_rfc3339(&text).unwrap().into()
    }

    fn date_from_day_hour(wd: Weekday, hour: i32) -> DateTime<Local> {
        let day = 11 + (wd as i32);
        let text = format!("2024-03-{day}T{hour:02}:00:00Z");
        DateTime::parse_from_rfc3339(&text).unwrap().into()
    }

    #[test]
    fn days_not_completed1() {
        // due Monday
        let days = vec![Weekday::Mon];
        let (workout, _) = build_squat(Schedule::Days(days));

        let now = date_from_day(Weekday::Mon);
        assert_eq!(workout.status_from(now), Status::Due(0));

        let now = date_from_day(Weekday::Tue);
        assert_eq!(workout.status_from(now), Status::Due(6));

        let now = date_from_day(Weekday::Sat);
        assert_eq!(workout.status_from(now), Status::Due(2));

        // due Friday
        let days = vec![Weekday::Fri];
        let (workout, _) = build_squat(Schedule::Days(days));

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
        let (workout, _) = build_squat(Schedule::Days(days));

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
        let (mut workout, name) = build_squat(Schedule::Days(days));

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
        let (mut workout, name) = build_squat(Schedule::Days(days));

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
        let (mut workout, name1, name2) = build_squat_bench(Schedule::Days(days));

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

    fn build_squat(schedule: Schedule) -> (Workout, ExerciseName) {
        let name = ExerciseName("Squat".to_owned());
        let formal_name = FormalName("Low-bar Squat".to_owned());
        let exercise = FixedRepsExercise::with_reps(vec![5; 3]);
        let exercise = BuildExercise::fixed_reps(name.clone(), formal_name, exercise).finalize();

        let mut workout = Workout::new("Full Body".to_owned(), schedule);
        workout.add_exercise(exercise);

        (workout, name)
    }

    fn build_squat_bench(schedule: Schedule) -> (Workout, ExerciseName, ExerciseName) {
        let name1 = ExerciseName("Squat".to_owned());
        let formal_name = FormalName("Low-bar Squat".to_owned());
        let exercise = FixedRepsExercise::with_reps(vec![5; 3]);
        let exercise1 = BuildExercise::fixed_reps(name1.clone(), formal_name, exercise).finalize();

        let name2 = ExerciseName("Bench".to_owned());
        let formal_name = FormalName("Bench Press".to_owned());
        let exercise = FixedRepsExercise::with_reps(vec![5; 3]);
        let exercise2 = BuildExercise::fixed_reps(name2.clone(), formal_name, exercise).finalize();

        let mut workout = Workout::new("Full Body".to_owned(), schedule);
        workout.add_exercise(exercise1);
        workout.add_exercise(exercise2);

        (workout, name1, name2)
    }
}
