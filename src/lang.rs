use crate::{file_handling, Input};
use std::fs::{self, canonicalize};
use std::io;
use std::path::{Path, PathBuf};
use thiserror::Error;

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
struct FileSearchResult {
    file: PathBuf,
    line: Option<usize>,
}

#[derive(Debug)]
pub enum Action {
    Append(PathBuf),
    PrependFile(FindSpecified),
    Delete(FindSpecified),
}
#[derive(Debug)]
pub struct RemoveConfig {
    pub verbose: bool,
    pub base_path: PathBuf,
    pub dst_tag: FindSpecified,
    pub languages: String,
    pub yes: bool,
}
#[derive(Debug)]
pub struct AppendConfig {
    pub verbose: bool,
    pub base_path: PathBuf,
    pub file: PathBuf,
    pub input: Input,
    pub src_tag: String,
}
#[derive(Debug)]
pub struct InsertConfig {
    pub verbose: bool,
    pub base_path: PathBuf,
    pub input: Input,
    pub src_tag: String,
    pub dst_tag: FindSpecified,
}
#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    FileError(#[from] file_handling::Error),
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error(transparent)]
    Serde(#[from] serde_json::Error),
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(
        "no matching file+tag for => base: {base}, lang: {language}, file: {file:?}, tag: {tag:?}"
    )]
    TagSearchFailed {
        base: PathBuf,
        tag: Option<String>,
        file: Option<PathBuf>,
        language: String,
    },
    #[error("something")]
    NoSeparator,
    #[error("something else")]
    LangNoFound,
}
type MyError = &'static str;
// todo: add replace (one line support)
// todo: add specify option
pub fn run(config: Config) -> Result<(), MyError> {
    let config = if config.verbose { dbg!(config) } else { config };
    let text = if let Some(text) = config.text {
        Ok(text)
    } else {
        let file = config.input_file.unwrap();
        file_handling::read_from_file(file).map_err(|_| "something whent wrong reading input file")
    }?;
    let language_texts = gen_language_text(&text, &config.new_var).map_err(|_| "lol")?;
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
pub fn process_language_text(line: &str, var_name: &str) -> Result<(String, String), Error> {
    let (lang, text) = line.split_once(',').ok_or(Error::NoSeparator)?;
    Ok((lang.to_string(), format!("{var_name}={text:?}")))
}
pub fn gen_language_text(text: &str, var_name: &str) -> Result<Vec<(String, String)>, Error> {
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
    let inter = if s == "" || s.ends_with("\n") {
        ""
    } else {
        "\n"
    };
    let s = s + inter + value;
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
            let predicate = |path: &Path| predicate(path, lang);
            (lang.to_string(), file_handling::find_file(base, &predicate))
        })
        .collect()
}

