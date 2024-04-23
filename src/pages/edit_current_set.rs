use axum::http::Uri;

use super::{EditorBuilder, SharedState};
use crate::{
    exercise::ExerciseName,
    pages::ListItem,
    weights::{self, WeightSet},
};

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
        .map(|(n, ws)| ListItem::with_help(n, &get_help(ws)))
        .collect();
    items.sort_by(|a, b| a.name.partial_cmp(&b.name).unwrap());
    items.push(ListItem::with_help("None", "no weights"));

    EditorBuilder::new(&post_url)
        .with_title("Select Weight Set")
        .with_list("sets", items, &active)
        .with_std_buttons(&cancel_url)
        .finalize()
}

pub fn post_set_current_set(
    state: SharedState,
    workout: &str,
    exercise: &str,
    sets: String,
) -> Result<Uri, anyhow::Error> {
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
