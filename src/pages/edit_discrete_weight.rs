use super::{EditorBuilder, ListItem, SharedState};
use crate::{
    exercise::{ExerciseData, ExerciseName},
    pages::EditItem,
    weights::{self, Weights},
};

pub fn get_edit_discrete_weight(state: SharedState, workout: &str, exercise: &str) -> String {
    fn make_labels(weights: &Weights, set_name: &str, data: &ExerciseData) -> Vec<ListItem> {
        if let Some(current) = data.weight {
            let min = weights
                .closest(&set_name, (current - 30.0).max(0.0))
                .value();
            let max = current + 30.0;

            let mut v = vec![ListItem::new("None")];
            let mut value = min;
            loop {
                let name = if (value - current).abs() < 0.001 {
                    // Note that if a selection is not found the first weight will be
                    // selected.
                    weights::format_weight(value, " lbs")
                } else {
                    weights::format_weight(value, " lbs")
                };
                v.push(ListItem::new(&name));
                let next = weights.advance(&set_name, value).value();
                if (next - value).abs() < 0.001 || next > max {
                    break;
                }
                value = next;
            }
            v
        } else {
            let mut v = vec![ListItem::new("None")];
            let mut value = weights.closest(&set_name, 0.0).value();
            loop {
                let name = &weights::format_weight(value, " lbs");
                v.push(ListItem::new(&name));
                let next = weights.advance(&set_name, value).value();
                if (next - value).abs() < 0.001 || v.len() > 20 {
                    break;
                }
                value = next;
            }
            v
        }
    }

    let post_url = format!("/set-any-weight/{workout}/{exercise}");
    let cancel_url = format!("/exercise/{workout}/{exercise}");

    let weights = &state.read().unwrap().user.weights;
    let program = &state.read().unwrap().user.program;
    let workout = program.find(workout).unwrap();
    let exercise = workout.find(&ExerciseName(exercise.to_owned())).unwrap();
    let data = exercise.data();

    let buttons = vec![
        EditItem::new("add-btn", "on_add()", "Add…"),
        EditItem::new("delete-btn", "on_delete()", "Delete"),
        EditItem::new("disable-btn", "on_edit()", "Edit…"),
    ];
    let set_name = data.weightset.clone().unwrap(); // only land in this function if there is a weightset
    let items = make_labels(weights, &set_name, data);

    // TODO think this file should be called edit_discrete_set
    // TODO on_load and on_click need to call enable_menu
    // TODO disable delete and edit if no selection

    // TODO do something with list help, another with method?
    let javascript = "";
    EditorBuilder::new(&post_url)
        .with_edit_dropdown("Edit Discrete Weight", buttons, javascript)
        .with_list("weights", items, &set_name)
        // add list
        .with_std_buttons(&cancel_url)
        .finalize()
}
