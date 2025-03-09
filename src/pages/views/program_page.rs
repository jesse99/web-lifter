use crate::app_state::SharedState;
use crate::errors::Error;
use crate::{program::Program, workout::Workout};
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
    notes: String,
}

impl ProgramData {
    fn new(program: &Program, error: String) -> ProgramData {
        let mut workouts = Vec::new();
        for delta in 0..(20 + 1) {
            let date = Local::now() + Duration::days(delta);
            let scheduled = program.find_workouts(date);
            if !scheduled.is_empty() {
                for w in scheduled.iter() {
                    workouts.push(WorkoutData::new(w, delta));
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
            notes: program.notes.clone(),
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
    fn new(workout: &Workout, delta: i64) -> WorkoutData {
        WorkoutData {
            name: workout.name.clone(),
            status_class: delta_to_status(delta),
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

fn delta_to_status(delta: i64) -> String {
    if delta == 0 {
        "due_today".to_owned()
    } else if delta == 1 {
        "tomorrow".to_owned()
    } else {
        "due_later".to_owned()
    }
}
