use super::SharedState;
use crate::pages::editor_builder::*;
use crate::pages::Error;
use crate::weights::WeightSet;
use axum::http::Uri;

const HELP:&'static str = "Weight sets can be generic or specific to an exercise, e.g.  \"Dumbbbells\" or \"Deadlift\". You can edit a weight set by selecting it within an exercise.";

fn get_weights<F>(state: SharedState, prefix: &str, post_url: &str, valid: F) -> String
where
    F: Fn(&WeightSet) -> bool,
{
    let cancel_url = "/";

    let weights = &state.read().unwrap().user.weights;
    let mut items: Vec<_> = weights
        .items()
        .filter(|(_, s)| valid(s))
        .map(|(n, _)| n.clone())
        .collect();
    items.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let javascript = include_str!("../../files/weight_sets.js");

    let buttons = vec![
        EditButton::new("add-discrete-btn", &format!("on_add('{prefix}')"), "Add"),
        EditButton::new("delete-btn", "on_delete()", "Delete"),
    ];

    let widgets: Vec<Box<dyn Widget>> = vec![
        Box::new(Prolog::with_edit_menu(
            "Edit Weight Sets",
            buttons,
            javascript,
        )),
        Box::new(List::with_names("sets", items, HELP).without_js()),
        Box::new(StdButtons::new(&cancel_url)),
    ];

    build_editor(&post_url, widgets)
}

pub fn get_discrete_weights(state: SharedState) -> String {
    let valid = |w: &WeightSet| match w {
        WeightSet::Discrete(_) => true,
        _ => false,
    };
    get_weights(state, "Discrete", "/set-discrete-weights", valid)
}

pub fn get_plate_weights(state: SharedState) -> String {
    let valid = |w: &WeightSet| match w {
        WeightSet::DualPlates(_, _) => true,
        _ => false,
    };
    get_weights(state, "Plates", "/set-plate-weights", valid)
}

pub fn post_set_discrete_weights(state: SharedState, sets: Vec<String>) -> Result<Uri, Error> {
    let path = "/";

    {
        let weights = &mut state.write().unwrap().user.weights;
        weights.try_set_discrete_weights(sets)?;
    }

    super::post_epilog(state, &path)
}

pub fn post_set_plate_weights(state: SharedState, sets: Vec<String>) -> Result<Uri, Error> {
    let path = "/";

    {
        let weights = &mut state.write().unwrap().user.weights;
        weights.try_set_plate_weights(sets)?;
    }

    super::post_epilog(state, &path)
}
