mod days;
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
use chrono::{Utc, Weekday};
use days::*;
use exercise::*;
use handlebars::Handlebars;
use history::*;
use notes::*;
use pages::*;
use program::*;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
use tower::ServiceBuilder;
use tower_http::add_extension::AddExtensionLayer;
use weights::*;
use workout::*;

fn make_program() -> pages::AppState {
    let user = match persist::load() {
        Ok(u) => u,
        Err(_) => {
            println!("using default state");
            let mut program = Program::new("Mine".to_owned());
            program.apply(ProgramOp::Add(create_full_body_workout()));
            program.apply(ProgramOp::Add(create_cardio_workout()));

            UserState {
                notes: Notes::new(),
                history: create_history(),
                weights: creat_weight_sets(),
                program,
            }
        }
    };

    AppState {
        handlebars: Handlebars::new(),
        user,
    }
}

fn create_cardio_workout() -> Workout {
    let mut workout = Workout::new("Cardio".to_owned(), Schedule::AnyDay);

    let exercise = DurationsExercise::new(vec![30 * 60]);
    let name = ExerciseName("Elliptical".to_owned());
    let formal_name = FormalName("Elliptical".to_owned());
    let exercise = BuildExercise::durations(name, formal_name, exercise).finalize();
    workout.apply(WorkoutOp::Add(exercise));

    workout
}

fn create_history() -> History {
    fn add(history: &mut History, name: &ExerciseName, reps: Vec<i32>, weight: f32, days_ago: i64) {
        let record = Record {
            program: "Mine".to_owned(),
            workout: "Full Body".to_owned(),
            date: Utc::now() - chrono::Duration::days(days_ago),
            sets: None,
            comment: None,
        };
        history.add(&name, record);
        for rep in reps {
            history.append_reps(&name, rep, Some(weight));
        }
    }

    fn add_squat(history: &mut History) {
        let name = ExerciseName("Squat".to_owned());
        add(history, &name, vec![4, 3, 3], 175.0, 2);
        add(history, &name, vec![5, 5, 5], 175.0, 5);
        add(history, &name, vec![5, 5, 4], 175.0, 8);
        add(history, &name, vec![5, 5, 4], 175.0, 13);
        add(history, &name, vec![5, 4, 4], 175.0, 16);
        add(history, &name, vec![5, 4, 3], 175.0, 19);
    }

    fn add_bench(history: &mut History) {
        let name = ExerciseName("Bench Press".to_owned());
        add(history, &name, vec![4, 4, 3], 150.0, 2);
        add(history, &name, vec![4, 4, 3], 150.0, 5);
        add(history, &name, vec![4, 4, 3], 150.0, 8);
        add(history, &name, vec![4, 3, 3], 150.0, 13);
        add(history, &name, vec![3, 3, 2], 150.0, 16);
        add(history, &name, vec![3, 3, 3], 150.0, 19);
    }

    fn add_rdl(history: &mut History) {
        let name = ExerciseName("RDL".to_owned());
        add(history, &name, vec![8, 8, 8], 155.0, 2);
        add(history, &name, vec![8, 8, 8], 145.0, 5);
        add(history, &name, vec![8, 8, 8], 135.0, 8);
    }

    let mut history = History::new();
    add_squat(&mut history);
    add_bench(&mut history);
    add_rdl(&mut history);
    history
}

