pub struct Config {
    pub text: Option<String>,
    pub input_file: Option<String>,
    pub base_path: Option<String>,
    pub append_file: Option<String>,
    pub find_var: Option<String>,
    pub verbose: bool,
}
pub fn run(config: Config) -> Result<(), &'static str> {
    Ok(())
}
