use crate::app_state::SharedState;
use crate::errors::Error;
use crate::pages::editor_builder::*;
use axum::http::Uri;

pub fn get_edit_workouts(state: SharedState) -> String {
    let post_url = format!("/set-workouts");
    let cancel_url = format!("/");

    let program = &state.read().unwrap().user.program;

    let buttons = vec![
        EditButton::new("delete-btn", "on_delete()", "Delete"),
        EditButton::new("disable-btn", "on_disable()", "Disable"),
        EditButton::new("enable-btn", "on_enable()", "Enable"),
        EditButton::new("down-btn", "on_move_down()", "Move Down"),
        EditButton::new("up-btn", "on_move_up()", "Move Up"),
    ];
    let javascript = include_str!("../../files/editable-list.js");

    let active = program.workouts().nth(0).map_or("", |w| &w.name);
    let items: Vec<(String, String)> = program
        .workouts()
        .map(|w| {
            if w.enabled {
                (w.name.clone(), "".to_owned())
            } else {
                (w.name.clone(), "text-black-50".to_owned())
            }
        })
        .collect();

    let widgets: Vec<Box<dyn Widget>> = vec![
        Box::new(Prolog::with_edit_menu("Edit Workouts", buttons, javascript)),
        Box::new(
            List::with_class(
                "names",
                items,
                "Ordered list of workouts to perform for the program.",
            )
            .with_active(active)
            .without_js(),
        ),
        Box::new(HiddenInput::new("disabled")),
        Box::new(StdButtons::new(&cancel_url)),
    ];

    build_editor(&post_url, widgets)
}

pub fn post_set_workouts(
    state: SharedState,
    enabled: Vec<&str>,
    disabled: Vec<bool>,
) -> Result<Uri, Error> {
    let path = format!("/");
    {
        let program = &mut state.write().unwrap().user.program;
        program.try_set_workouts(enabled, disabled)?;
    }

    super::post_epilog(state, &path)
}
