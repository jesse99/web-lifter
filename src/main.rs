mod days;
mod default;
mod exercise;
mod history;
mod notes;
mod pages;
mod persist;
mod program;
mod weights;
mod workout;

use anyhow::Context;
use axum::{
    extract::{Extension, Path, Query},
    http::{header, HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Form, Router,
};
use handlebars::Handlebars;
use pages::{AppError, SharedState};
use serde::Deserialize;
use std::sync::RwLock;
use tower::ServiceBuilder;
use tower_http::add_extension::AddExtensionLayer;

use crate::exercise::{FixedReps, VariableReps};

#[tokio::main]
async fn main() {
    let state = default::make_program();

    tracing_subscriber::fmt::init();
    let app = Router::new()
        // get ---------------------------------------------------------------------------
        .route("/", get(get_program))
        .route("/workout/:name", get(get_workout))
        .route("/exercise/:workout/:exercise", get(get_exercise))
        .route("/edit-weight/:workout/:exercise", get(get_edit_weight))
        .route(
            "/edit-any-weight/:workout/:exercise",
            get(get_edit_any_weight),
        )
        .route(
            "/edit-durations/:workout/:exercise",
            get(get_edit_durations),
        )
        .route(
            "/edit-fixed-reps/:workout/:exercise",
            get(get_edit_fixed_reps),
        )
        .route("/edit-var-reps/:workout/:exercise", get(get_edit_var_reps))
        .route("/edit-var-sets/:workout/:exercise", get(get_edit_var_sets))
        .route("/edit-note/:workout/:exercise", get(get_edit_note))
        .route("/edit-rest/:workout/:exercise", get(get_edit_rest))
        .route(
            "/edit-durs-record/:workout/:exercise/:id",
            get(get_edit_durs_record),
        )
        .route(
            "/edit-reps-record/:workout/:exercise/:id",
            get(get_edit_reps_record),
        )
        .route(
            "/scripts/exercise.js",
            get(|s| get_js(s, include_str!("../files/exercise.js"))),
        )
        .route(
            "/scripts/rest.js",
            get(|s| get_js(s, include_str!("../files/rest.js"))),
        )
        .route(
            "/scripts/durations.js",
            get(|s| get_js(s, include_str!("../files/durations.js"))),
        )
        .route(
            "/styles/style.css",
            get(|s| get_css(s, include_str!("../files/styles.css"))),
        )
        // post --------------------------------------------------------------------------
        .route("/exercise/:workout/:exercise/next-set", post(post_next_set))
        .route(
            "/exercise/:workout/:exercise/next-var-set",
            post(post_next_var_set),
        )
        .route("/reset/exercise/:workout/:exercise", post(reset_exercise))
        .route("/set-weight/:workout/:exercise", post(post_set_weight))
        .route("/revert-note/:workout/:exercise", post(post_revert_note))
        .route("/set-note/:workout/:exercise", post(post_set_note))
        .route(
            "/set-durations/:workout/:exercise",
            post(post_set_durations),
        )
        .route(
            "/set-fixed-reps/:workout/:exercise",
            post(post_set_fixed_reps),
        )
        .route("/set-var-reps/:workout/:exercise", post(post_set_var_reps))
        .route("/set-var-sets/:workout/:exercise", post(post_set_var_sets))
        .route("/set-rest/:workout/:exercise", post(post_set_rest))
        .route(
            "/set-any-weight/:workout/:exercise",
            post(post_set_any_weight),
        )
        .route(
            "/set-durs-record/:workout/:exercise/:id",
            post(post_set_durs_record),
        )
        .route(
            "/set-reps-record/:workout/:exercise/:id",
            post(post_set_reps_record),
        )
        // layer -------------------------------------------------------------------------
        .layer(
            ServiceBuilder::new() // TODO: more stuff at https://github.com/tokio-rs/axum/blob/dea36db400f27c025b646e5720b9a6784ea4db6e/examples/key-value-store/src/main.rs
                .layer(AddExtensionLayer::new(SharedState::new(RwLock::new(state))))
                .into_inner(),
        );

    // TODO This is currently setup to use port forwarding so that, from the public
    // Internet, clients can hit my router IP which will then forward to my Mac. But we'll
    // need to do something different when/if we deploy for real. At a minimum switch to
    // using port 80 (or 443).
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn get_css(Extension(_state): Extension<SharedState>, contents: &str) -> impl IntoResponse {
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/css")],
        contents.to_owned(),
    )
}

async fn get_js(Extension(_state): Extension<SharedState>, contents: &str) -> impl IntoResponse {
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/javascript")],
        contents.to_owned(),
    )
}

