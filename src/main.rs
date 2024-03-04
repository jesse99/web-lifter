use axum::{
    extract::Extension,
    http::{header, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use chrono::Weekday;
use handlebars::Handlebars;
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
    engine: Handlebars<'static>,
    program: Program,
    // exercises: Exercises,
}

type SharedState = Arc<RwLock<State>>;

fn make_program() -> State {
    let mut workout1 = Workout::new("Full Body".to_owned(), Schedule::Every(2));
    let workout2 = Workout::new("Cardio".to_owned(), Schedule::AnyDay);
    let workout3 = Workout::new(
        "Strong Lifts".to_owned(),
        Schedule::Days(vec![Weekday::Mon, Weekday::Wed, Weekday::Fri]),
    );

    let name1 = ExerciseName("Quad Stretch".to_owned());
    workout1.apply(WorkoutOp::Add(name1.clone()));

    let name2 = ExerciseName("Side Leg Lift".to_owned());
    workout1.apply(WorkoutOp::Add(name2.clone()));

    let mut program = Program::new("My".to_owned());
    program.apply(ProgramOp::Add(workout1));
    program.apply(ProgramOp::Add(workout2));
    program.apply(ProgramOp::Add(workout3));

    let mut exercises = Exercises::new();
    exercises.apply(ExercisesOp::Add(
        name1,
        Exercise::new("Standing Quad Stretch".to_owned()),
    ));
    exercises.apply(ExercisesOp::Add(
        name2,
        Exercise::new("Side Lying Abduction".to_owned()),
    ));

    State {
        engine: Handlebars::new(),
        program,
        // exercises,
    }
}

#[tokio::main]
async fn main() {
    let state = make_program();

    tracing_subscriber::fmt::init();

    // build our application with a route
    let app = Router::new()
        .route("/", get(get_program))
        .route("/styles/style.css", get(get_styles))
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

async fn get_styles(Extension(_state): Extension<SharedState>) -> impl IntoResponse {
    let contents = include_str!("../files/styles.css");
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/css")],
        contents,
    )
}

impl Status {
    fn to_label(&self) -> String {
        match self {
            Status::Completed => "completed".to_owned(),
            Status::Due(0) => "today".to_owned(),
            Status::Due(1) => "tomorrow".to_owned(),
            Status::Due(n) => format!("in {n} days"),
            Status::DueAnyTime => "any day".to_owned(),
            Status::Empty => "no workouts".to_owned(),
            Status::Overdue(1) => "overdue by 1 day".to_owned(),
            Status::Overdue(n) => format!("overdue by {n} days"),
            Status::PartiallyCompleted => "partially completed".to_owned(),
        }
    }

    fn to_class(&self) -> &str {
        match self {
            Status::Completed => "completed",
            Status::Due(0) => "due_today",
            Status::Due(1) => "tomorrow",
            Status::Due(_) => "due_later",
            Status::DueAnyTime => "any_time",
            Status::Empty => "empty",
            Status::Overdue(_) => "overdue",
            Status::PartiallyCompleted => "partial",
        }
    }
}

#[derive(Serialize, Deserialize)]
struct WorkoutData {
    name: String,
    status_label: String,
    status_class: String,
}

impl WorkoutData {
    fn new(workout: &Workout) -> WorkoutData {
        let status = workout.status();
        WorkoutData {
            name: workout.name.clone(),
            status_class: status.to_class().to_owned(),
            status_label: status.to_label(),
        }
    }
}

#[derive(Serialize, Deserialize)]
struct ProgramData {
    name: String,
    workouts: Vec<WorkoutData>,
}

impl ProgramData {
    fn new(program: &Program) -> ProgramData {
        let workouts = program.workouts().map(|w| WorkoutData::new(w)).collect();
        ProgramData {
            name: program.name.clone(),
            workouts,
        }
    }
}

async fn get_program(Extension(state): Extension<SharedState>) -> impl IntoResponse {
    let engine = &state.read().unwrap().engine;
    let program = &state.read().unwrap().program;

    // Note that MDN recommends against using aria tables, see https://developer.mozilla.org/en-US/docs/Web/Accessibility/ARIA/Roles/table_role
    let template = include_str!("../files/program.html");
    let data = ProgramData::new(program);
    let result = engine.render_template(template, &data).unwrap();

    axum::response::Html(result)
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
