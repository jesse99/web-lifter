// There are more efficient formats than json but json tends to do better handling changes
// than the binary formats and it is (more or less) human readable which is nice. The
// only place where efficiency may matter is for the history but even that should be OK
// until we start getting many thousands of entries. TODO would be nice to track this
// via some sort of stat (maybe by monitoring saved file sizes).
//
// The serde_flow crate can help if backward compatibility starts to become an issue.
// There's also serde_columnar but that isn't production ready (in early 2024).
use crate::app_state::UserState;
use std::fs::File;
use std::io::{BufWriter, Result};
use std::path::PathBuf;

pub fn save(state: &UserState) -> Result<()> {
    let path = get_data_path()?;
    let file = File::create(path)?;
    let file = BufWriter::new(file);
    serde_json::to_writer(file, state)?;
    Ok(())
}

pub fn load() -> Result<UserState> {
    let path = get_data_path()?;
    let file = File::open(path)?;
    let state = serde_json::from_reader(file)?;
    Ok(state)
}

// On my machine this is /Users/jessejones/Library/Application\ Support/web-lifter/mine
fn get_data_path() -> Result<PathBuf> {
    if let Some(mut path) = dirs::data_dir() {
        path.push("web-lifter");
        if !path.exists() {
            std::fs::create_dir(path.clone())?
        }

        // In the future we can map my name/password to "mine".
        path.push("mine");
        Ok(path)
    } else {
        Err(std::io::Error::other("Couldn't find data_dir"))
    }
}
