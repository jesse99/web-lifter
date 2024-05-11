use crate::pages::editor_builder::*;
use crate::pages::Error;
use crate::pages::SharedState;
use axum::http::Uri;

pub fn get_edit_block(state: SharedState, block_name: &str) -> String {
    let post_url = format!("/set-block/{block_name}");
    let cancel_url = "/";

    let program = &state.read().unwrap().user.program;
    let block = program.blocks().find(|b| b.name == block_name);
    let items = program
        .workouts()
        .map(|w| {
            (
                w.name.clone(),
                w.name.clone(),
                block.map_or(false, |b| {
                    b.workouts.iter().find(|w2| **w2 == w.name).is_some()
                }),
            )
        })
        .collect();
    let num_weeks = block.map(|b| b.num_weeks as f32);

    let widgets: Vec<Box<dyn Widget>> = vec![
        Box::new(Prolog::with_title(&format!("Edit {block_name} Block"))),
        Box::new(
            TextInput::new(
                "Name",
                block_name,
                "Name must be unique within the program's blocks.",
            )
            .with_required(),
        ),
        Box::new(
            FloatInput::new(
                "Num Weeks",
                num_weeks,
                "Number of weeks the block is active for.",
            )
            .with_min(1.0)
            .with_step(1.0)
            .with_required(),
        ),
        Box::new(Checkbox::new(
            "workouts",
            items,
            "Workouts to schedule when the block is active. Can be empty if you don't want to schedule anything for the block.",
        )),
        Box::new(StdButtons::new(&cancel_url)),
    ];

    build_editor(&post_url, widgets)
}

pub fn post_set_block(
    state: SharedState,
    old_name: &str,
    new_name: &str,
    num_weeks: i32,
    workouts: Vec<String>,
) -> Result<Uri, Error> {
    {
        let program = &mut state.write().unwrap().user.program;
        program.try_set_block(old_name, new_name, num_weeks, workouts)?;
    }

    let path = "/";
    super::post_epilog(state, &path)
}
