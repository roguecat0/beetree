use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("error: reading file {0}")]
    Read(PathBuf, #[source] std::io::Error),
    #[error("error: writing file: {0}")]
    Write(PathBuf, #[source] std::io::Error),
}

pub fn write_to_file(path: impl AsRef<Path>, text: &str) -> Result<(), Error> {
    let buf = path.as_ref().to_owned();
    fs::write(path, text).map_err(|e| Error::Write(buf, e))
}
pub fn read_from_file(path: impl AsRef<Path>) -> Result<String, Error> {
    let buf = path.as_ref().to_owned();
    fs::read_to_string(path).map_err(|e| Error::Read(buf, e))
}

pub fn find_file<F, T>(base: impl AsRef<Path>, predicate: &F) -> Option<T>
where
    F: Fn(&Path) -> Option<T>,
{
    let base = base.as_ref();
    let res = if !base.is_dir() {
        // let text = fs::read_to_string(base).ok()?;
        predicate(&base)
    } else {
        let res = base
            .read_dir()
            .ok()?
            .flatten()
            .find_map(|p| find_file(p.path(), predicate));
        res
    };
    res
}
