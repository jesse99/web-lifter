use crate::*;

pub fn make_program() -> pages::AppState {
    let blocks = vec![
        Block::new(
            "Heavy".to_owned(),
            vec!["Heavy Bench".to_owned(), "Heavy OHP".to_owned()],
            1,
        ),
        Block::new(
            "Medium".to_owned(),
            vec!["Medium Bench".to_owned(), "Medium OHP".to_owned()],
            1,
        ),
        Block::new("Rest".to_owned(), vec!["Rest".to_owned()], 1),
    ];
    let name = "My".to_owned();
    let mut default_program = Program::with_blocks(name, blocks, Local::now(), 1);
    default_program.apply(ProgramOp::Add(create_heavy_bench()));
    default_program.apply(ProgramOp::Add(create_heavy_ohp()));
    default_program.apply(ProgramOp::Add(create_medium_bench()));
    default_program.apply(ProgramOp::Add(create_medium_ohp()));
    default_program.apply(ProgramOp::Add(create_light()));
    default_program.apply(ProgramOp::Add(create_test()));

    let user = match persist::load() {
        Ok(u) => {
            let v = fixup_program(u);
            merge_program(v, default_program)
        }
        Err(e) => {
            // TODO need to better handle load errors
            // probably by adding an error label to pages
            // but that'll be easier once we support multiple users
            println!("load had error {}", e.kind());
            if e.kind() != ErrorKind::NotFound {
                default_program.name = format!("{} {e}", default_program.name);
            };
            UserState {
                notes: Notes::new(),
                history: create_history(),
                weights: creat_weight_sets(),
                program: default_program,
            }
        }
    };

    AppState {
        handlebars: Handlebars::new(),
        user,
    }
}

fn create_test() -> Workout {
    let mut workout = Workout::new("Test".to_owned(), Schedule::AnyDay);

    // Test Bench
    let warmups = vec![FixedReps::new(5, 50), FixedReps::new(3, 80)];
    let worksets = vec![VariableReps::new(1, 3, 100); 2];
    let e = VariableRepsExercise::new(warmups, worksets);
    let name = ExerciseName("Test Bench".to_owned());
    let formal_name = FormalName("Bench Press".to_owned());
    let exercise = BuildExercise::variable_reps(name.clone(), formal_name, e)
        .with_weightset("Dual Plates".to_owned())
        .with_weight(150.0)
        .with_rest_mins(3.5)
        .finalize();
    workout.apply(WorkoutOp::Add(exercise));

    // Test Chin-ups
    let e = VariableSetsExercise::new(16);
    let name = ExerciseName("Test Chin-ups".to_owned());
    let formal_name = FormalName("Chin-up".to_owned());
    let exercise = BuildExercise::variable_sets(name.clone(), formal_name, e)
        .with_rest_mins(3.5)
        .finalize();
    workout.apply(WorkoutOp::Add(exercise));

    workout
}

