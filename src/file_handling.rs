use std::fs;
use std::path::Path;
pub fn write_to_file(path: impl AsRef<Path>, text: &str) -> Result<(), Box<dyn std::error::Error>> {
    fs::write(path, text)?;
    Ok(())
}
pub fn read_from_file(path: impl AsRef<Path>) -> std::io::Result<String> {
    fs::read_to_string(path)
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
