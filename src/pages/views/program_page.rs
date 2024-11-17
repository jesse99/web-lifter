use crate::app_state::SharedState;
use crate::errors::Error;
use crate::{
    program::Program,
    workout::{Status, Workout},
};
use chrono::{Datelike, Duration, Local};
use serde::{Deserialize, Serialize};

pub fn get_program_page(state: SharedState) -> Result<String, Error> {
    let error = {
        let user = &mut state.write().unwrap().user;
        let e = user.errors.join(", ");
        user.errors.clear();
        e
    };
    // TODO: It'd be nice if handlers could call render_template outside the State lock.
    // Could call Handlebars::new() inside each handler though that looks fairly expensive.
    // Maybe use TLS?
    let handlebars = &state.read().unwrap().handlebars;
    let program = &state.read().unwrap().user.program;

    // Note that MDN recommends against using aria tables, see https://developer.mozilla.org/en-US/docs/Web/Accessibility/ARIA/Roles/table_role
    let template = include_str!("../../../files/program.html");
    let data = ProgramData::new(program, error);
    let contents = handlebars.render_template(template, &data)?;
    Ok(contents)
}

#[derive(Serialize, Deserialize)]
struct ProgramData {
    name: String,
    blocks: Vec<String>,
    workouts: Vec<WorkoutData>,
    error: String,
    week_disabled: String,
}

impl ProgramData {
    fn new(program: &Program, error: String) -> ProgramData {
        let mut workouts = Vec::new();
        for delta in 0..(16 + 1) {
            let date = Local::now() + Duration::days(delta);
            let scheduled = program.find_workouts(date);
            if !scheduled.is_empty() {
                for w in scheduled.iter() {
                    workouts.push(WorkoutData::new(program, w, delta));
                }
            }
        }
        let blocks = program.blocks().map(|b| b.name.clone()).collect();
        let week_disabled = if program.blocks().count() == 0 {
            "disabled".to_string()
        } else {
            "".to_string()
        };
        ProgramData {
            name: program.name.clone(),
            blocks,
            workouts,
            error,
            week_disabled,
        }
    }
}

#[derive(Serialize, Deserialize)]
struct WorkoutData {
    name: String,
    status_label: String,
    status_class: String,
}

// /workout/{{this.name}}
impl WorkoutData {
    fn new(program: &Program, workout: &Workout, delta: i64) -> WorkoutData {
        let bschedule = program.block_schedule();
        let status = workout.status(bschedule);
        WorkoutData {
            name: workout.name.clone(),
            status_class: status.to_class().to_owned(), // XXX get rid of all this?
            status_label: delta_to_label(delta),
        }
    }
}

fn delta_to_label(delta: i64) -> String {
    if delta == 0 {
        "today".to_owned()
    } else if delta < 7 {
        let date = Local::now() + Duration::days(delta);
        date.weekday().to_string()
    } else {
        format!("in {delta} days")
    }
}

impl Status {
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
