use crate::*;

pub fn get_program_page(state: SharedState) -> String {
    let engine = &state.read().unwrap().engine;
    let program = &state.read().unwrap().program;

    // Note that MDN recommends against using aria tables, see https://developer.mozilla.org/en-US/docs/Web/Accessibility/ARIA/Roles/table_role
    let template = include_str!("../../files/program.html");
    let data = ProgramData::new(program);
    engine.render_template(template, &data).unwrap()
}

#[derive(Serialize, Deserialize)]
struct ProgramData {
    name: String,
    workouts: Vec<WorkoutData>,
}

impl ProgramData {
    fn new(program: &Program) -> ProgramData {
        let workouts = program.workouts().map(|w| WorkoutData::new(w)).collect();
        ProgramData {
            name: program.name.clone(),
            workouts,
        }
    }
}

#[derive(Serialize, Deserialize)]
struct WorkoutData {
    name: String,
    status_label: String,
    status_class: String,
}

impl WorkoutData {
    fn new(workout: &Workout) -> WorkoutData {
        let status = workout.status();
        WorkoutData {
            name: workout.name.clone(),
            status_class: status.to_class().to_owned(),
            status_label: status.to_label(),
        }
    }
}

impl Status {
    fn to_label(&self) -> String {
        match self {
            Status::Completed => "completed".to_owned(),
            Status::Due(0) => "today".to_owned(),
            Status::Due(1) => "tomorrow".to_owned(),
            Status::Due(n) => format!("in {n} days"),
            Status::DueAnyTime => "any day".to_owned(),
            Status::Empty => "no workouts".to_owned(),
            Status::Overdue(1) => "overdue by 1 day".to_owned(),
            Status::Overdue(n) => format!("overdue by {n} days"),
            Status::PartiallyCompleted => "partially completed".to_owned(),
        }
    }

    fn to_class(&self) -> &str {
        match self {
            Status::Completed => "completed",
            Status::Due(0) => "due_today",
            Status::Due(1) => "tomorrow",
            Status::Due(_) => "due_later",
            Status::DueAnyTime => "any_time",
            Status::Empty => "empty",
            Status::Overdue(_) => "overdue",
            Status::PartiallyCompleted => "partial",
        }
    }
}