async fn get_program(
    Extension(state): Extension<SharedState>,
) -> Result<impl IntoResponse, AppError> {
    let contents = pages::get_program_page(state)?;
    Ok((
        [
            ("Cache-Control", "no-store, must-revalidate"),
            ("Expires", "0"),
        ],
        axum::response::Html(contents),
    ))
}

async fn get_workout(
    Path(name): Path<String>,
    Extension(state): Extension<SharedState>,
) -> Result<impl IntoResponse, AppError> {
    let contents = pages::get_workout_page(state, &name)?;
    Ok((
        [
            ("Cache-Control", "no-store, must-revalidate"),
            ("Expires", "0"),
        ],
        axum::response::Html(contents),
    ))
}

async fn get_exercise(
    Path((workout, exercise)): Path<(String, String)>,
    Extension(state): Extension<SharedState>,
) -> Result<impl IntoResponse, AppError> {
    let contents = pages::get_exercise_page(state, &workout, &exercise)?;
    Ok((
        [
            ("Cache-Control", "no-store, must-revalidate"),
            ("Expires", "0"),
        ],
        axum::response::Html(contents),
    ))
}

async fn get_edit_weight(
    Path((workout, exercise)): Path<(String, String)>,
    Extension(state): Extension<SharedState>,
) -> Result<impl IntoResponse, AppError> {
    let contents = pages::get_edit_weight_page(state, &workout, &exercise)?;
    Ok((
        [
            ("Cache-Control", "no-store, must-revalidate"),
            ("Expires", "0"),
        ],
        axum::response::Html(contents),
    ))
}

async fn get_edit_durations(
    Path((workout, exercise)): Path<(String, String)>,
    Extension(state): Extension<SharedState>,
) -> Result<impl IntoResponse, AppError> {
    let contents = pages::get_edit_durations_page(state, &workout, &exercise)?;
    Ok((
        [
            ("Cache-Control", "no-store, must-revalidate"),
            ("Expires", "0"),
        ],
        axum::response::Html(contents),
    ))
}

async fn get_edit_fixed_reps(
    Path((workout, exercise)): Path<(String, String)>,
    Extension(state): Extension<SharedState>,
) -> Result<impl IntoResponse, AppError> {
    let contents = pages::get_edit_fixed_reps_page(state, &workout, &exercise)?;
    Ok((
        [
            ("Cache-Control", "no-store, must-revalidate"),
            ("Expires", "0"),
        ],
        axum::response::Html(contents),
    ))
}

async fn get_edit_var_reps(
    Path((workout, exercise)): Path<(String, String)>,
    Extension(state): Extension<SharedState>,
) -> Result<impl IntoResponse, AppError> {
    let contents = pages::get_edit_var_reps_page(state, &workout, &exercise)?;
    Ok((
        [
            ("Cache-Control", "no-store, must-revalidate"),
            ("Expires", "0"),
        ],
        axum::response::Html(contents),
    ))
}

async fn get_edit_var_sets(
    Path((workout, exercise)): Path<(String, String)>,
    Extension(state): Extension<SharedState>,
) -> Result<impl IntoResponse, AppError> {
    let contents = pages::get_edit_var_sets_page(state, &workout, &exercise)?;
    Ok((
        [
            ("Cache-Control", "no-store, must-revalidate"),
            ("Expires", "0"),
        ],
        axum::response::Html(contents),
    ))
}

