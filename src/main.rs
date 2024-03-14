use axum::{
    extract::{Extension, Path},
    http::{header, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use chrono::{Utc, Weekday};
use handlebars::Handlebars;
use serde::{Deserialize, Serialize};
use std::{
    sync::{Arc, RwLock},
    time::Duration,
};
use tower::ServiceBuilder;
use tower_http::add_extension::AddExtensionLayer;

mod days;
mod exercise;
mod history;
mod pages;
mod program;
mod workout;

use days::*;
use exercise::*;
use history::*;
use pages::*;
use program::*;
use workout::*;

fn make_program() -> pages::State {
    // exercises
    let exercise = DurationsExercise::new(vec![20; 4]);
    let name = ExerciseName("Quad Stretch".to_owned());
    let formal_name = FormalName("Standing Quad Stretch".to_owned());
    let exercise1 = SetsExercise::durations(name, formal_name, exercise)
        .with_rest(20)
        .finalize();

    let exercise = FixedRepsExercise::new(vec![10; 2]);
    let name = ExerciseName("Side Leg Lift".to_owned());
    let formal_name = FormalName("Side Lying Abduction".to_owned());
    let exercise2 = SetsExercise::fixed_reps(name.clone(), formal_name, exercise).finalize();

    // workouts
    let mut workout1 = Workout::new("Full Body".to_owned(), Schedule::Every(2));
    workout1.apply(WorkoutOp::Add(exercise1));
    workout1.apply(WorkoutOp::Add(exercise2));

    let workout2 = Workout::new("Cardio".to_owned(), Schedule::AnyDay);
    let workout3 = Workout::new(
        "Strong Lifts".to_owned(),
        Schedule::Days(vec![Weekday::Mon, Weekday::Wed, Weekday::Fri]),
    );

    // program
    let mut program = Program::new("My".to_owned());
    program.apply(ProgramOp::Add(workout1));
    program.apply(ProgramOp::Add(workout2));
    program.apply(ProgramOp::Add(workout3));

    let mut history = History::new();
    let record = Record {
        program: program.name.clone(),
        workout: "Full Body".to_owned(),
        date: Utc::now() - chrono::Duration::days(12),
        sets: Some(CompletedSets::Reps(vec![(3, None), (3, None)])),
        comment: None,
    };
    history.add(&name, record);

    let record = Record {
        program: program.name.clone(),
        workout: "Full Body".to_owned(),
        date: Utc::now() - chrono::Duration::days(9),
        sets: Some(CompletedSets::Reps(vec![(5, None), (5, None)])),
        comment: None,
    };
    history.add(&name, record);

    let record = Record {
        program: program.name.clone(),
        workout: "Full Body".to_owned(),
        date: Utc::now() - chrono::Duration::days(6),
        sets: Some(CompletedSets::Reps(vec![(5, None), (4, None)])),
        comment: None,
    };
    history.add(&name, record);

    let record = Record {
        program: program.name.clone(),
        workout: "Full Body".to_owned(),
        date: Utc::now() - chrono::Duration::days(3),
        sets: Some(CompletedSets::Reps(vec![(10, None), (10, None)])),
        comment: None,
    };
    history.add(&name, record);

    State {
        engine: Handlebars::new(),
        history,
        program,
    }
}

#[tokio::main]
async fn main() {
    let state = make_program();

    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/", get(get_program))
        .route("/workout/:name", get(get_workout))
        .route("/exercise/:workout/:exercise", get(get_exercise))
        .route("/exercise/:workout/:exercise/next-set", post(next_set))
        .route("/scripts/exercise.js", get(get_exercise_js))
        .route("/styles/style.css", get(get_styles))
        // `POST /users` goes to `create_user`
        .route("/users", post(create_user))
        .layer(
            ServiceBuilder::new() // TODO: more stuff at https://github.com/tokio-rs/axum/blob/dea36db400f27c025b646e5720b9a6784ea4db6e/examples/key-value-store/src/main.rs
                .layer(AddExtensionLayer::new(SharedState::new(RwLock::new(state))))
                .into_inner(),
        );

    // let listener = tokio::net::TcpListener::bind("10.0.0.75:80").await.unwrap(); // run with sudo
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
    Ok(axum::response::Html(contents))
}

async fn get_workout(
    Path(name): Path<String>,
    Extension(state): Extension<SharedState>,
) -> Result<impl IntoResponse, InternalError> {
    let contents = get_workout_page(state, &name)?;
    Ok(axum::response::Html(contents))
}

async fn get_exercise(
    Path((workout, exercise)): Path<(String, String)>,
    Extension(state): Extension<SharedState>,
) -> Result<impl IntoResponse, InternalError> {
    let contents = get_exercise_page(state, &workout, &exercise)?;
    Ok(axum::response::Html(contents))
}

async fn next_set(
    Path((workout, exercise)): Path<(String, String)>,
    Extension(state): Extension<SharedState>,
) -> Result<impl IntoResponse, InternalError> {
    let contents = get_next_exercise_page(state, &workout, &exercise)?;
    Ok(axum::response::Html(contents))
}

// parse json request body and turn it into a CreateUser instance
async fn create_user(Json(payload): Json<CreateUser>) -> (StatusCode, Json<User>) {
    // insert your application logic here
    let user = User {
        id: 1337,
        username: payload.username,
    };

    // this will be converted into a JSON response
    // with a status code of `201 Created`
    (StatusCode::CREATED, Json(user))
}

// the input to our `create_user` handler
#[derive(Deserialize)]
struct CreateUser {
    username: String,
}

// the output to our `create_user` handler
#[derive(Serialize)]
struct User {
    id: u64,
    username: String,
}
