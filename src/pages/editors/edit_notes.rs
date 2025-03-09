use crate::app_state::SharedState;
use crate::errors::Error;
use crate::pages::editor_builder::*;
use axum::http::Uri;

/// For program notes.
pub fn get_edit_notes(state: SharedState) -> String {
    let post_url = format!("/set-notes");
    let cancel_url = format!("/");

    let program = &state.read().unwrap().user.program;
    let widgets: Vec<Box<dyn Widget>> = vec![
        Box::new(Prolog::with_title("Edit Program Notes")),
        Box::new(
            TextArea::new(
                "notes",
                15,
                60,
                "Information about the program, e.g. when to progress.",
            )
            .with_spellcheck()
            .with_autocapitalize("sentences")
            .with_body(&program.notes),
        ),
        Box::new(StdButtons::new(&cancel_url)),
    ];

    build_editor(&post_url, widgets)
}

pub fn post_set_notes(state: SharedState, notes: String) -> Result<Uri, Error> {
    {
        let program = &mut state.write().unwrap().user.program;
        program.set_notes(notes);
    }

    let path = format!("/");
    crate::pages::post_epilog(state, &path)
}
