use crate::{
    exercise::ExerciseName,
    pages::{InternalError, SharedState},
    program::Program,
    weights::{self, Weights},
};
use anyhow::Context;
use serde::{Deserialize, Serialize};

pub fn get_edit_weight_page(
    state: SharedState,
    workout: &str,
    exercise: &str,
) -> Result<String, InternalError> {
    let handlebars = &state.read().unwrap().handlebars;
    let weights = &state.read().unwrap().user.weights;
    let program = &state.read().unwrap().user.program;

    let template = include_str!("../../files/edit_weight.html");
    let data = EditWeightData::new(weights, program, workout, exercise)?;
    Ok(handlebars
        .render_template(template, &data)
        .context("failed to render template")?)
}

#[derive(Serialize, Deserialize)]
struct EditWeightData {
    workout: String,
    exercise: String,
    weight_set: String,
    weights: Vec<WeightData>,
}

impl EditWeightData {
    fn new(
        weights: &Weights,
        program: &Program,
        workout_name: &str,
        exercise_name: &str,
    ) -> Result<EditWeightData, anyhow::Error> {
        let workout = program.find(&workout_name).unwrap();
        let exercise = workout
            .find(&ExerciseName(exercise_name.to_owned()))
            .unwrap();
        let data = exercise.data();

        let (entries, weight_set) = if let Some(name) = &data.weightset {
            if let Some(current) = data.weight {
                let min = weights.closest(&name, (current - 30.0).max(0.0)).value();
                let max = current + 30.0;

                let mut v = vec![WeightData::none()];
                let mut value = min;
                loop {
                    if (value - current).abs() < 0.001 {
                        // Note that if a selection is not found the first weight will be
                        // selected.
                        v.push(WeightData::selected(value));
                    } else {
                        v.push(WeightData::new(value));
                    }
                    let next = weights.advance(&name, value).value();
                    if (next - value).abs() < 0.001 || next > max {
                        break;
                    }
                    value = next;
                }
                (v, name.clone())
            } else {
                let mut v = vec![WeightData::none()];
                let mut value = weights.closest(&name, 0.0).value();
                loop {
                    v.push(WeightData::new(value));
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
                    WeightData::new(5.0),
                    WeightData::new(10.0),
                    WeightData::new(15.0),
                    WeightData::selected(20.0),
                    WeightData::new(25.0),
                    WeightData::new(30.0),
                ],
                "?".to_owned(),
            )
        };

        Ok(EditWeightData {
            workout: workout_name.to_owned(),
            exercise: exercise_name.to_owned(),
            weight_set,
            weights: entries,
        })
    }
}

#[derive(Serialize, Deserialize)]
struct WeightData {
    weight: String,
    selected: String,
}

impl WeightData {
    fn new(weight: f32) -> WeightData {
        WeightData {
            weight: weights::format_weight(weight, " lbs"),
            selected: "".to_owned(),
        }
    }

    fn none() -> WeightData {
        WeightData {
            weight: "None".to_owned(),
            selected: "".to_owned(),
        }
    }

    fn selected(weight: f32) -> WeightData {
        WeightData {
            weight: weights::format_weight(weight, " lbs"),
            selected: "selected".to_owned(),
        }
    }
}
