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
    http::{header, HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use handlebars::Handlebars;
use pages::{InternalError, SharedState};
use serde::Deserialize;
use std::sync::RwLock;
use tower::ServiceBuilder;
use tower_http::add_extension::AddExtensionLayer;

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
) -> Result<impl IntoResponse, InternalError> {
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
) -> Result<impl IntoResponse, InternalError> {
    let contents = pages::get_exercise_page(state, &workout, &exercise)?;
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
) -> Result<impl IntoResponse, InternalError> {
    let new_url = pages::post_next_exercise_page(state, &workout, &exercise, None)?;

    let mut headers = HeaderMap::new();
    headers.insert(
        "Cache-Control",
        "no-store, must-revalidate".parse().unwrap(),
    );
    headers.insert("Expires", "0".parse().unwrap());
    headers.insert("Location", new_url.parse().unwrap());
    Ok((StatusCode::SEE_OTHER, headers))
}

async fn post_next_var_set(
    Path((workout, exercise)): Path<(String, String)>,
    options: Query<VarRepsOptions>,
    Extension(state): Extension<SharedState>,
) -> Result<impl IntoResponse, InternalError> {
    let new_url = pages::post_next_exercise_page(state, &workout, &exercise, Some(options.0))?;

    let mut headers = HeaderMap::new();
    headers.insert(
        "Cache-Control",
        "no-store, must-revalidate".parse().unwrap(),
    );
    headers.insert("Expires", "0".parse().unwrap());
    headers.insert("Location", new_url.parse().unwrap());
    Ok((StatusCode::SEE_OTHER, headers))
}