async fn get_edit_note(
    Path((workout, exercise)): Path<(String, String)>,
    Extension(state): Extension<SharedState>,
) -> Result<impl IntoResponse, AppError> {
    let contents = pages::get_edit_note_page(state, &workout, &exercise)?;
    Ok((
        [
            ("Cache-Control", "no-store, must-revalidate"),
            ("Expires", "0"),
        ],
        axum::response::Html(contents),
    ))
}

async fn get_edit_rest(
    Path((workout, exercise)): Path<(String, String)>,
    Extension(state): Extension<SharedState>,
) -> Result<impl IntoResponse, AppError> {
    let contents = pages::get_edit_rest_page(state, &workout, &exercise)?;
    Ok((
        [
            ("Cache-Control", "no-store, must-revalidate"),
            ("Expires", "0"),
        ],
        axum::response::Html(contents),
    ))
}

async fn get_edit_durs_record(
    Path((workout, exercise, id)): Path<(String, String, String)>,
    Extension(state): Extension<SharedState>,
) -> Result<impl IntoResponse, AppError> {
    let id: u64 = id
        .parse()
        .context(format!("expected int for id but found '{id}'"))?;
    let contents = pages::get_edit_durs_record_page(state, &workout, &exercise, id)?;
    Ok((
        [
            ("Cache-Control", "no-store, must-revalidate"),
            ("Expires", "0"),
        ],
        axum::response::Html(contents),
    ))
}

async fn get_edit_reps_record(
    Path((workout, exercise, id)): Path<(String, String, String)>,
    Extension(state): Extension<SharedState>,
) -> Result<impl IntoResponse, AppError> {
    let id: u64 = id
        .parse()
        .context(format!("expected int for id but found '{id}'"))?;
    let contents = pages::get_edit_reps_record_page(state, &workout, &exercise, id)?;
    Ok((
        [
            ("Cache-Control", "no-store, must-revalidate"),
            ("Expires", "0"),
        ],
        axum::response::Html(contents),
    ))
}

async fn get_edit_any_weight(
    Path((workout, exercise)): Path<(String, String)>,
    Extension(state): Extension<SharedState>,
) -> Result<impl IntoResponse, AppError> {
    let contents = pages::get_edit_any_weight_page(state, &workout, &exercise)?;
    Ok((
        [
            ("Cache-Control", "no-store, must-revalidate"),
            ("Expires", "0"),
        ],
        axum::response::Html(contents),
    ))
}

#[derive(Debug, Deserialize)]
struct VarRepsOptions {
    reps: i32,
    update: i32,
    advance: i32,
}

// After posts we do a redirect to a GET page. This prevents silly issues like duplicate
// POSTs when the user presses the refresh button. See https://www.theserverside.com/news/1365146/Redirect-After-Post
// for more.
async fn post_next_set(
    Path((workout, exercise)): Path<(String, String)>,
    Extension(state): Extension<SharedState>,
) -> Result<impl IntoResponse, AppError> {
    let new_url = pages::post_next_exercise(state, &workout, &exercise, None)?;

    let mut headers = HeaderMap::new();
    headers.insert(
        "Cache-Control",
        "no-store, must-revalidate".parse().unwrap(),
    );
    headers.insert("Expires", "0".parse().unwrap());
    headers.insert("Location", new_url.path().parse().unwrap());
    Ok((StatusCode::SEE_OTHER, headers))
}

async fn post_next_var_set(
    Path((workout, exercise)): Path<(String, String)>,
    options: Query<VarRepsOptions>,
    Extension(state): Extension<SharedState>,
) -> Result<impl IntoResponse, AppError> {
    let new_url = pages::post_next_exercise(state, &workout, &exercise, Some(options.0))?;

    let mut headers = HeaderMap::new();
    headers.insert(
        "Cache-Control",
        "no-store, must-revalidate".parse().unwrap(),
    );
    headers.insert("Expires", "0".parse().unwrap());
    headers.insert("Location", new_url.path().parse().unwrap());
    Ok((StatusCode::SEE_OTHER, headers))
}

