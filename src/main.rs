use axum::{
    extract::Extension,
    http::{header, HeaderValue, Response, StatusCode},
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
    let mut workout1 = Workout::new("Full Body".to_owned(), Schedule::Every(2));
    let workout2 = Workout::new("Cardio".to_owned(), Schedule::AnyDay);

    let name1 = ExerciseName("Quad Stretch".to_owned());
    workout1.apply(WorkoutOp::Add(name1.clone()));

    let name2 = ExerciseName("Side Leg Lift".to_owned());
    workout1.apply(WorkoutOp::Add(name2.clone()));

    let mut program = Program::new("My".to_owned());
    program.apply(ProgramOp::Add(workout1));
    program.apply(ProgramOp::Add(workout2));

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

static PAGE: &'static str = "<!DOCTYPE html>
<html lang=\"en\">
	<head>
		<meta charset=\"utf-8\">
		<title>web lifter</title>

        <link rel=\"stylesheet\" href=\"https://stackpath.bootstrapcdn.com/bootstrap/4.4.1/css/bootstrap.min.css\" integrity=\"sha384-Vkoo8x4CGsO3+Hhxv8T/Q5PaXtkKtu6ug5TOeNV6gBiFeWPGFN9MuhOf23Q9Ifjh\" crossorigin=\"anonymous\">
        <link href=\"styles/style.css?version=2\" rel=\"stylesheet\">
	</head>
	<body>
		$BODY
	</body>
</html>";

// TODO: use a real template engine?
fn replace(template: &str, key: &str, value: &str) -> String {
    template.replace(key, value)
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

// TODO:
// can we use something to help build html?
// do a commit
// add routes for workouts
async fn get_program(Extension(state): Extension<SharedState>) -> impl IntoResponse {
    let program = &state.read().unwrap().program;

    // Note that MDN recommends against using aria tables, see https://developer.mozilla.org/en-US/docs/Web/Accessibility/ARIA/Roles/table_role
    let mut body = String::new();
    body += &format!("<h2 id=\"title\">{} Program</h2>\n", program.name);
    body += "<div class=\"table-container\" role=\"table\" aria-label=\"Program\">\n";
    for workout in program.workouts() {
        let link = format!(
            "<a href=\"./workout/{}\">{}</a>",
            workout.name, workout.name
        ); // TODO: what if name has markup symbols?

        body += "   <div class=\"flex-table row\" role=\"rowgroup\">\n";
        body += &format!(
            "      <div class=\"flex-row\" role=\"cell\">{}</div>\n",
            link
        );

        let status = workout.status();
        body += &format!(
            "      <div class=\"flex-row {}\" role=\"cell\">{}</div>\n",
            status.to_class(),
            status.to_label()
        );
        body += "   </div>\n";
    }
    body += "</div>\n";

    let markup = replace(PAGE, "$TITLE", "web-lifter program");
    let markup = replace(&markup, "$BODY", &body);
    Html(markup)
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