fn create_heavy_bench() -> Workout {
    let schedule = Schedule::Days(vec![Weekday::Tue, Weekday::Sun]);
    let mut workout = Workout::new("Heavy Bench".to_owned(), schedule);

    // // Squat
    // let warmups = vec![
    //     FixedReps::new(5, 0),
    //     FixedReps::new(5, 50),
    //     FixedReps::new(3, 70),
    //     FixedReps::new(1, 90),
    // ];
    // let worksets = vec![VariableReps::new(3, 5, 100); 3];
    // let e = VariableRepsExercise::new(warmups, worksets);
    // let name = ExerciseName("Squat".to_owned());
    // let formal_name = FormalName("High bar Squat".to_owned());
    // let exercise = BuildExercise::variable_reps(name.clone(), formal_name, e)
    //     .with_weightset("Dual Plates".to_owned())
    //     .with_weight(175.0)
    //     .with_rest_mins(3.5)
    //     .finalize();
    // workout.apply(WorkoutOp::Add(exercise));

    // // Dislocates
    // let worksets = vec![FixedReps::new(10, 100)];
    // let e = FixedRepsExercise::with_percent(vec![], worksets);
    // let name = ExerciseName("Dislocates".to_owned());
    // let formal_name = FormalName("Shoulder Dislocate (band)".to_owned());
    // let exercise = BuildExercise::fixed_reps(name.clone(), formal_name, e).finalize();
    // workout.apply(WorkoutOp::Add(exercise));

    // Heavy Bench
    let warmups = vec![
        FixedReps::new(5, 50),
        FixedReps::new(5, 70),
        FixedReps::new(3, 80),
        FixedReps::new(1, 90),
    ];
    let worksets = vec![VariableReps::new(1, 3, 100); 3];
    let e = VariableRepsExercise::new(warmups, worksets);
    let name = ExerciseName("Heavy Bench".to_owned());
    let formal_name = FormalName("Bench Press".to_owned());
    let exercise = BuildExercise::variable_reps(name.clone(), formal_name, e)
        .with_weightset("Dual Plates".to_owned())
        .with_weight(150.0)
        .with_rest_mins(3.5)
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
    let worksets = vec![VariableReps::new(5, 10, 100); 3];
    let e = VariableRepsExercise::new(warmups, worksets);
    let name = ExerciseName("Cable Abduction".to_owned());
    let formal_name = FormalName("Cable Hip Abduction".to_owned());
    let exercise = BuildExercise::variable_reps(name.clone(), formal_name, e)
        .with_weightset("Cable Machine".to_owned())
        .with_weight(12.5)
        .with_rest_mins(2.0)
        .finalize();
    workout.apply(WorkoutOp::Add(exercise));

    // Heavy RDL
    let warmups = vec![
        FixedReps::new(5, 60),
        FixedReps::new(3, 80),
        FixedReps::new(1, 90),
    ];
    let worksets = vec![VariableReps::new(1, 3, 100); 3];
    let e = VariableRepsExercise::new(warmups, worksets);
    let name = ExerciseName("Heavy RDL".to_owned());
    let formal_name = FormalName("Romanian Deadlift".to_owned());
    let exercise = BuildExercise::variable_reps(name.clone(), formal_name, e)
        .with_weightset("Deadlift".to_owned())
        .with_weight(205.0)
        .with_rest_mins(3.0)
        .with_last_rest(0)
        .finalize();
    workout.apply(WorkoutOp::Add(exercise));

    workout
}

fn create_heavy_ohp() -> Workout {
    let schedule = Schedule::Days(vec![Weekday::Thu]);
    let mut workout = Workout::new("Heavy OHP".to_owned(), schedule);

    // Heavy OHP
    let warmups = vec![
        FixedReps::new(5, 50),
        FixedReps::new(5, 70),
        FixedReps::new(3, 80),
        FixedReps::new(1, 90),
    ];
    let worksets = vec![VariableReps::new(1, 3, 100); 3];
    let e = VariableRepsExercise::new(warmups, worksets);
    let name = ExerciseName("Heavy OHP".to_owned());
    let formal_name = FormalName("Overhead Press".to_owned());
    let exercise = BuildExercise::variable_reps(name.clone(), formal_name, e)
        .with_weightset("Dual Plates".to_owned())
        .with_weight(75.0)
        .with_rest_mins(3.5)
        .finalize();
    workout.apply(WorkoutOp::Add(exercise));

    // Heavy Leg Press
    let warmups = vec![
        FixedReps::new(5, 50),
        FixedReps::new(5, 70),
        FixedReps::new(3, 80),
        FixedReps::new(1, 90),
    ];
    let worksets = vec![VariableReps::new(1, 3, 100); 3];
    let e = VariableRepsExercise::new(warmups, worksets);
    let name = ExerciseName("Heavy Leg Press".to_owned());
    let formal_name = FormalName("Leg Press".to_owned());
    let exercise = BuildExercise::variable_reps(name.clone(), formal_name, e)
        .with_weightset("Dual Plates".to_owned())
        .with_weight(160.0)
        .with_rest_mins(3.5)
        .finalize();
    workout.apply(WorkoutOp::Add(exercise));

    // Chin-ups
    let e = VariableSetsExercise::new(16);
    let name = ExerciseName("Chin-ups".to_owned());
    let formal_name = FormalName("Chin-up".to_owned());
    let exercise = BuildExercise::variable_sets(name.clone(), formal_name, e)
        .with_rest_mins(3.5)
        .finalize();
    workout.apply(WorkoutOp::Add(exercise));

    // Face Pulls
    let warmups = vec![FixedReps::new(6, 75)];
    let worksets = vec![VariableReps::new(8, 12, 100); 3];
    let e = VariableRepsExercise::new(warmups, worksets);
    let name = ExerciseName("Face Pulls".to_owned());
    let formal_name = FormalName("Face Pull".to_owned());
    let exercise = BuildExercise::variable_reps(name.clone(), formal_name, e)
        .with_weightset("Cable Machine".to_owned())
        .with_weight(37.5)
        .with_rest_mins(2.0)
        .finalize();
    workout.apply(WorkoutOp::Add(exercise));

    workout
}