fn create_full_body_workout() -> Workout {
    let mut workout = Workout::new("Full Body".to_owned(), Schedule::Every(3));

    // Squat
    let warmups = vec![
        FixedReps::new(5, 0),
        FixedReps::new(5, 50),
        FixedReps::new(3, 70),
        FixedReps::new(1, 90),
    ];
    let worksets = vec![VariableReps::new(3, 5, 100); 3];
    let e = VariableRepsExercise::new(warmups, worksets);
    let name = ExerciseName("Squat".to_owned());
    let formal_name = FormalName("High bar Squat".to_owned());
    let exercise = BuildExercise::variable_reps(name.clone(), formal_name, e)
        .with_weightset("Dual".to_owned())
        .with_weight(175.0)
        .with_rest(210)
        .finalize();
    workout.apply(WorkoutOp::Add(exercise));

    // Dislocates
    let worksets = vec![FixedReps::new(10, 100)];
    let e = FixedRepsExercise::with_percent(vec![], worksets);
    let name = ExerciseName("Dislocates".to_owned());
    let formal_name = FormalName("Shoulder Dislocate (band)".to_owned());
    let exercise = BuildExercise::fixed_reps(name.clone(), formal_name, e).finalize();
    workout.apply(WorkoutOp::Add(exercise));

    // Bench Press
    let warmups = vec![
        FixedReps::new(5, 50),
        FixedReps::new(5, 70),
        FixedReps::new(3, 80),
        FixedReps::new(1, 90),
    ];
    let worksets = vec![VariableReps::new(3, 5, 100); 3];
    let e = VariableRepsExercise::new(warmups, worksets);
    let name = ExerciseName("Bench Press".to_owned());
    let formal_name = FormalName("Bench Press".to_owned());
    let exercise = BuildExercise::variable_reps(name.clone(), formal_name, e)
        .with_weightset("Dual".to_owned())
        .with_weight(150.0)
        .with_rest(210)
        .finalize();
    workout.apply(WorkoutOp::Add(exercise));

    // Quad Stretch
    let e = DurationsExercise::new(vec![20; 4]);
    let name = ExerciseName("Quad Stretch".to_owned());
    let formal_name = FormalName("Standing Quad Stretch".to_owned());
    let exercise = BuildExercise::durations(name, formal_name, e).finalize();
    workout.apply(WorkoutOp::Add(exercise));

    // Cable Abduction
    let warmups = vec![FixedReps::new(6, 75)];
    let worksets = vec![VariableReps::new(6, 12, 100); 3];
    let e = VariableRepsExercise::new(warmups, worksets);
    let name = ExerciseName("Cable Abduction".to_owned());
    let formal_name = FormalName("Cable Hip Abduction".to_owned());
    let exercise = BuildExercise::variable_reps(name.clone(), formal_name, e)
        .with_weightset("Cable Machine".to_owned())
        .with_weight(12.5)
        .with_rest(120)
        .finalize();
    workout.apply(WorkoutOp::Add(exercise));

    // RDL
    let warmups = vec![
        FixedReps::new(5, 60),
        FixedReps::new(3, 80),
        FixedReps::new(1, 90),
    ];
    let worksets = vec![VariableReps::new(4, 8, 100); 3];
    let e = VariableRepsExercise::new(warmups, worksets);
    let name = ExerciseName("RDL".to_owned());
    let formal_name = FormalName("Romanian Deadlift".to_owned());
    let exercise = BuildExercise::variable_reps(name.clone(), formal_name, e)
        .with_weightset("Deadlift".to_owned())
        .with_weight(165.0)
        .with_rest(180)
        .with_last_rest(0)
        .finalize();
    workout.apply(WorkoutOp::Add(exercise));

    workout
}

fn creat_weight_sets() -> Weights {
    let mut weights = Weights::new();

    let set = WeightSet::DualPlates(
        vec![
            Plate::new(5.0, 4),
            Plate::new(10.0, 4),
            Plate::new(25.0, 4),
            Plate::new(35.0, 4),
            Plate::new(45.0, 4),
        ],
        Some(45.0),
    );
    weights.add("Deadlift".to_owned(), set);

    let set = WeightSet::DualPlates(
        vec![
            Plate::new(2.5, 4),
            Plate::new(5.0, 4),
            Plate::new(10.0, 4),
            Plate::new(25.0, 4),
            Plate::new(45.0, 4),
        ],
        Some(45.0),
    );
    weights.add("Dual".to_owned(), set);

    let set = WeightSet::DualPlates(
        vec![
            Plate::new(5.0, 4),
            Plate::new(10.0, 4),
            Plate::new(25.0, 8),
            Plate::new(35.0, 6),
            Plate::new(45.0, 8),
        ],
        None,
    );
    weights.add("Single".to_owned(), set);

    let set = WeightSet::Discrete((25..=975).step_by(50).map(|i| (i as f32) / 10.0).collect());
    weights.add("Cable Machine".to_owned(), set);

    let mut w1: Vec<_> = (125..=500).step_by(75).map(|i| (i as f32) / 10.0).collect();
    let w2: Vec<_> = (550..=1300)
        .step_by(50)
        .map(|i| (i as f32) / 10.0)
        .collect();
    w1.extend(w2);
    let set = WeightSet::Discrete(w1);
    weights.add("Lat Pulldown".to_owned(), set);

    let set = WeightSet::Discrete((5..=100).step_by(5).map(|i| i as f32).collect());
    weights.add("Gym Dumbbells".to_owned(), set);

    let set = WeightSet::Discrete(vec![9.0, 13.0, 18.0]);
    weights.add("Kettlebell".to_owned(), set);

    weights
}

#[tokio::main]
async fn main() {
    let state = make_program();

    println!("{:?}", dirs::data_dir());

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