async fn reset_exercise(
    Path((workout, exercise)): Path<(String, String)>,
    Extension(state): Extension<SharedState>,
) -> Result<impl IntoResponse, AppError> {
    let new_url = pages::post_reset_exercise(state, &workout, &exercise)?;

    let mut headers = HeaderMap::new();
    headers.insert(
        "Cache-Control",
        "no-store, must-revalidate".parse().unwrap(),
    );
    headers.insert("Expires", "0".parse().unwrap());
    headers.insert("Location", new_url.path().parse().unwrap());
    Ok((StatusCode::SEE_OTHER, headers))
}

#[derive(Debug, Deserialize)]
struct SetWeight {
    weight: String, // "25 lbs"
}

// The user experience of failed form validation is not great. The user will get a new
// page with an error message and then have to use the back button to fix the issue.
// Annoying and pretty bad if there are multiple widgets that could have errors. There
// are other possibilites:
// 1) axum has some built in support for validation, see https://github.com/tokio-rs/axum/blob/main/examples/validator/src/main.rs
// However I don't think this will provide a better UX and the validation logic isn't
// together with the backend logic.
// 2) There are also validation crates for use with axum like valitron. Haven't looked at
// these much but I don't think they'll solve the UX issue.
// 3) We could redirect back to the edit page and include an error. That's almost a good
// UX but we'd want to preserve the user's edits which would be pretty annoying.
// 4) We could use javascript and something like AJAX to validate before submitting. That
// would give us a nice UX and allow us to do things like style the offending widget but
// means we'd have parallel code paths for validation and setting which is again quite
// annoying.
async fn post_set_weight(
    Path((workout, exercise)): Path<(String, String)>,
    Extension(state): Extension<SharedState>,
    Form(payload): Form<SetWeight>,
) -> Result<impl IntoResponse, AppError> {
    let parts: Vec<_> = payload.weight.split(" ").collect();
    let s = parts.get(0).context("empty payload")?.to_string();
    let w = if s == "None" {
        None
    } else {
        let x: f32 = s.parse().context(format!("expected f32 but found '{s}'"))?;
        Some(x)
    };
    let new_url = pages::post_set_weight(state, &workout, &exercise, w)?;

    let mut headers = HeaderMap::new();
    headers.insert(
        "Cache-Control",
        "no-store, must-revalidate".parse().unwrap(),
    );
    headers.insert("Expires", "0".parse().unwrap());
    headers.insert("Location", new_url.path().parse().unwrap());
    Ok((StatusCode::SEE_OTHER, headers))
}

// We don't allow None because we want the input type to be Number so that we get a numeric
// keypad on mobile (and we can't use a custom pattern with Number). So we'll just treat
// 0.0 as None.
async fn post_set_any_weight(
    Path((workout, exercise)): Path<(String, String)>,
    Extension(state): Extension<SharedState>,
    Form(payload): Form<SetWeight>,
) -> Result<impl IntoResponse, AppError> {
    let parts: Vec<_> = payload.weight.split(" ").collect();
    let s = parts.get(0).context("empty payload")?.to_string();
    let w: f32 = s.parse().context(format!("expected f32 but found '{s}'"))?;
    let w = if w.abs() < 0.001 { None } else { Some(w) };
    let new_url = pages::post_set_weight(state, &workout, &exercise, w)?;

    let mut headers = HeaderMap::new();
    headers.insert(
        "Cache-Control",
        "no-store, must-revalidate".parse().unwrap(),
    );
    headers.insert("Expires", "0".parse().unwrap());
    headers.insert("Location", new_url.path().parse().unwrap());
    // Ok((StatusCode::BAD_REQUEST, "Weight has to be less than 20.0"))
    Ok((StatusCode::SEE_OTHER, headers))
}

#[derive(Debug, Deserialize)]
struct SetNote {
    note: String,
}

async fn post_set_note(
    Path((workout, exercise)): Path<(String, String)>,
    Extension(state): Extension<SharedState>,
    Form(payload): Form<SetNote>,
) -> Result<impl IntoResponse, AppError> {
    let new_url = pages::post_set_note(state, &workout, &exercise, payload.note)?;

    let mut headers = HeaderMap::new();
    headers.insert(
        "Cache-Control",
        "no-store, must-revalidate".parse().unwrap(),
    );
    headers.insert("Expires", "0".parse().unwrap());
    headers.insert("Location", new_url.path().parse().unwrap());
    Ok((StatusCode::SEE_OTHER, headers))
}

