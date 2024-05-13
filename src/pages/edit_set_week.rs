use crate::app_state::SharedState;
use crate::errors::Error;
use crate::pages::editor_builder::*;
use crate::validation_err;
use axum::http::Uri;
use chrono::Local;

pub fn get_edit_set_week(state: SharedState) -> String {
    let post_url = "/set-week";
    let cancel_url = "/";

    let program = &state.read().unwrap().user.program;
    let week = program.current_block().map_or(1, |(w, _)| w);
    let suffix: Vec<_> = program
        .blocks()
        .map(|b| {
            format!(
                "{} is {} week{}",
                b.name,
                b.num_weeks,
                if b.num_weeks > 1 { "s" } else { "" }
            )
        })
        .collect();
    let suffix = suffix.join("<br>");
    let help = format!("Number of weeks into the current block where<br>{suffix}");

    let widgets: Vec<Box<dyn Widget>> = vec![
        Box::new(Prolog::with_title("Set Current Week")),
        Box::new(
            FloatInput::new("Week", Some(week as f32), &help)
                .with_min(1.0)
                .with_step(1.0)
                .with_required(),
        ),
        Box::new(StdButtons::new(&cancel_url)),
    ];

    build_editor(&post_url, widgets)
}

pub fn post_set_week(state: SharedState, week: i32) -> Result<Uri, Error> {
    let path = "/";

    if week <= 0 {
        return validation_err!("'Week should be greater than zero.");
    }

    {
        let program = &mut state.write().unwrap().user.program;
        program.set_week(Local::now(), week);
    }

    super::post_epilog(state, &path)
}
