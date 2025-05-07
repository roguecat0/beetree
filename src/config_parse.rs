use beetree::lang;
use beetree::lang::{Action, FindSpecified};
use beetree::translate;
use clap::ArgMatches;
use std::path::PathBuf;

pub trait ToConfig<T> {
    type Error;
    fn to_config(&self) -> Result<T, Self::Error>;
}

impl ToConfig<translate::Config> for ArgMatches {
    type Error = anyhow::Error;
    fn to_config(&self) -> Result<translate::Config, Self::Error> {
        let api_key = self
            .get_one::<String>("api_key")
            .expect("default")
            .to_string();
        let model = self
            .get_one::<String>("model")
            .expect("required")
            .to_string();
        let host = self
            .get_one::<String>("host")
            .expect("required")
            .to_string();
        let output_file = self.get_one::<PathBuf>("output_file").cloned();
        let input = if let Some(text) = self.get_one::<String>("text") {
            beetree::Input::Text(text.to_string())
        } else {
            let file = self.get_one::<PathBuf>("input_file").expect("required");
            let file = PathBuf::from(file);
            beetree::Input::File(file)
        };
        let verbose = self.get_flag("verbose");
        let languages = self
            .get_one::<String>("languages")
            .expect("required")
            .to_string();
        Ok(translate::Config {
            api_key,
            model,
            languages,
            host,
            input,
            output_file,
            verbose,
        })
    }
}
impl ToConfig<lang::Config> for ArgMatches {
    type Error = &'static str;
    fn to_config(&self) -> Result<lang::Config, Self::Error> {
        let search_file = self.get_one::<PathBuf>("search_file").map(Into::into);
        let action = if let Some(needle) = self.get_one::<String>("prepend_var") {
            Action::PrependFile(FindSpecified {
                needle: needle.to_string(),
                file: search_file,
            })
        } else {
            Action::Append(search_file.expect("guaranteed by clap"))
        };
        Ok(lang::Config {
            text: self.get_one::<String>("text").cloned(),
            input_file: self.get_one::<PathBuf>("input_file").map(|s| s.into()),
            action,
            base_path: self
                .get_one::<String>("base_path")
                .expect("guaranteed by clap")
                .into(),
            new_var: self
                .get_one::<String>("new_var")
                .cloned()
                .expect("guaranteed by clap"),
            verbose: self.get_flag("verbose"),
        })
    }
}
impl ToConfig<lang::RemoveConfig> for ArgMatches {
    type Error = anyhow::Error;
    fn to_config(&self) -> Result<lang::RemoveConfig, Self::Error> {
        let languages = self
            .get_one::<String>("languages")
            .expect("default")
            .to_owned();
        let base_path = self
            .get_one::<PathBuf>("base_path")
            .expect("default")
            .to_owned();
        let destination_tag = self
            .get_one::<String>("destination_tag")
            .expect("required")
            .to_owned();
        let file = self
            .get_one::<PathBuf>("search_file")
            .map(ToOwned::to_owned);
        let dst_tag = FindSpecified {
            needle: destination_tag,
            file,
        };
        Ok(lang::RemoveConfig {
            languages,
            base_path,
            dst_tag,
            verbose: self.get_flag("verbose"),
            yes: true,
        })
    }
}
impl ToConfig<lang::FindConfig> for ArgMatches {
    type Error = anyhow::Error;
    fn to_config(&self) -> Result<lang::FindConfig, Self::Error> {
        let languages = self
            .get_one::<String>("languages")
            .expect("default")
            .to_owned();
        let base_path = self
            .get_one::<PathBuf>("base_path")
            .expect("default")
            .to_owned();
        let destination_tag = self
            .get_one::<String>("destination_tag")
            .expect("required")
            .to_owned();
        let file = self
            .get_one::<PathBuf>("search_file")
            .map(ToOwned::to_owned);
        let dst_tag = FindSpecified {
            needle: destination_tag,
            file,
        };
        Ok(lang::FindConfig {
            languages,
            base_path,
            dst_tag,
            verbose: self.get_flag("verbose"),
        })
    }
}
impl ToConfig<lang::AppendConfig> for ArgMatches {
    type Error = anyhow::Error;
    fn to_config(&self) -> Result<lang::AppendConfig, Self::Error> {
        let base_path = self
            .get_one::<PathBuf>("base_path")
            .expect("default")
            .to_owned();
        let src_tag = self
            .get_one::<String>("source_tag")
            .expect("required")
            .to_owned();
        let file = self
            .get_one::<PathBuf>("search_file")
            .expect("required")
            .to_owned();
        let input = if let Some(text) = self.get_one::<String>("text") {
            beetree::Input::Text(text.to_string())
        } else {
            let file = self.get_one::<PathBuf>("input_file").expect("required");
            beetree::Input::File(file.to_owned())
        };
        Ok(lang::AppendConfig {
            base_path,
            src_tag,
            verbose: self.get_flag("verbose"),
            file,
            input,
        })
    }
}
impl ToConfig<lang::InsertConfig> for ArgMatches {
    type Error = anyhow::Error;
    fn to_config(&self) -> Result<lang::InsertConfig, Self::Error> {
        let base_path = self
            .get_one::<PathBuf>("base_path")
            .expect("default")
            .to_owned();
        let src_tag = self
            .get_one::<String>("source_tag")
            .expect("required")
            .to_owned();
        let input = if let Some(text) = self.get_one::<String>("text") {
            beetree::Input::Text(text.to_string())
        } else {
            let file = self.get_one::<PathBuf>("input_file").expect("required");
            beetree::Input::File(file.to_owned())
        };
        let destination_tag = self
            .get_one::<String>("destination_tag")
            .expect("required")
            .to_owned();
        let file = self
            .get_one::<PathBuf>("search_file")
            .map(ToOwned::to_owned);
        let dst_tag = FindSpecified {
            needle: destination_tag,
            file,
        };
        Ok(lang::InsertConfig {
            base_path,
            src_tag,
            dst_tag,
            verbose: self.get_flag("verbose"),
            input,
        })
    }
}