fn general_find(
    base: impl AsRef<Path>,
    langs: &[&str],
    file: Option<&Path>,
    tag: Option<&str>,
) -> Vec<(String, Result<FileSearchResult, Error>)> {
    let base = base.as_ref().to_owned();
    langs
        .iter()
        .map(|lang| {
            let mut base = base.clone();
            base.push(lang);
            let canon_file = file.and_then(|f| canonicalize(PathBuf::from_iter([&base, f])).ok());
            let predicate = |path: &Path| {
                //eprintln!("path: {:?}\nfile: {canon_file:?}", canonicalize(path).ok());

                let out_file = match (canonicalize(path).ok().as_ref(), &canon_file) {
                    (_, None) => None,
                    (None, Some(_)) => panic!("can't canonacalize file"),
                    (Some(path), Some(file)) => Some(path == file),
                };
                match (tag, out_file) {
                    (_, Some(false)) => None,
                    (None, Some(true)) => Some(FileSearchResult {
                        file: path.to_owned(),
                        line: None,
                    }),
                    (Some(tag), _) => {
                        find_line_occurance_in_file(path, tag).map(|n| FileSearchResult {
                            file: path.to_owned(),
                            line: Some(n),
                        })
                    }
                    (None, None) => panic!("no file or tag find given"),
                }
            };
            (
                lang.to_string(),
                file_handling::find_file(&base, &predicate).ok_or(Error::TagSearchFailed {
                    base,
                    tag: tag.map(ToOwned::to_owned),
                    file: file.map(ToOwned::to_owned),
                    language: lang.to_string(),
                }),
            )
        })
        .collect()
}
pub fn append(config: AppendConfig) -> Result<(), Error> {
    if config.verbose {
        dbg!(&config);
    }
    // (extract text)
    let text = match config.input {
        Input::Text(text) => text,
        Input::File(file) => file_handling::read_from_file(&file)?,
    };
    // extract language texts
    let language_texts = gen_language_text(&text, &config.src_tag)?;

    // extract languages
    let languages: Vec<&str> = language_texts
        .iter()
        .map(|(lang, _)| lang.as_ref())
        .collect();

    // find general (file and / or needle)
    let path_per_lang = general_find(config.base_path, &languages, Some(&config.file), None);

    // additional post processing
    let path_per_lang = path_per_lang
        .into_iter()
        .map(|(lang, result)| Ok((lang, result?)))
        .collect::<Result<Vec<(String, FileSearchResult)>, Error>>()?;

    // action appand
    for (lang, search_find) in path_per_lang {
        if config.verbose {
            eprintln!("appending to file: {:?}", &search_find.file);
        }
        let replacement_text = find_match(&lang, &language_texts).ok_or(Error::LangNoFound)?;
        append_to_file(&search_find.file, replacement_text)?;
    }
    Ok(())
}

pub fn remove(config: RemoveConfig) -> Result<(), Error> {
    if config.verbose {
        dbg!(&config);
    }
    // extract languages
    let languages: Vec<&str> = config.languages.split(",").collect();

    // find general (file and / or needle)
    let path_per_lang = general_find(
        config.base_path,
        &languages,
        config.dst_tag.file.as_deref(),
        Some(&config.dst_tag.needle),
    );

    // additional post processing
    let path_per_lang = path_per_lang
        .into_iter()
        .map(|(lang, result)| Ok((lang, result?)))
        .collect::<Result<Vec<(String, FileSearchResult)>, Error>>()?;

    // action remove
    for (_, search_find) in path_per_lang {
        let index = search_find.line.expect("general_find with needle");
        if config.verbose {
            eprintln!("removing line: {index} from file: {:?}", &search_find.file);
        }

        delete_line(&search_find.file, index)?
    }
    Ok(())
}
pub fn insert(config: InsertConfig) -> Result<(), Error> {
    if config.verbose {
        dbg!(&config);
    }
    // (extract text)
    let text = match config.input {
        Input::Text(text) => text,
        Input::File(file) => file_handling::read_from_file(&file)?,
    };
    // extract language texts
    let language_texts = gen_language_text(&text, &config.src_tag)?;

    // extract languages
    let languages: Vec<&str> = language_texts
        .iter()
        .map(|(lang, _)| lang.as_ref())
        .collect();

    // find general (file and / or needle)
    let path_per_lang = general_find(
        config.base_path,
        &languages,
        config.dst_tag.file.as_deref(),
        Some(&config.dst_tag.needle),
    );

    // additional post processing
    let path_per_lang = path_per_lang
        .into_iter()
        .map(|(lang, result)| Ok((lang, result?)))
        .collect::<Result<Vec<(String, FileSearchResult)>, Error>>()?;

    // action appand
    for (lang, search_find) in path_per_lang {
        let index = search_find.line.expect("general_find with needle");
        if config.verbose {
            eprintln!("appending to file: {:?}", &search_find.file);
        }
        let replacement_text = find_match(&lang, &language_texts).ok_or(Error::LangNoFound)?;
        insert_file_at_line(&search_find.file, replacement_text, index)?;
    }
    Ok(())
}
// general flow
// (extract text)
// (extract language texts)
// extract languages
// find file
// find needle
// append / replace / remove / insert