async fn post_revert_note(
    Path((workout, exercise)): Path<(String, String)>,
    Extension(state): Extension<SharedState>,
) -> Result<impl IntoResponse, AppError> {
    let new_url = pages::post_revert_note(state, &workout, &exercise)?;

    let mut headers = HeaderMap::new();
    headers.insert(
        "Cache-Control",
        "no-store, must-revalidate".parse().unwrap(),
    );
    headers.insert("Expires", "0".parse().unwrap());
    headers.insert("Location", new_url.path().parse().unwrap());
    Ok((StatusCode::SEE_OTHER, headers))
}

#[derive(Debug, Deserialize)]
struct SetDurations {
    durations: String,
    target: String,
    units: String, // "secs", "mins", or "hours"
}

async fn post_set_durations(
    Path((workout, exercise)): Path<(String, String)>,
    Extension(state): Extension<SharedState>,
    Form(payload): Form<SetDurations>,
) -> Result<impl IntoResponse, AppError> {
    let durations = payload
        .durations
        .split_whitespace()
        .map(|s| parse_time("durations", s, &payload.units))
        .collect::<Result<Vec<_>, _>>()?;
    let durations = durations.iter().filter_map(|o| *o).collect();
    let target = parse_time("target", &payload.target, &payload.units)?;
    let new_url = pages::post_set_durations(state, &workout, &exercise, durations, target)?;

    let mut headers = HeaderMap::new();
    headers.insert(
        "Cache-Control",
        "no-store, must-revalidate".parse().unwrap(),
    );
    headers.insert("Expires", "0".parse().unwrap());
    headers.insert("Location", new_url.path().parse().unwrap());
    Ok((StatusCode::SEE_OTHER, headers))
}

#[derive(Debug, Deserialize)]
struct SetFixedReps {
    warmups: String,
    worksets: String,
}

async fn post_set_fixed_reps(
    Path((workout, exercise)): Path<(String, String)>,
    Extension(state): Extension<SharedState>,
    Form(payload): Form<SetFixedReps>,
) -> Result<impl IntoResponse, AppError> {
    let warmups = payload
        .warmups
        .split_whitespace()
        .map(|s| parse_fixed_rep("warmups", s))
        .collect::<Result<Vec<_>, _>>()?;
    let worksets = payload
        .worksets
        .split_whitespace()
        .map(|s| parse_fixed_rep("worksets", s))
        .collect::<Result<Vec<_>, _>>()?;
    let new_url = pages::post_set_fixed_reps(state, &workout, &exercise, warmups, worksets)?;

    let mut headers = HeaderMap::new();
    headers.insert(
        "Cache-Control",
        "no-store, must-revalidate".parse().unwrap(),
    );
    headers.insert("Expires", "0".parse().unwrap());
    headers.insert("Location", new_url.path().parse().unwrap());
    Ok((StatusCode::SEE_OTHER, headers))
}

fn parse_fixed_rep(name: &str, value: &str) -> Result<FixedReps, anyhow::Error> {
    let parts: Vec<_> = value.split("/").collect();
    if parts.len() == 0 {
        return Err(anyhow::Error::msg(format!("{name} cannot be empty")));
    } else if parts.len() == 1 {
        let reps: i32 = parts[0]
            .parse()
            .context(format!("expected int for {name} reps but found '{value}'"))?;
        return Ok(FixedReps::new(reps, 100));
    } else if parts.len() == 2 {
        let reps: i32 = parts[0]
            .parse()
            .context(format!("expected int for {name} reps but found '{value}'"))?;
        let percent: i32 = parts[1].parse().context(format!(
            "expected int for {name} percent but found '{value}'"
        ))?;
        return Ok(FixedReps::new(reps, percent));
    } else {
        return Err(anyhow::Error::msg(format!(
            "Expected rep or rep/percent for {name} but found '{value}'"
        )));
    }
}

