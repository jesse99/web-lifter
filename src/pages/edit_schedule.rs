use crate::errors::Error;
use crate::pages::editor_builder::*;
// use crate::pages::Error;
use crate::app_state::SharedState;
use crate::validation_err;
use crate::workout::Schedule;
use axum::http::Uri;
use chrono::Weekday;

// This is a bit yucky: it's a GET that acts like a POST...
// TODO could fix this by adding some custom javascript to te button
pub fn get_schedule_daily(state: SharedState, workout: &str) -> String {
    {
        let program = &mut state.write().unwrap().user.program;
        let workout = program.find_mut(workout).unwrap();
        workout.schedule = Schedule::AnyDay;
    }

    url_escape::encode_path(&format!("/workout/{workout}")).into_owned()
}

pub fn get_edit_schedule_nth(state: SharedState, workout: &str) -> String {
    let post_url = format!("/set-schedule-nth/{workout}");
    let cancel_url = format!("/workout/{workout}");

    let program = &state.read().unwrap().user.program;
    let workout = program.find(&workout).unwrap();
    let n = match workout.schedule {
        Schedule::Every(x) => x as f32,
        _ => 3.0,
    };

    let widgets: Vec<Box<dyn Widget>> = vec![
        Box::new(Prolog::with_title("Schedule Every N Days")),
        Box::new(
            FloatInput::new("N", Some(n), "For example, 2 would be every other day.")
                .with_min(1.0)
                .with_step(1.0)
                .with_required(),
        ),
        Box::new(StdButtons::new(&cancel_url)),
    ];

    build_editor(&post_url, widgets)
}

pub fn get_edit_schedule_weekdays(state: SharedState, workout: &str) -> String {
    let post_url = format!("/set-schedule-weekdays/{workout}");
    let cancel_url = format!("/workout/{workout}");

    let program = &state.read().unwrap().user.program;
    let workout = program.find(&workout).unwrap();
    let days = match &workout.schedule {
        Schedule::Days(d) => d.clone(),
        _ => Vec::new(),
    };
    let items = vec![
        // TODO think we'll need javascript and a hidden button to accumulate values
        ("Sunday", "sun", days.contains(&Weekday::Sun)),
        ("Monday", "mon", days.contains(&Weekday::Mon)),
        ("Tuesday", "tues", days.contains(&Weekday::Tue)),
        ("Wednesday", "wed", days.contains(&Weekday::Wed)),
        ("Thursday", "thurs", days.contains(&Weekday::Thu)),
        ("Friday", "fri", days.contains(&Weekday::Fri)),
        ("Saturday", "sat", days.contains(&Weekday::Sat)),
    ];
    let items = items
        .iter()
        .map(|(l, v, e)| (l.to_string(), v.to_string(), *e))
        .collect();

    let widgets: Vec<Box<dyn Widget>> = vec![
        Box::new(Prolog::with_title("Schedule Week Days")),
        Box::new(Checkbox::new(
            "days",
            items,
            "Schedule for one or more days.",
        )),
        Box::new(StdButtons::new(&cancel_url)),
    ];

    build_editor(&post_url, widgets)
}

pub fn post_schedule_nth(state: SharedState, workout: &str, n: i32) -> Result<Uri, Error> {
    {
        let program = &mut state.write().unwrap().user.program;
        let workout = program.find_mut(&workout).unwrap();
        workout.try_set_schedule(Schedule::Every(n))?;
    }

    let path = format!("/workout/{workout}");
    super::post_epilog(state, &path)
}

pub fn post_set_schedule_weekdays(
    state: SharedState,
    workout: &str,
    days: Vec<String>,
) -> Result<Uri, Error> {
    {
        let days = days
            .into_iter()
            .map(|d| match d.as_ref() {
                "sun" => Ok(Weekday::Sun),
                "mon" => Ok(Weekday::Mon),
                "tues" => Ok(Weekday::Tue),
                "wed" => Ok(Weekday::Wed),
                "thurs" => Ok(Weekday::Thu),
                "fri" => Ok(Weekday::Fri),
                "sat" => Ok(Weekday::Sat),
                _ => validation_err!("Expected a weekday name, e.g. 'sun' but found '{d}'."),
            })
            .collect::<Result<Vec<_>, _>>()?;
        let program = &mut state.write().unwrap().user.program;
        let workout = program.find_mut(&workout).unwrap();
        workout.try_set_schedule(Schedule::Days(days))?;
    }

    let path = format!("/workout/{workout}");
    super::post_epilog(state, &path)
}
