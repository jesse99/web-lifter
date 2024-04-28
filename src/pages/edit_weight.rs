use crate::pages::editor_builder::*;
use crate::weights;
use crate::{exercise::ExerciseName, pages::SharedState};

pub fn get_edit_weight(state: SharedState, workout: &str, exercise: &str) -> String {
    let post_url = format!("/set-weight/{workout}/{exercise}");
    let cancel_url = format!("/exercise/{workout}/{exercise}");

    let weights = &state.read().unwrap().user.weights;
    let program = &state.read().unwrap().user.program;
    let workout = program.find(&workout).unwrap();
    let exercise = workout.find(&ExerciseName(exercise.to_owned())).unwrap();
    let data = exercise.data();

    let mut active = "".to_string();
    let (items, weight_set) = if let Some(name) = &data.weightset {
        if let Some(current) = data.weight {
            let min = weights.closest(&name, (current - 30.0).max(0.0)).value();
            let max = current + 30.0;

            let mut v = vec!["None".to_string()];
            let mut value = min;
            loop {
                let body = weights::format_weight(value, " lbs");
                if (value - current).abs() < 0.001 {
                    // Note that if a selection is not found the first weight will be
                    // selected.
                    active = body.clone();
                }
                v.push(body);
                let next = weights.advance(&name, value).value();
                if (next - value).abs() < 0.001 || next > max {
                    break;
                }
                value = next;
            }
            (v, name.clone())
        } else {
            let mut v = vec!["None".to_string()];
            let mut value = weights.closest(&name, 0.0).value();
            loop {
                v.push(weights::format_weight(value, " lbs"));
                let next = weights.advance(&name, value).value();
                if (next - value).abs() < 0.001 || v.len() > 20 {
                    break;
                }
                value = next;
            }
            (v, name.clone())
        }
    } else {
        // TODO do better here
        (
            vec![
                weights::format_weight(5.0, " lbs"),
                weights::format_weight(10.0, " lbs"),
                weights::format_weight(15.0, " lbs"),
                weights::format_weight(20.0, " lbs"),
                weights::format_weight(25.0, " lbs"),
                weights::format_weight(30.0, " lbs"),
            ],
            "?".to_owned(),
        )
    };
    let items: Vec<_> = items.iter().map(|b| (b.as_ref(), b.as_ref())).collect();

    let help = format!("Using weights from the \"{weight_set}\" weight set.");

    let widgets: Vec<Box<dyn Widget>> = vec![
        Box::new(Prolog::with_title("Edit Weight")),
        Box::new(
            Dropdown::new("Weight", &items[..], "")
                .with_active(&active)
                .with_help(&help),
        ),
        Box::new(StdButtons::new(&cancel_url)),
    ];

    build_editor(&post_url, widgets)
}