#[derive(Debug, Deserialize)]
struct SetVarReps {
    warmups: String,
    worksets: String,
}

async fn post_set_var_reps(
    Path((workout, exercise)): Path<(String, String)>,
    Extension(state): Extension<SharedState>,
    Form(payload): Form<SetVarReps>,
) -> Result<impl IntoResponse, AppError> {
    fn parse_rep(name: &str, value: &str, percent: i32) -> Result<VariableReps, anyhow::Error> {
        let parts: Vec<_> = value.split("-").collect();
        if parts.len() == 0 {
            return Err(anyhow::Error::msg(format!("{name} cannot be empty")));
        } else if parts.len() == 1 {
            let reps: i32 = parts[0]
                .parse()
                .context(format!("expected int for {name} reps but found '{value}'"))?;
            return Ok(VariableReps::new(reps, reps, percent));
        } else if parts.len() == 2 {
            let min: i32 = parts[0].parse().context(format!(
                "expected int for {name} min reps but found '{value}'"
            ))?;
            let max: i32 = parts[1].parse().context(format!(
                "expected int for {name} max reps but found '{value}'"
            ))?;
            return Ok(VariableReps::new(min, max, percent));
        } else {
            return Err(anyhow::Error::msg(format!(
                "Expected rep or min_rep-max_rep for {name} but found '{value}'"
            )));
        }
    }

    fn parse_var_rep(name: &str, value: &str) -> Result<VariableReps, anyhow::Error> {
        let parts: Vec<_> = value.split("/").collect();
        if parts.len() == 0 {
            return Err(anyhow::Error::msg(format!("{name} cannot be empty")));
        } else if parts.len() == 1 {
            return parse_rep(name, parts[0], 100);
        } else if parts.len() == 2 {
            let percent: i32 = parts[1].parse().context(format!(
                "expected int for {name} percent but found '{value}'"
            ))?;
            return parse_rep(name, parts[0], percent);
        } else {
            return Err(anyhow::Error::msg(format!(
                "Expected rep or min_rep-max_rep or rep/percent or min_rep-max_rep/percent for {name} but found '{value}'"
            )));
        }
    }

    let warmups = payload
        .warmups
        .split_whitespace()
        .map(|s| parse_fixed_rep("warmups", s))
        .collect::<Result<Vec<_>, _>>()?;
    let worksets = payload
        .worksets
        .split_whitespace()
        .map(|s| parse_var_rep("worksets", s))
        .collect::<Result<Vec<_>, _>>()?;
    let new_url = pages::post_set_var_reps(state, &workout, &exercise, warmups, worksets)?;

    let mut headers = HeaderMap::new();
    headers.insert(
        "Cache-Control",
        "no-store, must-revalidate".parse().unwrap(),
    );
    headers.insert("Expires", "0".parse().unwrap());
    headers.insert("Location", new_url.path().parse().unwrap());
    Ok((StatusCode::SEE_OTHER, headers))
}

#[derive(Debug, Deserialize)]
struct SetVarSets {
    target: String,
}

async fn post_set_var_sets(
    Path((workout, exercise)): Path<(String, String)>,
    Extension(state): Extension<SharedState>,
    Form(payload): Form<SetVarSets>,
) -> Result<impl IntoResponse, AppError> {
    let target: i32 = payload.target.parse().context(format!(
        "expected integer for target but found '{}'",
        payload.target
    ))?;
    let new_url = pages::post_set_var_sets(state, &workout, &exercise, target)?;

    let mut headers = HeaderMap::new();
    headers.insert(
        "Cache-Control",
        "no-store, must-revalidate".parse().unwrap(),
    );
    headers.insert("Expires", "0".parse().unwrap());
    headers.insert("Location", new_url.path().parse().unwrap());
    Ok((StatusCode::SEE_OTHER, headers))
}

#[derive(Debug, Deserialize)]
struct SetRest {
    rest: String,
    last_rest: String,
    units: String, // "secs", "mins", or "hours"
}