fn create_medium_bench() -> Workout {
    let schedule = Schedule::Days(vec![Weekday::Tue, Weekday::Sun]);
    let mut workout = Workout::new("Medium Bench".to_owned(), schedule);

    // Medium Bench
    let warmups = vec![
        FixedReps::new(5, 50),
        FixedReps::new(5, 70),
        FixedReps::new(3, 80),
        FixedReps::new(1, 90),
    ];
    let worksets = vec![VariableReps::new(3, 6, 100); 3];
    let e = VariableRepsExercise::new(warmups, worksets);
    let name = ExerciseName("Medium Bench".to_owned());
    let formal_name = FormalName("Bench Press".to_owned());
    let exercise = BuildExercise::variable_reps(name.clone(), formal_name, e)
        .with_weightset("Dual Plates".to_owned())
        .with_weight(135.0)
        .with_rest_mins(3.5)
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
    let worksets = vec![VariableReps::new(5, 10, 100); 3];
    let e = VariableRepsExercise::new(warmups, worksets);
    let name = ExerciseName("Cable Abduction".to_owned());
    let formal_name = FormalName("Cable Hip Abduction".to_owned());
    let exercise = BuildExercise::variable_reps(name.clone(), formal_name, e)
        .with_weightset("Cable Machine".to_owned())
        .with_weight(12.5)
        .with_rest_mins(2.0)
        .finalize();
    workout.apply(WorkoutOp::Add(exercise));

    // Medium RDL
    let warmups = vec![
        FixedReps::new(5, 60),
        FixedReps::new(3, 80),
        FixedReps::new(1, 90),
    ];
    let worksets = vec![VariableReps::new(3, 6, 100); 3];
    let e = VariableRepsExercise::new(warmups, worksets);
    let name = ExerciseName("Medium RDL".to_owned());
    let formal_name = FormalName("Romanian Deadlift".to_owned());
    let exercise = BuildExercise::variable_reps(name.clone(), formal_name, e)
        .with_weightset("Deadlift".to_owned())
        .with_weight(205.0)
        .with_rest_mins(3.0)
        .with_last_rest(0)
        .finalize();
    workout.apply(WorkoutOp::Add(exercise));

    workout
}

fn create_medium_ohp() -> Workout {
    let schedule = Schedule::Days(vec![Weekday::Thu]);
    let mut workout = Workout::new("Medium OHP".to_owned(), schedule);

    // OHP
    let warmups = vec![
        FixedReps::new(5, 50),
        FixedReps::new(5, 70),
        FixedReps::new(3, 80),
        FixedReps::new(1, 90),
    ];
    let worksets = vec![VariableReps::new(3, 6, 100); 3];
    let e = VariableRepsExercise::new(warmups, worksets);
    let name = ExerciseName("OHP".to_owned());
    let formal_name = FormalName("Overhead Press".to_owned());
    let exercise = BuildExercise::variable_reps(name.clone(), formal_name, e)
        .with_weightset("Dual Plates".to_owned())
        .with_weight(75.0)
        .with_rest_mins(3.5)
        .finalize();
    workout.apply(WorkoutOp::Add(exercise));

    // Leg Press
    let warmups = vec![
        FixedReps::new(5, 50),
        FixedReps::new(5, 70),
        FixedReps::new(3, 80),
        FixedReps::new(1, 90),
    ];
    let worksets = vec![VariableReps::new(3, 6, 100); 3];
    let e = VariableRepsExercise::new(warmups, worksets);
    let name = ExerciseName("Leg Press".to_owned());
    let formal_name = FormalName("Leg Press".to_owned());
    let exercise = BuildExercise::variable_reps(name.clone(), formal_name, e)
        .with_weightset("Machine Plates".to_owned())
        .with_weight(160.0)
        .with_rest_mins(3.5)
        .finalize();
    workout.apply(WorkoutOp::Add(exercise));

    let e = VariableSetsExercise::new(6);
    let name = ExerciseName("Medium Chin-ups".to_owned());
    let formal_name = FormalName("Chin-up".to_owned());
    let exercise = BuildExercise::variable_sets(name.clone(), formal_name, e)
        .with_rest_mins(3.0)
        .finalize();
    workout.apply(WorkoutOp::Add(exercise));

    // Face Pulls
    let warmups = vec![FixedReps::new(6, 75)];
    let worksets = vec![VariableReps::new(8, 12, 100); 3];
    let e = VariableRepsExercise::new(warmups, worksets);
    let name = ExerciseName("Face Pulls".to_owned());
    let formal_name = FormalName("Face Pull".to_owned());
    let exercise = BuildExercise::variable_reps(name.clone(), formal_name, e)
        .with_weightset("Cable Machine".to_owned())
        .with_weight(37.5)
        .with_rest_mins(2.0)
        .finalize();
    workout.apply(WorkoutOp::Add(exercise));

    workout
}

