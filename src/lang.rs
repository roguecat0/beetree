use crate::file_handling;
use std::fs::{self, canonicalize};
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct Config {
    pub verbose: bool,
    pub base_path: PathBuf,
    pub text: Option<String>,
    pub input_file: Option<PathBuf>,
    pub action: Action,
    pub new_var: String,
}
#[derive(Debug)]
pub struct FindSpecified {
    pub needle: String,
    pub file: Option<PathBuf>,
}
#[derive(Debug)]
pub enum Action {
    Append(PathBuf),
    PrependFile(FindSpecified),
    Delete(FindSpecified),
}
type MyError = &'static str;
// todo: add replace (one line support)
// todo: add specify option
// todo: add remove (one line support)
pub fn run(config: Config) -> Result<(), MyError> {
    let config = if config.verbose { dbg!(config) } else { config };
    let text = if let Some(text) = config.text {
        Ok(text)
    } else {
        let file = config.input_file.unwrap();
        file_handling::read_from_file(file).map_err(|_| "something whent wrong reading input file")
    }?;
    let language_texts = gen_language_text(&text, &config.new_var)?;
    let langs: Vec<String> = language_texts
        .iter()
        .map(|(lang, _)| lang.to_string())
        .collect();

    match config.action {
        Action::Append(file) => {
            let append_paths: Vec<_> = langs
                .iter()
                .map(|lang| {
                    (
                        lang.to_string(),
                        canonicalize(PathBuf::from_iter([
                            &config.base_path,
                            Path::new(lang),
                            &file,
                        ]))
                        .ok(),
                    )
                })
                .collect();
            let path_per_lang = language_base_find_file(config.base_path, &langs, &|path, lang| {
                let Some(append_path) = find_match(lang, &append_paths) else {
                    panic!("how even?")
                };
                if canonicalize(path).ok().as_ref() == append_path.as_ref() {
                    Some(path.to_owned())
                } else {
                    None
                }
            });
            for (lang, buff) in path_per_lang
                .into_iter()
                .map(|(lang, buff)| (lang, buff.ok_or("path not found for lang: l")))
            {
                let p = buff?;
                let replacement_text =
                    find_match(&lang, &language_texts).ok_or("no match in language texts...")?;
                append_to_file(&p, replacement_text).map_err(|_| "failed to append to file")?;
            }
        }
        Action::PrependFile(FindSpecified { needle, .. }) => {
            let path_per_lang = language_base_find_file(config.base_path, &langs, &|path, _| {
                find_line_occurance_in_file(path, &needle).map(|n| (path.to_owned(), n))
            });

            for (lang, buff) in path_per_lang
                .into_iter()
                .map(|(lang, buff)| (lang, buff.ok_or("path not found for lang: l")))
            {
                let (p, index) = buff?;
                let replacement_text =
                    find_match(&lang, &language_texts).ok_or("no match in language texts...")?;

                insert_file_at_line(p, replacement_text, index)
                    .map_err(|_| "failed to append to file")?;
            }
        }
        Action::Delete(FindSpecified { needle, .. }) => {
            let path_per_lang = language_base_find_file(config.base_path, &langs, &|path, _| {
                find_line_occurance_in_file(path, &needle).map(|n| (path.to_owned(), n))
            });

            for (_, buff) in path_per_lang
                .into_iter()
                .map(|(lang, buff)| (lang, buff.ok_or("path not found for lang: l")))
            {
                let (p, index) = buff?;
                delete_line(p, index).map_err(|_| "failed to append to file")?;
            }
        }
    }
    Ok(())
}
pub fn find_line_occurance_in_file(path: impl AsRef<Path>, variable: &str) -> Option<usize> {
    let s = fs::read_to_string(&path).ok()?;
    find_line_occurance(&s, variable)
}
pub fn find_match<'a, T>(lang: &str, values: &'a [(String, T)]) -> Option<&'a T> {
    values.iter().find(|(l, _)| l == lang).map(|(_, t)| t)
}
pub fn process_language_text(line: &str, var_name: &str) -> Result<(String, String), MyError> {
    let (lang, text) = line
        .split_once(',')
        .ok_or("no ',' separator between lang and text")?;
    Ok((lang.to_string(), format!("{var_name}={text:?}")))
}
pub fn gen_language_text(text: &str, var_name: &str) -> Result<Vec<(String, String)>, MyError> {
    text.lines()
        .map(|line| process_language_text(line, var_name))
        .collect()
}
fn find_line_occurance(text: &str, variable: &str) -> Option<usize> {
    text.lines()
        .enumerate()
        .find_map(|(i, line)| line.starts_with(variable).then_some(i))
}

fn append_to_file(path: impl AsRef<Path>, value: &str) -> io::Result<()> {
    let s: String = fs::read_to_string(&path)?;
    let s = s + "\n" + value;
    fs::write(&path, &s)?;
    Ok(())
}
fn insert_file_at_line(path: impl AsRef<Path>, value: &str, index: usize) -> io::Result<()> {
    let s: String = fs::read_to_string(&path)?;
    let s = if s.len() == 0 {
        value.to_string()
    } else if index == 0 {
        value.to_owned() + "\n" + &s
    } else {
        s.lines()
            .enumerate()
            .map(|(i, l)| {
                if i == index - 1 {
                    l.to_owned() + "\n" + value
                } else {
                    l.to_owned()
                }
            })
            .collect::<Vec<String>>()
            .join("\n")
    };
    fs::write(&path, &s)?;
    Ok(())
}
fn delete_line(path: impl AsRef<Path>, index: usize) -> io::Result<()> {
    let s: String = fs::read_to_string(&path)?;
    let s = s
        .lines()
        .enumerate()
        .filter_map(|(i, l)| (i != index).then(|| l.to_string()))
        .collect::<Vec<String>>()
        .join("\n");
    fs::write(&path, &s)?;
    Ok(())
}
pub fn language_base_find_file<F, T>(
    base: impl AsRef<Path>,
    langs: &[String],
    predicate: &F,
) -> Vec<(String, Option<T>)>
where
    F: Fn(&Path, &str) -> Option<T>,
{
    let base = base.as_ref().to_owned();
    langs
        .iter()
        .map(|lang| {
            let mut base = base.clone();
            base.push(lang);
            println!("{base:?}");
            // not very efficient if lang part is calculation heavy
            let predicate = |path: &Path| predicate(path, lang);
            (lang.to_string(), file_handling::find_file(base, &predicate))
        })
        .collect()
}
