use crate::{
    exercise::ExerciseName,
    pages::SharedState,
    program::Program,
    weights::{self, WeightSet, Weights},
};
use anyhow::Context;
use serde::{Deserialize, Serialize};

pub fn get_current_set_page(
    state: SharedState,
    workout: &str,
    exercise: &str,
) -> Result<String, anyhow::Error> {
    let handlebars = &state.read().unwrap().handlebars;
    let weights = &state.read().unwrap().user.weights;
    let program = &state.read().unwrap().user.program;

    let template = include_str!("../../files/edit_current_set.html");
    let data = EditCurrentData::new(weights, program, workout, exercise)?;
    Ok(handlebars
        .render_template(template, &data)
        .context("failed to render template")?)
}

#[derive(Serialize, Deserialize)]
struct EditCurrentData {
    workout: String,
    exercise: String,
    sets: Vec<Set>,
}

impl EditCurrentData {
    fn new(
        weights: &Weights,
        program: &Program,
        workout_name: &str,
        exercise_name: &str,
    ) -> Result<EditCurrentData, anyhow::Error> {
        let workout = program.find(&workout_name).unwrap();
        let exercise = workout
            .find(&ExerciseName(exercise_name.to_owned()))
            .unwrap();
        let data = exercise.data();

        let selected = &data.weightset;
        let mut sets: Vec<_> = weights
            .names()
            .map(|n| Set::new(weights, n, selected))
            .collect();
        sets.push(if selected.is_none() {
            Set {
                name: "None".to_owned(),
                active: "active".to_owned(),
                current: "true".to_owned(),
                summary: "no weights".to_owned(),
            }
        } else {
            Set {
                name: "None".to_owned(),
                active: "".to_owned(),
                current: "false".to_owned(),
                summary: "no weights".to_owned(),
            }
        });
        sets.sort_by(|a, b| a.name.partial_cmp(&b.name).unwrap());

        Ok(EditCurrentData {
            workout: workout_name.to_owned(),
            exercise: exercise_name.to_owned(),
            sets,
        })
    }
}

#[derive(Serialize, Deserialize)]
struct Set {
    name: String,
    active: String,
    current: String,
    summary: String,
}

impl Set {
    fn new(weights: &Weights, name: &str, selected: &Option<String>) -> Set {
        let ws = weights.get(name).unwrap();
        let summary = match ws {
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
        };
        if let Some(selected) = selected {
            if selected == name {
                return Set {
                    name: name.to_owned(),
                    active: "active".to_owned(),
                    current: "true".to_owned(),
                    summary,
                };
            }
        }
        Set {
            name: name.to_owned(),
            active: "".to_owned(),
            current: "false".to_owned(),
            summary,
        }
    }
}