fn create_light() -> Workout {
    let schedule = Schedule::Days(vec![Weekday::Tue, Weekday::Sun]);
    let mut workout = Workout::new("Light".to_owned(), schedule);

    // Couch Stretch
    let e = DurationsExercise::new(vec![20; 4]).with_target_secs(120);
    let name = ExerciseName("Couch Stretch".to_owned());
    let formal_name = FormalName("Couch Stretch".to_owned());
    let exercise = BuildExercise::durations(name, formal_name, e).finalize();
    workout.apply(WorkoutOp::Add(exercise));

    // Face Pulls
    let warmups = vec![FixedReps::new(6, 75)];
    let worksets = vec![VariableReps::new(6, 12, 100); 3];
    let e = VariableRepsExercise::new(warmups, worksets);
    let name = ExerciseName("Face Pulls".to_owned());
    let formal_name = FormalName("Face Pull".to_owned());
    let exercise = BuildExercise::variable_reps(name.clone(), formal_name, e)
        .with_weightset("Cable Machine".to_owned())
        .with_weight(37.5)
        .with_rest_mins(2.0)
        .finalize();
    workout.apply(WorkoutOp::Add(exercise));

    // Light Chin-ups
    let e = VariableSetsExercise::new(12);
    let name = ExerciseName("Light Chin-ups".to_owned());
    let formal_name = FormalName("Chin-up".to_owned());
    let exercise = BuildExercise::variable_sets(name.clone(), formal_name, e)
        .with_rest_mins(3.0)
        .finalize();
    workout.apply(WorkoutOp::Add(exercise));

    // Cable Crunchs
    let warmups = vec![FixedReps::new(6, 75)];
    let worksets = vec![VariableReps::new(6, 12, 100); 3];
    let e = VariableRepsExercise::new(warmups, worksets);
    let name = ExerciseName("Cable Crunchs".to_owned());
    let formal_name = FormalName("Cable Crunch".to_owned());
    let exercise = BuildExercise::variable_reps(name.clone(), formal_name, e)
        .with_weightset("Cable Machine".to_owned())
        .with_weight(17.5)
        .with_rest_mins(2.0)
        .finalize();
    workout.apply(WorkoutOp::Add(exercise));

    workout
}

