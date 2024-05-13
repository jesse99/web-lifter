use crate::app_state::SharedState;
use crate::errors::Error;
use crate::pages::editor_builder::*;
use crate::weights::{Plate, WeightSet};
use crate::{
    exercise::ExerciseName,
    weights::{self, Weights},
};
use axum::http::Uri;

pub fn get_edit_plate_set(state: SharedState, workout: &str, exercise: &str) -> String {
    fn make_labels(weights: &Weights, set_name: &str) -> (Vec<String>, Option<f32>) {
        if let Some(set) = weights.get(set_name) {
            match set {
                weights::WeightSet::Discrete(_) => panic!("expected plate weights"),
                weights::WeightSet::DualPlates(plates, bar) => (
                    plates
                        .iter()
                        .map(|p| {
                            format!("{} x{}", weights::format_weight(p.weight, " lbs"), p.count,)
                        })
                        .collect(),
                    *bar,
                ),
            }
        } else {
            (Vec::new(), None)
        }
    }

    let post_url = format!("/set-plates-set/{workout}/{exercise}");
    let cancel_url = format!("/exercise/{workout}/{exercise}");

    let weights = &state.read().unwrap().user.weights;
    let program = &state.read().unwrap().user.program;
    let workout = program.find(workout).unwrap();
    let exercise = workout.find(&ExerciseName(exercise.to_owned())).unwrap();
    let data = exercise.data();

    let buttons = vec![
        EditButton::new("add-btn", "", "Add…")
            .with_attr("data-bs-toggle", "modal")
            .with_attr("data-bs-target", "#add_modal"),
        EditButton::new("delete-btn", "on_delete()", "Delete"),
    ];
    let set_name = data.weightset.clone().unwrap(); // only land in this function if there is a weightset
    let (items, bar) = make_labels(weights, &set_name);
    let javascript = include_str!("../../files/plates.js");
    let modal = include_str!("../../files/plates-modal.html");

    let widgets: Vec<Box<dyn Widget>> = vec![
        Box::new(Prolog::with_edit_menu("Edit Plates", buttons, javascript)),
        Box::new(
            TextInput::new(
                "Name",
                &set_name,
                "The name of the weight set, e.g. \"Deadlift\".",
            )
            .with_required(),
        ),
        Box::new(FloatInput::new(
            "Bar",
            bar,
            "Optional fixed weight, usually for a barbell.",
        )),
        Box::new(List::with_names("weights", items, "The plates in the weight set.").without_js()),
        Box::new(Html::new(modal)),
        Box::new(StdButtons::new(&cancel_url)),
    ];

    build_editor(&post_url, widgets)
}

pub fn post_set_plate_set(
    state: SharedState,
    workout: &str,
    exercise: &str,
    set_name: &str,
    plates: Vec<(f32, i32)>,
    bar: Option<f32>,
) -> Result<Uri, Error> {
    let path = format!("/exercise/{workout}/{exercise}");
    let exercise = ExerciseName(exercise.to_owned());
    let plates = plates.iter().map(|(w, c)| Plate::new(*w, *c)).collect();

    {
        let old_name = {
            let program = &state.write().unwrap().user.program;
            let workout = program.find(&workout).unwrap();
            let exercise = workout.find(&exercise).unwrap();
            let d = exercise.data();
            d.weightset.clone().map_or("".to_string(), |s| s)
        };
        {
            let weights = WeightSet::DualPlates(plates, bar);
            let wghts = &mut state.write().unwrap().user.weights;
            wghts.try_change_set(&old_name, set_name, weights)?;
        }
        if old_name != set_name {
            let program = &mut state.write().unwrap().user.program;
            let workout = program.find_mut(&workout).unwrap();
            let exercise = workout.find_mut(&exercise).unwrap();
            let d = exercise.data_mut();
            d.weightset = Some(set_name.to_string()); // try_change_set has handled name validation
        }
    }

    super::post_epilog(state, &path)
}
