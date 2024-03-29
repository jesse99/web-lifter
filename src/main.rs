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

use axum::{
    extract::{Extension, Path, Query},
    http::{header, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use chrono::{Local, Weekday};
use days::*;
use exercise::*;
use handlebars::Handlebars;
use history::*;
use notes::*;
use pages::*;
use program::*;
use serde::{Deserialize, Serialize};
use std::{
    io::ErrorKind,
    sync::{Arc, RwLock},
};
use tower::ServiceBuilder;
use tower_http::add_extension::AddExtensionLayer;
use weights::*;
use workout::*;

#[tokio::main]
async fn main() {
    let state = default::make_program();

    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/", get(get_program))
        .route("/workout/:name", get(get_workout))
        .route("/exercise/:workout/:exercise", get(get_exercise))
        .route("/exercise/:workout/:exercise/next-set", post(post_next_set))
        .route(
            "/exercise/:workout/:exercise/next-var-set",
            post(post_next_var_set),
        )
        .route("/scripts/exercise.js", get(get_exercise_js))
        .route("/styles/style.css", get(get_styles))
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

async fn get_program(
    Extension(state): Extension<SharedState>,
) -> Result<impl IntoResponse, InternalError> {
    let contents = get_program_page(state)?;
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
) -> Result<impl IntoResponse, InternalError> {
    let error = String::new();
    let contents = get_workout_page(state, &name, error)?;
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
) -> Result<impl IntoResponse, InternalError> {
    let contents = get_exercise_page(state, &workout, &exercise)?;
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

async fn post_next_set(
    Path((workout, exercise)): Path<(String, String)>,
    Extension(state): Extension<SharedState>,
) -> Result<impl IntoResponse, InternalError> {
    let contents = post_next_exercise_page(state, &workout, &exercise, None)?;
    Ok((
        [
            ("Cache-Control", "no-store, must-revalidate"),
            ("Expires", "0"),
        ],
        axum::response::Html(contents),
    ))
}

async fn post_next_var_set(
    Path((workout, exercise)): Path<(String, String)>,
    options: Query<VarRepsOptions>,
    Extension(state): Extension<SharedState>,
) -> Result<impl IntoResponse, InternalError> {
    let contents = post_next_exercise_page(state, &workout, &exercise, Some(options.0))?;
    Ok((
        [
            ("Cache-Control", "no-store, must-revalidate"),
            ("Expires", "0"),
        ],
        axum::response::Html(contents),
    ))
}
