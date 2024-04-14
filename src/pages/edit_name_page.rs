use crate::pages::SharedState;
use anyhow::Context;
use serde::{Deserialize, Serialize};

pub fn get_edit_name_page(
    state: SharedState,
    workout: &str,
    exercise: &str,
) -> Result<String, anyhow::Error> {
    let handlebars = &state.read().unwrap().handlebars;

    let template = include_str!("../../files/edit_name.html");
    let data = EditNameData::new(workout, exercise)?;
    Ok(handlebars
        .render_template(template, &data)
        .context("failed to render template")?)
}

#[derive(Serialize, Deserialize)]
struct EditNameData {
    post_url: String,
    cancel_url: String,
    help: String,
    value: String,
}

impl EditNameData {
    fn new(workout: &str, exercise: &str) -> Result<EditNameData, anyhow::Error> {
        let post_url =
            url_escape::encode_path(&format!("/set-name/{}/{}", workout, exercise)).into_owned();
        let cancel_url =
            url_escape::encode_path(&format!("/exercise/{}/{}", workout, exercise)).into_owned();
        let help = "Must be unique within the workout".to_owned();
        let value = exercise.to_owned();

        Ok(EditNameData {
            post_url,
            cancel_url,
            help,
            value,
        })
    }
}
