mod days;
mod exercise;
mod history;
mod pages;
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
use chrono::{Utc, Weekday};
use days::*;
use exercise::*;
use handlebars::Handlebars;
use history::*;
use pages::*;
use program::*;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
use tower::ServiceBuilder;
use tower_http::add_extension::AddExtensionLayer;
use weights::*;
use workout::*;

fn make_program() -> pages::AppState {
    // exercises
    let exercise = DurationsExercise::new(vec![20; 4]);
    let name = ExerciseName("Quad Stretch".to_owned());
    let formal_name = FormalName("Standing Quad Stretch".to_owned());
    let exercise1 = SetsExercise::durations(name, formal_name, exercise)
        .with_weight(10.0)
        .with_rest(20)
        .finalize();

    let warmups = vec![FixedReps::new(5, 80), FixedReps::new(3, 90)];
    let worksets = vec![
        FixedReps::new(8, 100),
        FixedReps::new(8, 100),
        FixedReps::new(8, 100),
    ];
    let exercise = FixedRepsExercise::with_percent(warmups, worksets);
    let name = ExerciseName("Side Leg Lift".to_owned());
    let formal_name = FormalName("Side Lying Abduction".to_owned());
    let exercise2 = SetsExercise::fixed_reps(name.clone(), formal_name, exercise)
        .with_weight(12.0)
        .with_rest(20)
        .finalize();

    let warmups = vec![FixedReps::new(5, 80), FixedReps::new(3, 90)];
    let worksets = vec![VariableReps::new(4, 8, 100); 3];
    let exercise = VariableRepsExercise::new(warmups, worksets);
    let name = ExerciseName("Squat".to_owned());
    let formal_name = FormalName("Low bar Squat".to_owned());
    let exercise3 = SetsExercise::variable_reps(name.clone(), formal_name, exercise)
        .with_weightset("barbell".to_owned())
        .with_weight(135.0)
        // .with_rest(20)
        .finalize();

    // workouts
    let mut workout1 = Workout::new("Full Body".to_owned(), Schedule::Every(2));
    workout1.apply(WorkoutOp::Add(exercise1));
    workout1.apply(WorkoutOp::Add(exercise2));
    workout1.apply(WorkoutOp::Add(exercise3));

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
        sets: None,
        comment: None,
    };
    history.add(&name, record);
    history.append_reps(&name, 3, None);
    history.append_reps(&name, 3, None);

    let record = Record {
        program: program.name.clone(),
        workout: "Full Body".to_owned(),
        date: Utc::now() - chrono::Duration::days(9),
        sets: None,
        comment: None,
    };
    history.add(&name, record);
    history.append_reps(&name, 5, None);
    history.append_reps(&name, 5, None);

    let record = Record {
        program: program.name.clone(),
        workout: "Full Body".to_owned(),
        date: Utc::now() - chrono::Duration::days(6),
        sets: None,
        comment: None,
    };
    history.add(&name, record);
    history.append_reps(&name, 5, None);
    history.append_reps(&name, 4, None);

    let record = Record {
        program: program.name.clone(),
        workout: "Full Body".to_owned(),
        date: Utc::now() - chrono::Duration::days(3),
        sets: None,
        comment: None,
    };
    history.add(&name, record);
    history.append_reps(&name, 10, None);
    history.append_reps(&name, 10, None);

    let mut weights = Weights::new();
    let set = WeightSet::DualPlates(
        vec![
            Plate::new(5.0, 6),
            Plate::new(10.0, 6),
            Plate::new(25.0, 4),
            Plate::new(45.0, 4),
        ],
        Some(45.0),
    );
    weights.add("barbell".to_owned(), set);

    AppState {
        engine: Handlebars::new(),
        history,
        weights,
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
    let contents = get_workout_page(state, &name)?;
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
