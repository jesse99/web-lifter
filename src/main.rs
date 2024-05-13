mod app_state;
mod days;
mod default;
mod errors;
mod exercise;
mod history;
mod notes;
mod pages;
mod persist;
mod program;
mod weights;
mod workout;

use app_state::SharedState;
use axum::{
    extract::{Extension, Path, Query},
    http::{header, HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Form, Router,
};
use errors::{Error, Unwrapper};
use handlebars::Handlebars;
use serde::Deserialize;
use std::sync::RwLock;
use tower::ServiceBuilder;
use tower_http::add_extension::AddExtensionLayer;

use crate::exercise::{
    BuildExercise, DurationsExercise, Exercise, ExerciseName, FixedReps, FixedRepsExercise,
    FormalName, VariableReps, VariableRepsExercise, VariableSetsExercise,
};

#[tokio::main]
async fn main() {
    let state = default::make_program();

    tracing_subscriber::fmt::init();
    let app = Router::new()
        // data --------------------------------------------------------------------------
        .route(
            "/scripts/discrete.js",
            get(|s| get_js(s, include_str!("../files/discrete.js"))),
        )
        .route(
            "/scripts/exercise.js",
            get(|s| get_js(s, include_str!("../files/exercise.js"))),
        )
        .route(
            "/scripts/editable-list.js",
            get(|s| get_js(s, include_str!("../files/editable-list.js"))),
        )
        .route(
            "/scripts/formal_name.js",
            get(|s| get_js(s, include_str!("../files/formal_name.js"))),
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
        // get ---------------------------------------------------------------------------
        .route("/", get(get_program))
        .route("/show-overview", get(get_overview))
        .route("/add-workout", get(get_edit_add_workout))
        .route("/edit-discrete-weights", get(get_discrete_weights))
        .route("/edit-plate-weights", get(get_plate_weights))
        .route("/edit-blocks", get(get_blocks))
        .route("/edit-block/:name", get(get_edit_block))
        .route("/edit-week", get(get_edit_set_week))
        .route("/edit-program-name", get(get_edit_program_name))
        .route("/edit-workouts", get(get_edit_edit_workouts))
        .route("/workout/:name", get(get_workout))
        .route("/schedule-daily/:workout", get(get_schedule_daily))
        .route("/edit-schedule-nth/:workout", get(get_schedule_nth))
        .route("/edit-schedule-weekday/:workout", get(get_schedule_weekday))
        .route("/exercise/:workout/:exercise", get(get_exercise))
        .route("/add-exercise/:workout", get(get_add_exercise))
        .route("/edit-workout-name/:workout", get(get_edit_workout_name))
        .route("/edit-exercises/:workout", get(get_edit_exercises))
        .route("/edit-name/:workout/:exercise", get(get_edit_exercise_name))
        .route(
            "/edit-formal-name/:workout/:exercise",
            get(get_edit_formal_name),
        )
        .route("/edit-weight/:workout/:exercise", get(get_edit_weight))
        .route(
            "/edit-any-weight/:workout/:exercise",
            get(get_edit_any_weight),
        )
        .route(
            "/edit-discrete-weight/:workout/:exercise",
            get(get_edit_discrete_set),
        )
        .route(
            "/edit-plates-weight/:workout/:exercise",
            get(get_edit_plate_set),
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
        .route("/edit-current-set/:workout/:exercise", get(get_current_set))
        // post --------------------------------------------------------------------------
        .route("/set-program-name", post(post_set_program_name))
        .route("/set-week", post(post_set_week))
        .route("/set-discrete-weights", post(post_set_discrete_weights))
        .route("/set-plate-weights", post(post_set_plate_weights))
        .route("/set-blocks", post(post_set_blocks))
        .route("/set-block/:name", post(post_set_block))
        .route("/set-workouts", post(post_set_workouts))
        .route("/set-add-workout", post(post_set_add_workout))
        .route("/exercise/:workout/:exercise/next-set", post(post_next_set))
        .route(
            "/exercise/:workout/:exercise/next-var-set",
            post(post_next_var_set),
        )
        .route("/set-workout-name/:workout", post(post_set_workout_name))
        .route("/set-schedule-nth/:workout", post(post_set_schedule_nth))
        .route(
            "/set-schedule-weekdays/:workout",
            post(post_set_schedule_weekdays),
        )
        .route("/reset/exercise/:workout/:exercise", post(reset_exercise))
        .route("/append-exercise/:workout", post(post_append_exercise))
        .route("/set-exercises/:workout", post(post_set_exercises))
        .route(
            "/set-exercise-name/:workout/:exercise",
            post(post_set_exercise_name),
        )
        .route(
            "/set-formal-name/:workout/:exercise",
            post(post_set_formal_name),
        )
        .route("/set-weight/:workout/:exercise", post(post_set_weight))
        .route(
            "/set-discrete-set/:workout/:exercise",
            post(post_set_discrete_set),
        )
        .route(
            "/set-plates-set/:workout/:exercise",
            post(post_set_plates_set),
        )
        .route("/revert-note/:workout/:exercise", post(post_revert_note))
        .route("/set-note/:workout/:exercise", post(post_set_note))
        .route(
            "/set-current-set/:workout/:exercise",
            post(post_set_current_set),
        )
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

async fn get_program(Extension(state): Extension<SharedState>) -> Result<impl IntoResponse, Error> {
    let contents = pages::get_program_page(state)?;
    Ok((
        [
            ("Cache-Control", "no-store, must-revalidate"),
            ("Expires", "0"),
        ],
        axum::response::Html(contents),
    ))
}

async fn get_edit_program_name(
    Extension(state): Extension<SharedState>,
) -> Result<impl IntoResponse, Error> {
    let post_url = "/set-program-name";
    let cancel_url = "/";
    let help = "Must be unique within the programs.";
    let program = &state.read().unwrap().user.program;
    let contents = pages::get_edit_name(&program.name, help, &post_url, &cancel_url);
    Ok((
        [
            ("Cache-Control", "no-store, must-revalidate"),
            ("Expires", "0"),
        ],
        axum::response::Html(contents),
    ))
}

async fn get_overview(
    Extension(state): Extension<SharedState>,
) -> Result<impl IntoResponse, Error> {
    let contents = pages::get_overview_page(state)?;
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
) -> Result<impl IntoResponse, Error> {
    let contents = pages::get_workout_page(state, &name)?;
    Ok((
        [
            ("Cache-Control", "no-store, must-revalidate"),
            ("Expires", "0"),
        ],
        axum::response::Html(contents),
    ))
}

async fn get_schedule_daily(
    Path(workout): Path<String>,
    Extension(state): Extension<SharedState>,
) -> Result<impl IntoResponse, Error> {
    let new_url = pages::get_schedule_daily(state, &workout);

    let mut headers = HeaderMap::new();
    headers.insert(
        "Cache-Control",
        "no-store, must-revalidate".parse().unwrap(),
    );
    headers.insert("Expires", "0".parse().unwrap());
    headers.insert("Location", new_url.parse().unwrap());
    Ok((StatusCode::SEE_OTHER, headers))
}

async fn get_schedule_nth(
    Path(workout): Path<String>,
    Extension(state): Extension<SharedState>,
) -> Result<impl IntoResponse, Error> {
    let contents = pages::get_edit_schedule_nth(state, &workout);
    Ok((
        [
            ("Cache-Control", "no-store, must-revalidate"),
            ("Expires", "0"),
        ],
        axum::response::Html(contents),
    ))
}

async fn get_schedule_weekday(
    Path(workout): Path<String>,
    Extension(state): Extension<SharedState>,
) -> Result<impl IntoResponse, Error> {
    let contents = pages::get_edit_schedule_weekdays(state, &workout);
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
) -> Result<impl IntoResponse, Error> {
    let contents = pages::get_exercise_page(state, &workout, &exercise)?;
    Ok((
        [
            ("Cache-Control", "no-store, must-revalidate"),
            ("Expires", "0"),
        ],
        axum::response::Html(contents),
    ))
}

async fn get_add_exercise(
    Path(workout): Path<String>,
    Extension(_state): Extension<SharedState>,
) -> Result<impl IntoResponse, Error> {
    let contents = pages::get_add_exercise(&workout);
    Ok((
        [
            ("Cache-Control", "no-store, must-revalidate"),
            ("Expires", "0"),
        ],
        axum::response::Html(contents),
    ))
}

async fn get_edit_exercises(
    Path(workout): Path<String>,
    Extension(state): Extension<SharedState>,
) -> Result<impl IntoResponse, Error> {
    let contents = pages::get_edit_exercises(state, &workout);
    Ok((
        [
            ("Cache-Control", "no-store, must-revalidate"),
            ("Expires", "0"),
        ],
        axum::response::Html(contents),
    ))
}

async fn get_edit_edit_workouts(
    Extension(state): Extension<SharedState>,
) -> Result<impl IntoResponse, Error> {
    let contents = pages::get_edit_workouts(state);
    Ok((
        [
            ("Cache-Control", "no-store, must-revalidate"),
            ("Expires", "0"),
        ],
        axum::response::Html(contents),
    ))
}

async fn get_discrete_weights(
    Extension(state): Extension<SharedState>,
) -> Result<impl IntoResponse, Error> {
    let contents = pages::get_discrete_weights(state);
    Ok((
        [
            ("Cache-Control", "no-store, must-revalidate"),
            ("Expires", "0"),
        ],
        axum::response::Html(contents),
    ))
}

async fn get_plate_weights(
    Extension(state): Extension<SharedState>,
) -> Result<impl IntoResponse, Error> {
    let contents = pages::get_plate_weights(state);
    Ok((
        [
            ("Cache-Control", "no-store, must-revalidate"),
            ("Expires", "0"),
        ],
        axum::response::Html(contents),
    ))
}

async fn get_blocks(Extension(state): Extension<SharedState>) -> Result<impl IntoResponse, Error> {
    let contents = pages::get_blocks(state);
    Ok((
        [
            ("Cache-Control", "no-store, must-revalidate"),
            ("Expires", "0"),
        ],
        axum::response::Html(contents),
    ))
}

async fn get_edit_block(
    Path(block_name): Path<String>,
    Extension(state): Extension<SharedState>,
) -> Result<impl IntoResponse, Error> {
    let contents = pages::get_edit_block(state, &block_name);
    Ok((
        [
            ("Cache-Control", "no-store, must-revalidate"),
            ("Expires", "0"),
        ],
        axum::response::Html(contents),
    ))
}

async fn get_edit_set_week(
    Extension(state): Extension<SharedState>,
) -> Result<impl IntoResponse, Error> {
    let contents = pages::get_edit_set_week(state);
    Ok((
        [
            ("Cache-Control", "no-store, must-revalidate"),
            ("Expires", "0"),
        ],
        axum::response::Html(contents),
    ))
}

async fn get_edit_add_workout(
    Extension(_state): Extension<SharedState>,
) -> Result<impl IntoResponse, Error> {
    let post_url = format!("/set-add-workout");
    let cancel_url = "/";
    let help = "Must be unique within the program.";
    let contents = pages::get_edit_name("", help, &post_url, &cancel_url);
    Ok((
        [
            ("Cache-Control", "no-store, must-revalidate"),
            ("Expires", "0"),
        ],
        axum::response::Html(contents),
    ))
}

async fn get_edit_workout_name(
    Path(workout): Path<String>,
    Extension(_state): Extension<SharedState>,
) -> Result<impl IntoResponse, Error> {
    let post_url = format!("/set-workout-name/{}", workout);
    let cancel_url = format!("/workout/{}", workout);
    let help = "Must be unique within the program.";
    let contents = pages::get_edit_name(&workout, help, &post_url, &cancel_url);
    Ok((
        [
            ("Cache-Control", "no-store, must-revalidate"),
            ("Expires", "0"),
        ],
        axum::response::Html(contents),
    ))
}

async fn get_edit_exercise_name(
    Path((workout, exercise)): Path<(String, String)>,
    Extension(_state): Extension<SharedState>,
) -> Result<impl IntoResponse, Error> {
    let post_url = format!("/set-exercise-name/{}/{}", workout, exercise);
    let cancel_url = format!("/exercise/{}/{}", workout, exercise);
    let value = exercise.to_owned();
    let help = "Must be unique within the workout.";
    let contents = pages::get_edit_name(&value, help, &post_url, &cancel_url);
    Ok((
        [
            ("Cache-Control", "no-store, must-revalidate"),
            ("Expires", "0"),
        ],
        axum::response::Html(contents),
    ))
}

async fn get_edit_formal_name(
    Path((workout, exercise)): Path<(String, String)>,
    Extension(state): Extension<SharedState>,
) -> Result<impl IntoResponse, Error> {
    let contents = pages::get_formal_name(state, &workout, &exercise);
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
) -> Result<impl IntoResponse, Error> {
    let contents = pages::get_edit_weight(state, &workout, &exercise);
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
) -> Result<impl IntoResponse, Error> {
    let contents = pages::get_edit_durations(state, &workout, &exercise);
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
) -> Result<impl IntoResponse, Error> {
    let contents = pages::get_edit_fixed_reps(state, &workout, &exercise);
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
) -> Result<impl IntoResponse, Error> {
    let contents = pages::get_edit_var_reps(state, &workout, &exercise);
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
) -> Result<impl IntoResponse, Error> {
    let contents = pages::get_edit_var_sets(state, &workout, &exercise);
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
) -> Result<impl IntoResponse, Error> {
    let contents = pages::get_edit_note(state, &workout, &exercise);
    Ok((
        [
            ("Cache-Control", "no-store, must-revalidate"),
            ("Expires", "0"),
        ],
        axum::response::Html(contents),
    ))
}

async fn get_current_set(
    Path((workout, exercise)): Path<(String, String)>,
    Extension(state): Extension<SharedState>,
) -> Result<impl IntoResponse, Error> {
    let contents = pages::get_current_set(state, &workout, &exercise);
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
) -> Result<impl IntoResponse, Error> {
    let contents = pages::get_edit_rest(state, &workout, &exercise);
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
) -> Result<impl IntoResponse, Error> {
    let id: u64 = id
        .parse()
        .unwrap_or_err(&format!("expected int for id but found '{id}'"))?;
    let contents = pages::get_edit_durs_record(state, &workout, &exercise, id)?;
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
) -> Result<impl IntoResponse, Error> {
    let id: u64 = id
        .parse()
        .unwrap_or_err(&format!("expected int for id but found '{id}'"))?;
    let contents = pages::get_edit_reps_record(state, &workout, &exercise, id)?;
    Ok((
        [
            ("Cache-Control", "no-store, must-revalidate"),
            ("Expires", "0"),
        ],
        axum::response::Html(contents),
    ))
}

async fn get_edit_discrete_set(
    Path((workout, exercise)): Path<(String, String)>,
    Extension(state): Extension<SharedState>,
) -> Result<impl IntoResponse, Error> {
    let contents = pages::get_edit_discrete_set(state, &workout, &exercise);
    Ok((
        [
            ("Cache-Control", "no-store, must-revalidate"),
            ("Expires", "0"),
        ],
        axum::response::Html(contents),
    ))
}

async fn get_edit_plate_set(
    Path((workout, exercise)): Path<(String, String)>,
    Extension(state): Extension<SharedState>,
) -> Result<impl IntoResponse, Error> {
    let contents = pages::get_edit_plate_set(state, &workout, &exercise);
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
) -> Result<impl IntoResponse, Error> {
    let contents = pages::get_edit_any_weight(state, &workout, &exercise);
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
) -> Result<impl IntoResponse, Error> {
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
) -> Result<impl IntoResponse, Error> {
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
) -> Result<impl IntoResponse, Error> {
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
struct AppendExercise {
    name: String,  // exercise name
    types: String, // "durations", "fixed", "var-reps", or "var-sets"
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
async fn post_append_exercise(
    Path(workout): Path<String>,
    Extension(state): Extension<SharedState>,
    Form(payload): Form<AppendExercise>,
) -> Result<impl IntoResponse, Error> {
    fn default_durations(name: &str) -> Exercise {
        let e = DurationsExercise::new(vec![30; 3]);
        let name = ExerciseName(name.to_owned());
        let formal_name = FormalName("".to_owned());
        BuildExercise::durations(name, formal_name, e).finalize()
    }

    fn default_fixed(name: &str) -> Exercise {
        let sets = vec![5; 3];
        let e = FixedRepsExercise::with_reps(sets);
        let name = ExerciseName(name.to_owned());
        let formal_name = FormalName("".to_owned());
        BuildExercise::fixed_reps(name, formal_name, e)
            .with_rest_mins(2.0)
            .finalize()
    }

    fn default_var_reps(name: &str) -> Exercise {
        let warmups = vec![
            FixedReps::new(5, 70),
            FixedReps::new(3, 80),
            FixedReps::new(1, 90),
        ];
        let worksets = vec![VariableReps::new(3, 6, 100); 3];
        let e = VariableRepsExercise::new(warmups, worksets);
        let name = ExerciseName(name.to_owned());
        let formal_name = FormalName("".to_owned());
        BuildExercise::variable_reps(name.clone(), formal_name, e)
            .with_rest_mins(2.5)
            .finalize()
    }

    fn default_var_sets(name: &str) -> Exercise {
        let e = VariableSetsExercise::new(15);
        let name = ExerciseName(name.to_owned());
        let formal_name = FormalName("".to_owned());
        BuildExercise::variable_sets(name.clone(), formal_name, e)
            .with_rest_mins(2.5)
            .finalize()
    }

    let exercise = match payload.types.as_ref() {
        "durations" => default_durations(&payload.name),
        "fixed" => default_fixed(&payload.name),
        "var-reps" => default_var_reps(&payload.name),
        "var-sets" => default_var_sets(&payload.name),
        _ => return validation_err!("bad exercise type"),
    };
    let new_url = pages::post_append_exercise(state, &workout, exercise)?;

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
struct EditWeights {
    sets: String, // "Block 1¦Block 2"
}

async fn post_set_discrete_weights(
    Extension(state): Extension<SharedState>,
    Form(payload): Form<EditWeights>,
) -> Result<impl IntoResponse, Error> {
    let sets: Vec<_> = payload.sets.split("¦").map(|s| s.to_string()).collect();
    let new_url = pages::post_set_discrete_weights(state, sets)?;

    let mut headers = HeaderMap::new();
    headers.insert(
        "Cache-Control",
        "no-store, must-revalidate".parse().unwrap(),
    );
    headers.insert("Expires", "0".parse().unwrap());
    headers.insert("Location", new_url.path().parse().unwrap());
    Ok((StatusCode::SEE_OTHER, headers))
}

async fn post_set_plate_weights(
    Extension(state): Extension<SharedState>,
    Form(payload): Form<EditWeights>,
) -> Result<impl IntoResponse, Error> {
    let sets: Vec<_> = payload.sets.split("¦").map(|s| s.to_string()).collect();
    let new_url = pages::post_set_plate_weights(state, sets)?;

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
struct EditBlocks {
    blocks: String, // "Block 1¦Block 2"
}

async fn post_set_blocks(
    Extension(state): Extension<SharedState>,
    Form(payload): Form<EditBlocks>,
) -> Result<impl IntoResponse, Error> {
    let blocks: Vec<_> = payload.blocks.split("¦").map(|s| s.to_string()).collect();
    let new_url = pages::post_set_blocks(state, blocks)?;

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
struct EditBlock {
    name: String,
    num_weeks: String,
    workouts: String,
}

async fn post_set_block(
    Path(old_name): Path<String>,
    Extension(state): Extension<SharedState>,
    Form(payload): Form<EditBlock>,
) -> Result<impl IntoResponse, Error> {
    let num_weeks: i32 = payload.num_weeks.parse().unwrap_or_err(&format!(
        "expected int for num_weeks but found '{}'",
        payload.num_weeks
    ))?;
    let workouts = payload
        .workouts
        .trim()
        .split("¦")
        .map(|s| s.to_string())
        .filter(|s| !s.is_empty())
        .collect();
    let new_url = pages::post_set_block(state, &old_name, &payload.name, num_weeks, workouts)?;

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
struct EditableList {
    names: String,    // "Name 1\tName 2"
    disabled: String, // "true\tfalse"
}

async fn post_set_workouts(
    Extension(state): Extension<SharedState>,
    Form(payload): Form<EditableList>,
) -> Result<impl IntoResponse, Error> {
    let workouts: Vec<_> = payload.names.split("\t").collect();
    let disabled = payload
        .disabled
        .split("\t") // TODO probably should use '¦'
        .map(|s| s.parse())
        .collect::<Result<Vec<_>, _>>()
        .unwrap_or_err("bad disabled list")?;
    let new_url = pages::post_set_workouts(state, workouts, disabled)?;

    let mut headers = HeaderMap::new();
    headers.insert(
        "Cache-Control",
        "no-store, must-revalidate".parse().unwrap(),
    );
    headers.insert("Expires", "0".parse().unwrap());
    headers.insert("Location", new_url.path().parse().unwrap());
    Ok((StatusCode::SEE_OTHER, headers))
}

async fn post_set_exercises(
    Path(workout): Path<String>,
    Extension(state): Extension<SharedState>,
    Form(payload): Form<EditableList>,
) -> Result<impl IntoResponse, Error> {
    let exercises: Vec<_> = payload.names.split("\t").collect();
    let disabled = payload
        .disabled
        .split("\t") // TODO probably should use '¦'
        .map(|s| s.parse())
        .collect::<Result<Vec<_>, _>>()
        .unwrap_or_err("bad disabled list")?;
    let new_url = pages::post_set_exercises(state, &workout, exercises, disabled)?;

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
struct SetName {
    name: String,
}

async fn post_set_add_workout(
    Extension(state): Extension<SharedState>,
    Form(payload): Form<SetName>,
) -> Result<impl IntoResponse, Error> {
    let new_url = pages::post_set_add_workout(state, &payload.name)?;

    let mut headers = HeaderMap::new();
    headers.insert(
        "Cache-Control",
        "no-store, must-revalidate".parse().unwrap(),
    );
    headers.insert("Expires", "0".parse().unwrap());
    headers.insert("Location", new_url.path().parse().unwrap());
    Ok((StatusCode::SEE_OTHER, headers))
}

async fn post_set_program_name(
    Extension(state): Extension<SharedState>,
    Form(payload): Form<SetName>,
) -> Result<impl IntoResponse, Error> {
    let new_url = pages::post_set_program_name(state, &payload.name)?;

    let mut headers = HeaderMap::new();
    headers.insert(
        "Cache-Control",
        "no-store, must-revalidate".parse().unwrap(),
    );
    headers.insert("Expires", "0".parse().unwrap());
    headers.insert("Location", new_url.path().parse().unwrap());
    Ok((StatusCode::SEE_OTHER, headers))
}

async fn post_set_workout_name(
    Path(workout): Path<String>,
    Extension(state): Extension<SharedState>,
    Form(payload): Form<SetName>,
) -> Result<impl IntoResponse, Error> {
    let new_url = pages::post_set_workout_name(state, &workout, &payload.name)?;

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
struct SetWeek {
    week: String,
}

async fn post_set_week(
    Extension(state): Extension<SharedState>,
    Form(payload): Form<SetWeek>,
) -> Result<impl IntoResponse, Error> {
    let week: i32 = payload
        .week
        .parse()
        .unwrap_or_err(&format!("expected int but found '{}'", payload.week))?;
    let new_url = pages::post_set_week(state, week)?;

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
struct SetNth {
    n: String,
}

async fn post_set_schedule_nth(
    Path(workout): Path<String>,
    Extension(state): Extension<SharedState>,
    Form(payload): Form<SetNth>,
) -> Result<impl IntoResponse, Error> {
    let n: i32 = payload
        .n
        .parse()
        .unwrap_or_err(&format!("expected int but found '{}'", payload.n))?;
    let new_url = pages::post_schedule_nth(state, &workout, n)?;

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
struct SetDays {
    days: String, // "sun mon tues"
}

async fn post_set_schedule_weekdays(
    Path(workout): Path<String>,
    Extension(state): Extension<SharedState>,
    Form(payload): Form<SetDays>,
) -> Result<impl IntoResponse, Error> {
    let days = payload
        .days
        .trim()
        .split("¦")
        .map(|s| s.to_string())
        .collect();
    let new_url = pages::post_set_schedule_weekdays(state, &workout, days)?;

    let mut headers = HeaderMap::new();
    headers.insert(
        "Cache-Control",
        "no-store, must-revalidate".parse().unwrap(),
    );
    headers.insert("Expires", "0".parse().unwrap());
    headers.insert("Location", new_url.path().parse().unwrap());
    Ok((StatusCode::SEE_OTHER, headers))
}

async fn post_set_exercise_name(
    Path((workout, exercise)): Path<(String, String)>,
    Extension(state): Extension<SharedState>,
    Form(payload): Form<SetName>,
) -> Result<impl IntoResponse, Error> {
    let new_url = pages::post_set_exercise_name(state, &workout, &exercise, &payload.name)?;

    let mut headers = HeaderMap::new();
    headers.insert(
        "Cache-Control",
        "no-store, must-revalidate".parse().unwrap(),
    );
    headers.insert("Expires", "0".parse().unwrap());
    headers.insert("Location", new_url.path().parse().unwrap());
    Ok((StatusCode::SEE_OTHER, headers))
}

async fn post_set_formal_name(
    Path((workout, exercise)): Path<(String, String)>,
    Extension(state): Extension<SharedState>,
    Form(payload): Form<SetName>,
) -> Result<impl IntoResponse, Error> {
    let new_url = pages::post_set_formal_name(state, &workout, &exercise, &payload.name)?;

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

async fn post_set_weight(
    Path((workout, exercise)): Path<(String, String)>,
    Extension(state): Extension<SharedState>,
    Form(payload): Form<SetWeight>,
) -> Result<impl IntoResponse, Error> {
    let parts: Vec<_> = payload.weight.split(" ").collect();
    let s = parts.get(0).unwrap_or_err("empty payload")?.to_string();
    let w = if s == "None" {
        None
    } else {
        let x: f32 = s
            .parse()
            .unwrap_or_err(&format!("expected f32 but found '{s}'"))?;
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

#[derive(Debug, Deserialize)]
struct SetDiscreteSet {
    name: String,
    weights: String, // weights like "25.000" separated by "¦"
}

async fn post_set_discrete_set(
    Path((workout, exercise)): Path<(String, String)>,
    Extension(state): Extension<SharedState>,
    Form(payload): Form<SetDiscreteSet>,
) -> Result<impl IntoResponse, Error> {
    let weights = payload
        .weights
        .split("¦")
        .map(|s| s.parse::<f32>())
        .collect::<Result<Vec<_>, _>>()
        .unwrap_or_err("bad weights list")?;
    let new_url = pages::post_set_discrete_set(state, &workout, &exercise, &payload.name, weights)?;

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
struct SetPlateSet {
    name: String,
    weights: String, // weights like "25.000x4" separated by "¦"
    bar: String,
}

async fn post_set_plates_set(
    Path((workout, exercise)): Path<(String, String)>,
    Extension(state): Extension<SharedState>,
    Form(payload): Form<SetPlateSet>,
) -> Result<impl IntoResponse, Error> {
    fn parse_plate(value: &str) -> Result<(f32, i32), Error> {
        let parts: Vec<_> = value.split("x").collect();
        if parts.len() == 2 {
            let weight: f32 = parts[0]
                .parse()
                .unwrap_or_err(&format!("expected float for weight but found '{value}'"))?;
            let count: i32 = parts[1]
                .parse()
                .unwrap_or_err(&format!("expected int for count but found '{value}'"))?;
            return Ok((weight, count));
        } else {
            return validation_err!("Expected weightxcount but found '{value}'");
        }
    }

    let plates = payload
        .weights
        .split("¦")
        .map(|s| parse_plate(s))
        .collect::<Result<Vec<_>, _>>()?;
    let bar = if payload.bar.is_empty() {
        None
    } else {
        Some(payload.bar.parse::<f32>().unwrap_or_err("bad bar")?)
    };
    let new_url =
        pages::post_set_plate_set(state, &workout, &exercise, &payload.name, plates, bar)?;

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
) -> Result<impl IntoResponse, Error> {
    let parts: Vec<_> = payload.weight.split(" ").collect();
    let s = parts.get(0).unwrap_or_err("empty payload")?.to_string();
    let w: f32 = s
        .parse()
        .unwrap_or_err(&format!("expected f32 but found '{s}'"))?;
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
struct SetCurrentSet {
    sets: String,
}

async fn post_set_current_set(
    Path((workout, exercise)): Path<(String, String)>,
    Extension(state): Extension<SharedState>,
    Form(payload): Form<SetCurrentSet>,
) -> Result<impl IntoResponse, Error> {
    let new_url = pages::post_set_current_set(state, &workout, &exercise, payload.sets)?;

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
struct SetNote {
    note: String,
}

async fn post_set_note(
    Path((workout, exercise)): Path<(String, String)>,
    Extension(state): Extension<SharedState>,
    Form(payload): Form<SetNote>,
) -> Result<impl IntoResponse, Error> {
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
) -> Result<impl IntoResponse, Error> {
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
    times: String,
    target: String,
    units: String, // "secs", "mins", or "hours"
}

async fn post_set_durations(
    Path((workout, exercise)): Path<(String, String)>,
    Extension(state): Extension<SharedState>,
    Form(payload): Form<SetDurations>,
) -> Result<impl IntoResponse, Error> {
    let durations = payload
        .times
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
) -> Result<impl IntoResponse, Error> {
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

fn parse_fixed_rep(name: &str, value: &str) -> Result<FixedReps, Error> {
    let parts: Vec<_> = value.split("/").collect();
    if parts.len() == 0 {
        return validation_err!("{name} cannot be empty");
    } else if parts.len() == 1 {
        let reps: i32 = parts[0]
            .parse()
            .unwrap_or_err(&format!("expected int for {name} reps but found '{value}'"))?;
        return Ok(FixedReps::new(reps, 100));
    } else if parts.len() == 2 {
        let reps: i32 = parts[0]
            .parse()
            .unwrap_or_err(&format!("expected int for {name} reps but found '{value}'"))?;
        let percent: i32 = parts[1].parse().unwrap_or_err(&format!(
            "expected int for {name} percent but found '{value}'"
        ))?;
        return Ok(FixedReps::new(reps, percent));
    } else {
        return validation_err!("Expected rep or rep/percent for {name} but found '{value}'");
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
) -> Result<impl IntoResponse, Error> {
    fn parse_rep(name: &str, value: &str, percent: i32) -> Result<VariableReps, Error> {
        let parts: Vec<_> = value.split("-").collect();
        if parts.len() == 0 {
            return validation_err!("{name} cannot be empty");
        } else if parts.len() == 1 {
            let reps: i32 = parts[0]
                .parse()
                .unwrap_or_err(&format!("expected int for {name} reps but found '{value}'"))?;
            return Ok(VariableReps::new(reps, reps, percent));
        } else if parts.len() == 2 {
            let min: i32 = parts[0].parse().unwrap_or_err(&format!(
                "expected int for {name} min reps but found '{value}'"
            ))?;
            let max: i32 = parts[1].parse().unwrap_or_err(&format!(
                "expected int for {name} max reps but found '{value}'"
            ))?;
            return Ok(VariableReps::new(min, max, percent));
        } else {
            return validation_err!(
                "Expected rep or min_rep-max_rep for {name} but found '{value}'"
            );
        }
    }

    fn parse_var_rep(name: &str, value: &str) -> Result<VariableReps, Error> {
        let parts: Vec<_> = value.split("/").collect();
        if parts.len() == 0 {
            return validation_err!("{name} cannot be empty");
        } else if parts.len() == 1 {
            return parse_rep(name, parts[0], 100);
        } else if parts.len() == 2 {
            let percent: i32 = parts[1].parse().unwrap_or_err(&format!(
                "expected int for {name} percent but found '{value}'"
            ))?;
            return parse_rep(name, parts[0], percent);
        } else {
            return validation_err!(
                "Expected rep or min_rep-max_rep or rep/percent or min_rep-max_rep/percent for {name} but found '{value}'"
            );
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
) -> Result<impl IntoResponse, Error> {
    let target: i32 = payload.target.parse().unwrap_or_err(&format!(
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
) -> Result<impl IntoResponse, Error> {
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
    times: String,
    weights: String,
    comment: String,
    units: String, // "secs", "mins", or "hours"
}

async fn post_set_durs_record(
    Path((workout, exercise, id)): Path<(String, String, String)>,
    Extension(state): Extension<SharedState>,
    Form(payload): Form<SetDurationsRecord>,
) -> Result<impl IntoResponse, Error> {
    let durations = payload
        .times
        .split_whitespace()
        .map(|s| parse_time("times", s, &payload.units))
        .collect::<Result<Vec<_>, _>>()?;
    let durations = durations.iter().filter_map(|o| *o).collect();
    let weights = payload
        .weights
        .split_whitespace()
        .map(|s| s.parse::<f32>())
        .collect::<Result<Vec<_>, _>>()
        .unwrap_or_err("bad weights")?;
    let id = id
        .parse()
        .unwrap_or_err(&format!("expected integer for id but found '{id}'"))?;
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
) -> Result<impl IntoResponse, Error> {
    let reps = payload
        .reps
        .split_whitespace()
        .map(|s| s.parse::<i32>())
        .collect::<Result<Vec<_>, _>>()
        .unwrap_or_err("bad reps")?;
    let weights = payload
        .weights
        .split_whitespace()
        .map(|s| s.parse::<f32>())
        .collect::<Result<Vec<_>, _>>()
        .unwrap_or_err("bad weights")?;
    let id = id
        .parse()
        .unwrap_or_err(&format!("expected integer for id but found '{id}'"))?;
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

fn parse_time(name: &str, value: &str, units: &str) -> Result<Option<i32>, Error> {
    if !value.is_empty() {
        let mut x: f32 = value
            .parse()
            .unwrap_or_err(&format!("expected f32 for {name} but found '{value}'"))?;
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