fn create_history() -> History {
    fn add(history: &mut History, name: &ExerciseName, reps: Vec<i32>, weight: f32, days_ago: i64) {
        let date = Local::now() - chrono::Duration::days(days_ago);
        let record = Record {
            program: "My".to_owned(),
            workout: "Heavy Bench".to_owned(),
            started: date,
            completed: Some(date),
            sets: None,
            comment: None,
        };
        history.start(&name, record);
        for rep in reps {
            history.append_reps(&name, rep, Some(weight));
        }
        history.finish(&name);
    }

    fn add_squat(history: &mut History) {
        let name = ExerciseName("Squat".to_owned());
        add(history, &name, vec![5, 4, 3], 175.0, 19);
        add(history, &name, vec![5, 4, 4], 175.0, 16);
        add(history, &name, vec![5, 5, 4], 175.0, 13);
        add(history, &name, vec![5, 5, 4], 175.0, 8);
        add(history, &name, vec![5, 5, 5], 175.0, 5);
        add(history, &name, vec![4, 3, 3], 175.0, 2);
    }

    fn add_bench(history: &mut History) {
        let name = ExerciseName("Heavy Bench".to_owned());
        add(history, &name, vec![3, 3, 3], 150.0, 20);
        add(history, &name, vec![3, 3, 2], 150.0, 17);
        add(history, &name, vec![4, 3, 3], 150.0, 14);
        add(history, &name, vec![4, 4, 3], 150.0, 11);
        add(history, &name, vec![4, 4, 3], 150.0, 6);
        add(history, &name, vec![4, 4, 3], 150.0, 4);
        add(history, &name, vec![3, 3, 3], 150.0, 1);
    }

    fn add_rdl(history: &mut History) {
        let name = ExerciseName("Heavy RDL".to_owned());
        add(history, &name, vec![8, 8, 8], 135.0, 11);
        add(history, &name, vec![8, 8, 8], 145.0, 7);
        add(history, &name, vec![8, 8, 8], 155.0, 4);
        add(history, &name, vec![8, 8, 8], 165.0, 1);
    }

    fn add_abduction(history: &mut History) {
        let name = ExerciseName("Cable Abduction".to_owned());
        add(history, &name, vec![10, 10, 10], 12.5, 11);
        add(history, &name, vec![10, 10, 4], 17.5, 7);
        add(history, &name, vec![10, 10, 10], 12.5, 4);
        add(history, &name, vec![10, 10, 10], 12.5, 1);
    }

    let mut history = History::new();
    add_squat(&mut history);
    add_bench(&mut history);
    add_rdl(&mut history);
    add_abduction(&mut history);
    history
}

fn creat_weight_sets() -> Weights {
    let mut weights = Weights::new();

    let set = WeightSet::DualPlates(
        vec![
            Plate::new(5.0, 4),
            Plate::new(10.0, 4),
            Plate::new(25.0, 6),
            Plate::new(45.0, 6),
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
    weights.add("Dual Plates".to_owned(), set);

    let set = WeightSet::DualPlates(
        vec![
            Plate::new(5.0, 4),
            Plate::new(10.0, 4),
            Plate::new(25.0, 8),
            Plate::new(45.0, 8),
        ],
        None,
    );
    weights.add("Machine Plates".to_owned(), set);

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

fn fixup_program(mut state: UserState) -> UserState {
    state.history.fixup();
    state.program.fixup();
    state
}

fn merge_program(mut state: UserState, default_program: Program) -> UserState {
    let loaded_names: Vec<_> = state.program.workouts().map(|w| w.name.clone()).collect();

    for new_workout in default_program.workouts() {
        if let Some(loaded_workout) = state.program.find_mut(&new_workout.name) {
            let loaded_exercises: Vec<_> = loaded_workout
                .exercises()
                .map(|e| e.name().clone())
                .collect();
            for loaded_exercise in loaded_exercises {
                if new_workout.find(&loaded_exercise).is_none() {
                    println!(
                        "removing old exercise '{}' from '{}'",
                        loaded_exercise, new_workout.name
                    );
                    loaded_workout.apply(WorkoutOp::Del(loaded_exercise));
                }
            }
            for new_exercise in new_workout.exercises() {
                if loaded_workout.find(new_exercise.name()).is_none() {
                    println!(
                        "adding new exercise '{}' to '{}'",
                        new_exercise.name(),
                        new_workout.name
                    );
                    loaded_workout.apply(WorkoutOp::Add(new_exercise.clone()));
                }
            }
        } else {
            println!("adding new workout '{}'", new_workout.name);
            state.program.apply(ProgramOp::Add(new_workout.clone()));
        }
    }

    for loaded_workout in loaded_names {
        if default_program.find(&loaded_workout).is_none() {
            println!("removing old workout '{}'", loaded_workout);
            state.program.apply(ProgramOp::Del(loaded_workout));
        }
    }
    state
}
