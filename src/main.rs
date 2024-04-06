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
        .route("/edit-note/:workout/:exercise", get(get_edit_note))
        .route("/edit-rest/:workout/:exercise", get(get_edit_rest))
        .route("/scripts/exercise.js", get(get_exercise_js))
        .route("/scripts/rest.js", get(get_rest_js))
        .route("/scripts/durations.js", get(get_durations_js))
        .route("/styles/style.css", get(get_styles))
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
        .route("/set-rest/:workout/:exercise", post(post_set_rest))
        .route(
            "/set-any-weight/:workout/:exercise",
            post(post_set_any_weight),
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

async fn get_styles(Extension(_state): Extension<SharedState>) -> impl IntoResponse {
    let contents = include_str!("../files/styles.css");
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/css")],
        contents,
    )
}

async fn get_exercise_js(Extension(_state): Extension<SharedState>) -> impl IntoResponse {
    let contents = include_str!("../files/exercise.js");
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/javascript")],
        contents,
    )
}

async fn get_durations_js(Extension(_state): Extension<SharedState>) -> impl IntoResponse {
    let contents = include_str!("../files/durations.js");
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/javascript")],
        contents,
    )
}

async fn get_rest_js(Extension(_state): Extension<SharedState>) -> impl IntoResponse {
    let contents = include_str!("../files/rest.js");
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/javascript")],
        contents,
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
