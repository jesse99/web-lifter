use crate::pages::Error;
use crate::{
    exercise::{
        DurationsExercise, Exercise, ExerciseData, FixedReps, FixedRepsExercise, SetIndex,
        VariableReps, VariableRepsExercise, VariableSetsExercise,
    },
    pages::SharedState,
    program::Program,
    weights,
    workout::Schedule,
};
use serde::{Deserialize, Serialize};

pub fn get_overview_page(state: SharedState) -> Result<String, Error> {
    let handlebars = &state.read().unwrap().handlebars;
    let program = &state.read().unwrap().user.program;

    let template = include_str!("../../files/overview.html");
    let data = OverviewData::new(program);
    let contents = handlebars.render_template(template, &data)?;
    Ok(contents)
}

#[derive(Serialize, Deserialize)]
struct OverviewData {
    text: String,
}

const INDENT: &str = "&nbsp;&nbsp;&nbsp;&nbsp;";

impl OverviewData {
    fn new(program: &Program) -> OverviewData {
        let mut text = String::new();
        text += &format!("<strong>{} Program</strong><br>", program.name);
        if !program.blocks().count() > 0 {
            text += "Blocks<br>";
            for block in program.blocks() {
                text += &format!("{INDENT}{}<br>", block.name);

                let names = block.workouts.join(", ");
                text += &format!("{INDENT}{INDENT}workouts: {names}<br>");
                text += &format!("{INDENT}{INDENT}num weeks: {}<br>", block.num_weeks);
            }
            if let Some((week, block)) = program.current_block() {
                text += &format!("Current Week: {week} ({})<br>", block.name);
            }
        }

        for workout in program.workouts() {
            text += &format!("<br><strong>{} Workout</strong>  ", workout.name);
            text += &match &workout.schedule {
                Schedule::AnyDay => "any day".to_owned(),
                Schedule::Every(n) => format!("every {n} days"),
                Schedule::Days(days) => days
                    .iter()
                    .map(|d| d.to_string())
                    .collect::<Vec<String>>()
                    .join(" "),
            };
            text += "<br>";
            for exercise in workout.exercises() {
                let d = exercise.data();
                if d.enabled {
                    text += &format!("<u>{}</u><br>", exercise.name());
                    text += &exercise_details(exercise);
                } else {
                    text += &format!("<u>{}</u> (disabled)<br>", exercise.name());
                }
            }
        }
        OverviewData { text }
    }
}

fn exercise_details(exercise: &Exercise) -> String {
    fn durations_details(e: &DurationsExercise) -> String {
        let mut text = String::new();
        let sets = &(0..e.num_sets())
            .map(|i| format!("{}s", e.set(SetIndex::Workset(i)))) // TODO convert to a short time, eg secs or mins
            .collect::<Vec<String>>()
            .join(" ");
        text += &format!("{INDENT}sets: {sets}<br>"); // TODO use css to indent?
        if let Some(target) = e.target() {
            text += &format!("{INDENT}target: {target}s<br>"); // TODO convert to a short time, eg secs or mins
        }
        text
    }

    fn rep_to_str(r: &FixedReps) -> String {
        if r.percent == 100 {
            format!("{}", r.reps)
        } else {
            format!("{}/{}", r.reps, r.percent)
        }
    }

    fn fixed_details(e: &FixedRepsExercise) -> String {
        let mut text = String::new();
        if e.num_warmups() > 0 {
            let sets = &(0..e.num_warmups())
                .map(|i| rep_to_str(e.set(SetIndex::Warmup(i))))
                .collect::<Vec<String>>()
                .join(" ");
            text += &format!("{INDENT}warmups: {sets}<br>");
        }
        let sets = &(0..e.num_worksets())
            .map(|i| rep_to_str(e.set(SetIndex::Workset(i))))
            .collect::<Vec<String>>()
            .join(" ");
        text += &format!("{INDENT}worksets: {sets}<br>");
        text
    }

    fn var_reps_to_str(r: &VariableReps) -> String {
        let prefix = if r.min == r.max {
            format!("{}", r.min)
        } else {
            format!("{}-{}", r.min, r.max)
        };
        let suffix = if r.percent == 100 {
            "".to_owned()
        } else {
            format!("/{}", r.percent)
        };
        format!("{prefix}{suffix}")
    }

    fn var_reps_details(e: &VariableRepsExercise) -> String {
        let mut text = String::new();
        if e.num_warmups() > 0 {
            let sets = &(0..e.num_warmups())
                .map(|i| rep_to_str(e.warmup(i)))
                .collect::<Vec<String>>()
                .join(" ");
            text += &format!("{INDENT}warmups: {sets}<br>");
        }
        let sets = &(0..e.num_worksets())
            .map(|i| var_reps_to_str(e.workset(i)))
            .collect::<Vec<String>>()
            .join(" ");
        text += &format!("{INDENT}worksets: {sets}<br>");
        text
    }

    fn var_sets_details(e: &VariableSetsExercise) -> String {
        let mut text = String::new();
        text += &format!("{INDENT}reps: {}", e.target());

        let num_sets = e.get_previous().len();
        if num_sets > 0 {
            text += &format!(" over {num_sets}+ sets");
        }
        text += "<br>";
        text
    }

    fn data_details(d: &ExerciseData) -> String {
        let mut text = String::new();
        if let Some(weight) = d.weight {
            text += &format!(
                "{INDENT}weight: {}<br>",
                weights::format_weight(weight, " lbs")
            );
        }
        if let Some(weightset) = &d.weightset {
            text += &format!("{INDENT}weight set: {weightset}<br>");
        }
        if let Some(rest) = &d.rest {
            text += &format!("{INDENT}rest: {rest}s<br>"); // TODO convert to a short time, eg secs or mins
        }
        if let Some(last_rest) = &d.last_rest {
            if *last_rest > 0 {
                text += &format!("{INDENT}last rest: {last_rest}s<br>");
                // TODO convert to a short time, eg secs or mins
            }
        }
        text
    }
    match exercise {
        Exercise::Durations(d, e) => durations_details(e) + &data_details(d),
        Exercise::FixedReps(d, e) => fixed_details(e) + &data_details(d),
        Exercise::VariableReps(d, e) => var_reps_details(e) + &data_details(d),
        Exercise::VariableSets(d, e) => var_sets_details(e) + &data_details(d),
    }
}
