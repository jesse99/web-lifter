use super::SharedState;
use crate::pages::editor_builder::*;
use crate::{
    exercise::{ExerciseData, ExerciseName},
    weights::{self, Weights},
};

pub fn get_edit_discrete_set(state: SharedState, workout: &str, exercise: &str) -> String {
    fn make_labels(weights: &Weights, set_name: &str, data: &ExerciseData) -> Vec<String> {
        if let Some(current) = data.weight {
            let min = weights
                .closest(&set_name, (current - 30.0).max(0.0))
                .value();
            let max = current + 30.0;

            let mut v = vec!["None".to_string()];
            let mut value = min;
            loop {
                let name = if (value - current).abs() < 0.001 {
                    // Note that if a selection is not found the first weight will be
                    // selected.
                    weights::format_weight(value, " lbs")
                } else {
                    weights::format_weight(value, " lbs")
                };
                v.push(name);
                let next = weights.advance(&set_name, value).value();
                if (next - value).abs() < 0.001 || next > max {
                    break;
                }
                value = next;
            }
            v
        } else {
            let mut v = vec!["None".to_string()];
            let mut value = weights.closest(&set_name, 0.0).value();
            loop {
                let name = weights::format_weight(value, " lbs");
                v.push(name);
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

    let buttons = [
        ("add-btn", "on_add()", "Add…"),
        ("delete-btn", "on_delete()", "Delete"),
    ];
    let set_name = data.weightset.clone().unwrap(); // only land in this function if there is a weightset
    let items = make_labels(weights, &set_name, data);
    let javascript = include_str!("../../files/discrete.js");

    // TODO think this file should be called edit_discrete_set
    // TODO on_load and on_click need to call enable_menu

    let widgets: Vec<Box<dyn Widget>> = vec![
        Box::new(Prolog::with_edit_menu(
            "Edit Discrete Weight",
            &buttons,
            javascript,
        )),
        Box::new(
            TextInput::new(
                "Name",
                &set_name,
                "The name of the weight set, e.g. \"Dumbbells\".",
            )
            .with_required(),
        ),
        Box::new(List::with_names("weights", items, "The weights in the weight set.").without_js()),
        Box::new(StdButtons::new(&cancel_url)),
    ];

    build_editor(&post_url, widgets)
}
