use std::fs;
use std::path::Path;
pub fn write_to_file(path: impl AsRef<Path>, text: &str) -> Result<(), Box<dyn std::error::Error>> {
    fs::write(path, text)?;
    Ok(())
}
pub fn read_from_file(path: impl AsRef<Path>) -> std::io::Result<String> {
    fs::read_to_string(path)
}
