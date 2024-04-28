use super::SharedState;
use crate::pages::editor_builder::*;
use axum::http::Uri;

pub fn get_edit_exercises(state: SharedState, workout: &str) -> String {
    let post_url = format!("/set-exercises/{workout}");
    let cancel_url = format!("/workout/{workout}");

    let program = &state.read().unwrap().user.program;
    let workout = program.find(workout).unwrap();

    let buttons = [
        ("delete-btn", "on_delete()", "Delete"),
        ("disable-btn", "on_disable()", "Disable"),
        ("enable-btn", "on_enable()", "Enable"),
        ("down-btn", "on_move_down()", "Move Down"),
        ("up-btn", "on_move_up()", "Move Up"),
    ];
    let javascript = include_str!("../../files/exercises.js");

    let active = workout.exercises().nth(0).map_or("", |e| &e.name().0);
    let items: Vec<(String, String)> = workout
        .exercises()
        .map(|e| {
            let d = e.data();
            if d.enabled {
                (e.name().0.clone(), "".to_owned())
            } else {
                (e.name().0.clone(), "text-black-50".to_owned())
            }
        })
        .collect();

    let widgets: Vec<Box<dyn Widget>> = vec![
        Box::new(Prolog::with_edit_menu(
            "Edit Exercises",
            &buttons,
            javascript,
        )),
        Box::new(
            List::with_class(
                "exercises",
                items,
                "Ordered list of exercises to perform for the workout.",
            )
            .with_active(active)
            .without_js(),
        ),
        Box::new(HiddenInput::new("disabled")),
        Box::new(StdButtons::new(&cancel_url)),
    ];

    build_editor(&post_url, widgets)
}

pub fn post_set_exercises(
    state: SharedState,
    workout: &str,
    enabled: Vec<&str>,
    disabled: Vec<bool>,
) -> Result<Uri, anyhow::Error> {
    let path = format!("/workout/{workout}");
    {
        let program = &mut state.write().unwrap().user.program;
        let workout = program.find_mut(&workout).unwrap();
        workout.try_set_exercises(enabled, disabled)?;
    }

    super::post_epilog(state, &path)
}