async fn post_set_rest(
    Path((workout, exercise)): Path<(String, String)>,
    Extension(state): Extension<SharedState>,
    Form(payload): Form<SetRest>,
) -> Result<impl IntoResponse, AppError> {
    let rest = parse_time("rest", &payload.rest, &payload.units)?;
    let last_rest = parse_time("last_rest", &payload.last_rest, &payload.units)?;
    let new_url = pages::post_set_rest(state, &workout, &exercise, rest, last_rest)?;

    let mut headers = HeaderMap::new();
    headers.insert(
        "Cache-Control",
        "no-store, must-revalidate".parse().unwrap(),
    );
    headers.insert("Expires", "0".parse().unwrap());
    headers.insert("Location", new_url.path().parse().unwrap());
    Ok((StatusCode::SEE_OTHER, headers))
}

#[derive(Debug, Deserialize)]
struct SetDurationsRecord {
    durations: String,
    weights: String,
    comment: String,
    units: String, // "secs", "mins", or "hours"
}

async fn post_set_durs_record(
    Path((workout, exercise, id)): Path<(String, String, String)>,
    Extension(state): Extension<SharedState>,
    Form(payload): Form<SetDurationsRecord>,
) -> Result<impl IntoResponse, AppError> {
    let durations = payload
        .durations
        .split_whitespace()
        .map(|s| parse_time("durations", s, &payload.units))
        .collect::<Result<Vec<_>, _>>()?;
    let durations = durations.iter().filter_map(|o| *o).collect();
    let weights = payload
        .weights
        .split_whitespace()
        .map(|s| s.parse::<f32>())
        .collect::<Result<Vec<_>, _>>()?;
    let id = id
        .parse()
        .context(format!("expected integer for id but found '{id}'"))?;
    let new_url = pages::post_set_durs_record(
        state,
        &workout,
        &exercise,
        durations,
        weights,
        payload.comment,
        id,
    )?;

    let mut headers = HeaderMap::new();
    headers.insert(
        "Cache-Control",
        "no-store, must-revalidate".parse().unwrap(),
    );
    headers.insert("Expires", "0".parse().unwrap());
    headers.insert("Location", new_url.path().parse().unwrap());
    Ok((StatusCode::SEE_OTHER, headers))
}

#[derive(Debug, Deserialize)]
struct SetRepsRecord {
    reps: String,
    weights: String,
    comment: String,
}

async fn post_set_reps_record(
    Path((workout, exercise, id)): Path<(String, String, String)>,
    Extension(state): Extension<SharedState>,
    Form(payload): Form<SetRepsRecord>,
) -> Result<impl IntoResponse, AppError> {
    let reps = payload
        .reps
        .split_whitespace()
        .map(|s| s.parse::<i32>())
        .collect::<Result<Vec<_>, _>>()?;
    let weights = payload
        .weights
        .split_whitespace()
        .map(|s| s.parse::<f32>())
        .collect::<Result<Vec<_>, _>>()?;
    let id = id
        .parse()
        .context(format!("expected integer for id but found '{id}'"))?;
    let new_url = pages::post_set_reps_record(
        state,
        &workout,
        &exercise,
        reps,
        weights,
        payload.comment,
        id,
    )?;

    let mut headers = HeaderMap::new();
    headers.insert(
        "Cache-Control",
        "no-store, must-revalidate".parse().unwrap(),
    );
    headers.insert("Expires", "0".parse().unwrap());
    headers.insert("Location", new_url.path().parse().unwrap());
    Ok((StatusCode::SEE_OTHER, headers))
}

fn parse_time(name: &str, value: &str, units: &str) -> Result<Option<i32>, AppError> {
    if !value.is_empty() {
        let mut x: f32 = value
            .parse()
            .context(format!("expected f32 for {name} but found '{value}'"))?;
        if units == "mins" {
            x *= 60.0;
        }
        if units == "hours" {
            x *= 60.0 * 60.0;
        }
        if x > 0.1 {
            return Ok(Some(x as i32));
        }
    }
    Ok(None)
}
