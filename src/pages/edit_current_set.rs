use super::SharedState;
use crate::pages::editor_builder::*;
use crate::pages::Error;
use crate::{
    exercise::ExerciseName,
    weights::{self, WeightSet},
};
use axum::http::Uri;

pub fn get_current_set(state: SharedState, workout: &str, exercise: &str) -> String {
    fn get_help(ws: &WeightSet) -> String {
        match ws {
            WeightSet::Discrete(weights) => {
                weights
                    .iter()
                    .map(|w| weights::format_weight(*w, ""))
                    .collect::<Vec<_>>()
                    .join(", ")
                    + " lbs"
            }
            WeightSet::DualPlates(plates, bar) => {
                plates
                    .iter()
                    .map(|p| weights::format_weight(p.weight, ""))
                    .collect::<Vec<_>>()
                    .join(", ")
                    + " lbs"
                    + &if let Some(bar) = bar {
                        format!(" with {} bar", weights::format_weight(*bar, " lbs"))
                    } else {
                        "".to_owned()
                    }
            }
        }
    }

    let post_url = format!("/set-current-set/{workout}/{exercise}");
    let cancel_url = format!("/exercise/{workout}/{exercise}");

    let weights = &state.read().unwrap().user.weights;
    let program = &state.read().unwrap().user.program;
    let workout = program.find(workout).unwrap();
    let exercise = workout.find(&ExerciseName(exercise.to_owned())).unwrap();
    let data = exercise.data();

    let active = data.weightset.clone().map_or("None".to_owned(), |n| n);
    let mut items: Vec<_> = weights
        .items()
        .map(|(n, ws)| (n.clone(), get_help(ws)))
        .collect();
    items.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    items.push(("None".to_string(), "no weights".to_string()));

    let widgets: Vec<Box<dyn Widget>> = vec![
        Box::new(Prolog::with_title("Select Weight Set")),
        Box::new(List::with_help("sets", items, "").with_active(&active)),
        Box::new(StdButtons::new(&cancel_url)),
    ];

    build_editor(&post_url, widgets)
}

pub fn post_set_current_set(
    state: SharedState,
    workout: &str,
    exercise: &str,
    sets: String,
) -> Result<Uri, Error> {
    let path = format!("/exercise/{workout}/{exercise}");
    let exercise = ExerciseName(exercise.to_owned());

    {
        let program = &mut state.write().unwrap().user.program;
        let workout = program.find_mut(&workout).unwrap();
        let exercise = workout.find_mut(&exercise).unwrap();
        if sets == "None" {
            exercise.try_set_weight_set(None)?;
        } else {
            exercise.try_set_weight_set(Some(sets))?;
        }
    }

    super::post_epilog(state, &path)
}
