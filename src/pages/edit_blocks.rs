use super::SharedState;
use crate::pages::editor_builder::*;
use crate::pages::Error;
use axum::http::Uri;

pub fn get_blocks(state: SharedState) -> String {
    let post_url = "/set-blocks";
    let cancel_url = "/";

    let program = &state.read().unwrap().user.program;
    let items: Vec<_> = program.blocks().map(|b| b.name.clone()).collect();
    let javascript = include_str!("../../files/blocks.js");

    let buttons = vec![
        EditButton::new("add-btn", "on_add()", "Add"),
        EditButton::new("delete-btn", "on_delete()", "Delete"),
        EditButton::new("down-btn", "on_move_down()", "Move Down"),
        EditButton::new("up-btn", "on_move_up()", "Move Up"),
    ];
    let help = "Workouts are scheduled for the block within the current week.";

    let widgets: Vec<Box<dyn Widget>> = vec![
        Box::new(Prolog::with_edit_menu("Edit Blocks", buttons, javascript)),
        Box::new(List::with_names("blocks", items, help).without_js()),
        Box::new(StdButtons::new(&cancel_url)),
    ];

    build_editor(&post_url, widgets)
}

pub fn post_set_blocks(state: SharedState, blocks: Vec<String>) -> Result<Uri, Error> {
    let path = "/";

    {
        let program = &mut state.write().unwrap().user.program;
        program.try_set_blocks(blocks)?;
    }

    super::post_epilog(state, &path)
}
