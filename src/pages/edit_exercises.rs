use super::{EditItem, EditorBuilder, ListItem, SharedState};
use axum::http::Uri;

pub fn get_edit_exercises(state: SharedState, workout: &str) -> String {
    let post_url = format!("/set-exercises/{workout}");
    let cancel_url = format!("/workout/{workout}");

    let program = &state.read().unwrap().user.program;
    let workout = program.find(workout).unwrap();

    let buttons = vec![
        EditItem::new("delete-btn", "on_delete()", "Delete"),
        EditItem::new("disable-btn", "on_disable()", "Disable"),
        EditItem::new("enable-btn", "on_enable()", "Enable"),
        EditItem::new("down-btn", "on_move_down()", "Move Down"),
        EditItem::new("up-btn", "on_move_up()", "Move Up"),
    ];
    let javascript = include_str!("../../files/exercises.js");

    let active = workout.exercises().nth(0).map_or("", |e| &e.name().0);
    let items = workout
        .exercises()
        .map(|e| {
            let d = e.data();
            if d.enabled {
                ListItem::new(&e.name().0)
            } else {
                ListItem::with_class(&e.name().0, &["text-black-50"])
            }
        })
        .collect();
    let help = Some("Ordered list of exercises to perform for the workout.");

    EditorBuilder::new(&post_url)
        .with_edit_dropdown("Edit Exercises", buttons, javascript)
        .with_list("exercises", items, &active, help, true)
        .with_hidden_input("disabled")
        .with_std_buttons(&cancel_url)
        .finalize()
}

pub fn post_set_exercises(
    state: SharedState,
    workout_name: &str,
    enabled: Vec<&str>,
    disabled: Vec<bool>,
) -> Result<Uri, anyhow::Error> {
    {
        let program = &mut state.write().unwrap().user.program;
        let workout = program.find_mut(&workout_name).unwrap();
        workout.try_set_exercises(enabled, disabled)?;
    }

    {
        let user = &mut state.write().unwrap().user;
        if let Err(e) = crate::persist::save(user) {
            user.errors.push(format!("{e}")); // not fatal so we don't return an error
        }
    }

    let path = format!("/workout/{workout_name}");
    let uri = url_escape::encode_path(&path);
    let uri = uri.parse()?;
    Ok(uri)
}
