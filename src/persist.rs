use crate::*;
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

fn get_data_path() -> Result<PathBuf> {
    if let Some(mut path) = dirs::data_dir() {
        path.push("web-lifter");
        if !path.exists() {
            std::fs::create_dir(path.clone())?
        }
        path.push("mine");
        Ok(path)
    } else {
        Err(std::io::Error::other("Couldn't find data_dir"))
    }
}
