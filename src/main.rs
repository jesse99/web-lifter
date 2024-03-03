use axum::{
    extract::Extension,
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
use tower::ServiceBuilder;
use tower_http::add_extension::AddExtensionLayer;

mod days;
mod exercise;
mod exercises;
mod program;
mod workout;

use days::*;
use exercise::*;
use exercises::*;
use program::*;
use workout::*;

struct State {
    program: Program,
    exercises: Exercises,
}

type SharedState = Arc<RwLock<State>>;

fn make_program() -> State {
    let mut workout = Workout::new("Mine".to_owned(), Schedule::Every(2));

    let name1 = ExerciseName("Quad Stretch".to_owned());
    workout.apply(WorkoutOp::Add(name1.clone()));

    let name2 = ExerciseName("Side Leg Lift".to_owned());
    workout.apply(WorkoutOp::Add(name2.clone()));

    let mut program = Program::new("Mine".to_owned());
    program.apply(ProgramOp::Add(workout));

    let mut exercises = Exercises::new();
    exercises.apply(ExercisesOp::Add(
        name1,
        Exercise::new("Standing Quad Stretch".to_owned()),
    ));
    exercises.apply(ExercisesOp::Add(
        name2,
        Exercise::new("Side Lying Abduction".to_owned()),
    ));

    State { program, exercises }
}

#[tokio::main]
async fn main() {
    let state = make_program();

    tracing_subscriber::fmt::init();

    // build our application with a route
    let app = Router::new()
        .route("/", get(get_program))
        // `POST /users` goes to `create_user`
        .route("/users", post(create_user))
        .layer(
            ServiceBuilder::new() // TODO: more stuff at https://github.com/tokio-rs/axum/blob/dea36db400f27c025b646e5720b9a6784ea4db6e/examples/key-value-store/src/main.rs
                .layer(AddExtensionLayer::new(SharedState::new(RwLock::new(state))))
                .into_inner(),
        );

    // run our app with hyper
    // let listener = tokio::net::TcpListener::bind("10.0.0.75:80").await.unwrap(); // run with sudo
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn get_program(Extension(state): Extension<SharedState>) -> impl IntoResponse {
    let program = &state.read().unwrap().program;
    Html(
        format!(
            "Hello, you're running the <strong>{}</strong> program!!!",
            program.name
        )
        .to_owned(),
    )
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
