use std::path::PathBuf;

pub mod file_handling;
pub mod lang;
pub mod translate;

pub enum Input {
    File(PathBuf),
    Text(String),
}
